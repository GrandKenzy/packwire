import os
import sys
import stat
from pathlib import Path
from typing import List, Optional

from packwire.core.config import SHIMS_DIR, SYSTEM, ensure_directories

# Optional Windows-specific imports
try:
    import winreg
    import ctypes
    HAS_WIN32 = True
except ImportError:
    HAS_WIN32 = False


class PatherVisitor:
    """Visitor responsible for creating binary shims and configuring PATH across OS platforms."""

    def __init__(self, shims_dir: Optional[Path] = None):
        self.shims_dir = shims_dir or SHIMS_DIR
        ensure_directories()

    def create_shims(
        self,
        target_dir: Path,
        binaries: List[str],
        version_suffix: Optional[str] = None,
        is_default: bool = True
    ) -> List[str]:
        """
        Create wrapper shims inside SHIMS_DIR pointing to binaries in target_dir.
        - On Windows: creates .cmd and .ps1 shims.
        - On POSIX (Linux/macOS): creates executable shell script shims with chmod +x.
        """
        created_shims: List[str] = []

        for b in binaries:
            stem = Path(b).stem
            candidates_to_try = [b]
            if SYSTEM == "windows":
                if not b.endswith(".exe"):
                    candidates_to_try.append(f"{b}.exe")
            else:
                if b.endswith(".exe"):
                    candidates_to_try.insert(0, stem)
                else:
                    candidates_to_try.append(stem)

            target_exe = None
            for cand in candidates_to_try:
                if (target_dir / cand).exists():
                    target_exe = (target_dir / cand).resolve()
                    break
                elif (target_dir / "bin" / cand).exists():
                    target_exe = (target_dir / "bin" / cand).resolve()
                    break

            if not target_exe:
                for cand in candidates_to_try:
                    found = list(target_dir.rglob(cand))
                    if found:
                        target_exe = found[0].resolve()
                        break

            if not target_exe:
                target_exe = (target_dir / b).resolve()

            # Ensure target binary is executable on POSIX platforms
            if SYSTEM != "windows" and target_exe.exists():
                try:
                    cur_mode = target_exe.stat().st_mode
                    target_exe.chmod(cur_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
                except Exception:
                    pass

            base_name = Path(b).stem.lower()

            names_to_create = []
            if is_default:
                names_to_create.append(base_name)

            if version_suffix:
                clean_ver = version_suffix.replace(".", "").replace("-", "")
                names_to_create.append(f"{base_name}{clean_ver}")
                names_to_create.append(f"{base_name}-{version_suffix}")

            for name in names_to_create:
                if SYSTEM == "windows":
                    cmd_shim = self.shims_dir / f"{name}.cmd"
                    ps1_shim = self.shims_dir / f"{name}.ps1"

                    cmd_content = f'@echo off\r\n"{str(target_exe)}" %*\r\n'
                    cmd_shim.write_text(cmd_content, encoding="utf-8")

                    ps1_content = f'& "{str(target_exe)}" $args\r\n'
                    ps1_shim.write_text(ps1_content, encoding="utf-8")

                    created_shims.append(f"{name}.cmd")
                    print(f"[Packwire Pather] Shim creado: {cmd_shim.name} -> {target_exe.name}")
                else:
                    # Linux / macOS POSIX shim
                    posix_shim = self.shims_dir / name
                    script_content = f'#!/bin/sh\nexec "{str(target_exe)}" "$@"\n'
                    posix_shim.write_text(script_content, encoding="utf-8")
                    # Set executable permissions (rwxr-xr-x)
                    current_stat = posix_shim.stat()
                    posix_shim.chmod(current_stat.st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)

                    created_shims.append(name)
                    print(f"[Packwire Pather] Shim POSIX creado: {posix_shim.name} -> {target_exe.name}")

        return list(set(created_shims))

    def remove_shims(self, shim_names: List[str]) -> None:
        """Remove specified shims from SHIMS_DIR."""
        for name in shim_names:
            base = Path(name).stem
            # Check both extensionless (POSIX) and Windows extensions
            candidates = [self.shims_dir / name, self.shims_dir / base]
            for ext in (".cmd", ".ps1", ".bat", ""):
                candidates.append(self.shims_dir / f"{base}{ext}")

            for shim_file in candidates:
                if shim_file.exists():
                    shim_file.unlink(missing_ok=True)
                    print(f"[Packwire Pather] Shim eliminado: {shim_file.name}")

    def is_shims_in_path(self) -> bool:
        """Check if SHIMS_DIR is already present in current session PATH or User Registry PATH."""
        shims_str = str(self.shims_dir).lower()
        current_path = os.environ.get("PATH", "").lower()
        if shims_str in current_path.split(os.pathsep):
            return True

        if SYSTEM == "windows" and HAS_WIN32:
            try:
                with winreg.OpenKey(winreg.HKEY_CURRENT_USER, r"Environment", 0, winreg.KEY_READ) as key:
                    user_path, _ = winreg.QueryValueEx(key, "Path")
                    if shims_str in user_path.lower().split(";"):
                        return True
            except Exception:
                pass

        return False

    def add_shims_to_user_path(self) -> bool:
        """
        Add SHIMS_DIR to the User PATH.
        - Windows: Writes to HKCU\\Environment and broadcasts WM_SETTINGCHANGE.
        - Linux/macOS: Appends export PATH to ~/.bashrc or ~/.zshrc.
        """
        shims_str = str(self.shims_dir)

        if self.is_shims_in_path():
            print(f"[Packwire Pather] El directorio de shims ya está en el PATH: {shims_str}")
            return True

        if SYSTEM == "windows" and HAS_WIN32:
            try:
                with winreg.OpenKey(
                    winreg.HKEY_CURRENT_USER,
                    r"Environment",
                    0,
                    winreg.KEY_READ | winreg.KEY_WRITE
                ) as key:
                    try:
                        user_path, val_type = winreg.QueryValueEx(key, "Path")
                    except FileNotFoundError:
                        user_path = ""
                        val_type = winreg.REG_EXPAND_SZ

                    paths = [p for p in user_path.split(";") if p.strip()]
                    if shims_str not in paths:
                        paths.insert(0, shims_str)
                        new_path = ";".join(paths)
                        winreg.SetValueEx(key, "Path", 0, val_type, new_path)
                        print(f"[Packwire Pather] Añadido exitosamente al PATH del usuario en Windows: {shims_str}")

                os.environ["PATH"] = f"{shims_str}{os.pathsep}{os.environ.get('PATH', '')}"

                HWND_BROADCAST = 0xFFFF
                WM_SETTINGCHANGE = 0x001A
                SMTO_ABORTIFHUNG = 0x0002
                result = ctypes.c_long()
                ctypes.windll.user32.SendMessageTimeoutW(
                    HWND_BROADCAST,
                    WM_SETTINGCHANGE,
                    0,
                    "Environment",
                    SMTO_ABORTIFHUNG,
                    5000,
                    ctypes.byref(result)
                )
                return True
            except Exception as e:
                print(f"[Packwire Pather] Error al actualizar PATH en el registro: {e}")
                return False
        else:
            # POSIX (Linux/macOS)
            export_line = f'\nexport PATH="{shims_str}:$PATH"\n'
            rc_files = [Path.home() / ".zshrc", Path.home() / ".bashrc", Path.home() / ".profile"]
            written = False
            for rc in rc_files:
                if rc.exists():
                    try:
                        content = rc.read_text(encoding="utf-8")
                        if shims_str not in content:
                            with open(rc, "a", encoding="utf-8") as f:
                                f.write(export_line)
                            print(f"[Packwire Pather] Añadido PATH en {rc.name}: {shims_str}")
                            written = True
                    except Exception:
                        pass
            os.environ["PATH"] = f"{shims_str}{os.pathsep}{os.environ.get('PATH', '')}"
            return written or True

    def remove_shims_from_user_path(self) -> bool:
        r"""
        Remove SHIMS_DIR from the User PATH environment variable.
        - Windows: Removes from HKCU\Environment and broadcasts WM_SETTINGCHANGE.
        - Linux/macOS: Removes export line from shell rc files.
        """
        shims_str = str(self.shims_dir).lower()

        if SYSTEM == "windows" and HAS_WIN32:
            try:
                with winreg.OpenKey(
                    winreg.HKEY_CURRENT_USER,
                    r"Environment",
                    0,
                    winreg.KEY_READ | winreg.KEY_WRITE
                ) as key:
                    try:
                        user_path, val_type = winreg.QueryValueEx(key, "Path")
                    except FileNotFoundError:
                        return True

                    paths = [p for p in user_path.split(";") if p.strip()]
                    filtered_paths = [p for p in paths if p.strip().lower() != shims_str]

                    if len(filtered_paths) != len(paths):
                        new_path = ";".join(filtered_paths)
                        winreg.SetValueEx(key, "Path", 0, val_type, new_path)
                        print(f"[Packwire Pather] Removido exitosamente del PATH del usuario en Windows: {self.shims_dir}")

                # Update current process PATH
                current_paths = [p for p in os.environ.get("PATH", "").split(os.pathsep) if p.strip()]
                os.environ["PATH"] = os.pathsep.join([p for p in current_paths if p.strip().lower() != shims_str])

                # Broadcast setting change
                HWND_BROADCAST = 0xFFFF
                WM_SETTINGCHANGE = 0x001A
                SMTO_ABORTIFHUNG = 0x0002
                result = ctypes.c_long()
                ctypes.windll.user32.SendMessageTimeoutW(
                    HWND_BROADCAST,
                    WM_SETTINGCHANGE,
                    0,
                    "Environment",
                    SMTO_ABORTIFHUNG,
                    5000,
                    ctypes.byref(result)
                )
                return True
            except Exception as e:
                print(f"[Packwire Pather] Error al remover PATH del registro: {e}")
                return False
        else:
            # POSIX (Linux/macOS)
            rc_files = [Path.home() / ".zshrc", Path.home() / ".bashrc", Path.home() / ".profile"]
            for rc in rc_files:
                if rc.exists():
                    try:
                        lines = rc.read_text(encoding="utf-8").splitlines()
                        filtered = [l for l in lines if str(self.shims_dir) not in l]
                        if len(filtered) != len(lines):
                            rc.write_text("\n".join(filtered) + "\n", encoding="utf-8")
                            print(f"[Packwire Pather] Removido PATH de {rc.name}")
                    except Exception:
                        pass
            current_paths = [p for p in os.environ.get("PATH", "").split(os.pathsep) if p.strip()]
            os.environ["PATH"] = os.pathsep.join([p for p in current_paths if p.strip().lower() != shims_str])
            return True
