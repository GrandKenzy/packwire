"""
Packwire Self-Installer & System Integration.
Integrates Packwire into Windows:
- Creates launcher shims (packwire.cmd, packwire.ps1) in %APPDATA%/packwire/shims
- Registers shims directory in User PATH (registry + WM_SETTINGCHANGE broadcast)
- Creates Start Menu and Desktop shortcuts with custom icon
"""

import sys
import os
import shutil
import subprocess
from pathlib import Path
from typing import Tuple, Optional

from packwire.core.config import SHIMS_DIR, PACKWIRE_ROOT, APPS_DIR, SYSTEM, ensure_directories
from packwire.core.visitors.pather import PatherVisitor


def create_windows_shortcut(
    target_path: Path,
    shortcut_path: Path,
    arguments: str = "",
    icon_path: Optional[Path] = None,
    description: str = "Packwire - Gestor de Paquetes"
) -> bool:
    """Create a Windows .lnk shortcut using PowerShell WScript.Shell."""
    if SYSTEM != "windows":
        return False

    shortcut_path.parent.mkdir(parents=True, exist_ok=True)
    icon_line = f'$s.IconLocation = "{str(icon_path)}";' if icon_path and icon_path.exists() else ''

    ps_script = f"""
    $w = New-Object -ComObject WScript.Shell
    $s = $w.CreateShortcut('{str(shortcut_path)}')
    $s.TargetPath = '{str(target_path)}'
    $s.Arguments = '{arguments}'
    $s.WorkingDirectory = '{str(target_path.parent)}'
    $s.Description = '{description}'
    {icon_line}
    $s.Save()
    """
    try:
        res = subprocess.run(
            ["powershell", "-NoProfile", "-NonInteractive", "-Command", ps_script],
            capture_output=True,
            text=True
        )
        return res.returncode == 0
    except Exception as e:
        print(f"[!] Advertencia al crear acceso directo {shortcut_path.name}: {e}")
        return False


def install_self(
    add_to_path: bool = True,
    create_start_menu: bool = True,
    create_desktop: bool = False
) -> Tuple[bool, str]:
    """
    Install and integrate Packwire into the local environment.
    """
    ensure_directories()
    pather = PatherVisitor()
    messages = []

    is_frozen = getattr(sys, "frozen", False)
    root_dir = Path(__file__).resolve().parent.parent

    # Find or generate icon
    ico_path = root_dir / "packwire" / "core" / "gui" / "web" / "logo.ico"
    if not ico_path.exists():
        from packwire.packager import ensure_icon
        ensure_icon()

    # 1. Create Packwire Shims in SHIMS_DIR
    cmd_shim = SHIMS_DIR / "packwire.cmd"
    ps1_shim = SHIMS_DIR / "packwire.ps1"

    if is_frozen:
        exe_path = Path(sys.executable).resolve()
        cmd_content = f'@echo off\r\n"{str(exe_path)}" %*\r\n'
        ps1_content = f'& "{str(exe_path)}" $args\r\n'
        target_executable = exe_path
        gui_target = exe_path
        gui_args = "ui"
    else:
        python_exe = Path(sys.executable).resolve()
        pythonw_exe = python_exe.parent / "pythonw.exe"
        # Support python script invocation
        cmd_content = f'@echo off\r\n"{str(python_exe)}" -m packwire %*\r\n'
        ps1_content = f'& "{str(python_exe)}" -m packwire $args\r\n'
        target_executable = cmd_shim
        if pythonw_exe.exists():
            gui_target = pythonw_exe
            gui_args = "-m packwire ui"
        else:
            gui_target = target_executable
            gui_args = "ui"

    cmd_shim.write_text(cmd_content, encoding="utf-8")
    ps1_shim.write_text(ps1_content, encoding="utf-8")
    messages.append(f"Lanzador shim creado en {cmd_shim}")

    # 2. Add SHIMS_DIR to User PATH
    if add_to_path:
        if pather.add_shims_to_user_path():
            messages.append("Directorio de shims añadido a la variable PATH del usuario.")
        else:
            messages.append("El directorio de shims ya estaba en el PATH o no requirió cambios.")

    # 3. Create Start Menu Shortcut (opens GUI by default)
    if create_start_menu and SYSTEM == "windows":
        app_data = os.environ.get("APPDATA")
        if app_data:
            start_menu_dir = Path(app_data) / "Microsoft" / "Windows" / "Start Menu" / "Programs"
            shortcut_file = start_menu_dir / "Packwire.lnk"
            if create_windows_shortcut(gui_target, shortcut_file, arguments=gui_args, icon_path=ico_path, description="Packwire Package Manager"):
                messages.append(f"Acceso directo creado en Menú Inicio: {shortcut_file.name}")

    # 4. Optional Desktop Shortcut
    if create_desktop and SYSTEM == "windows":
        user_profile = os.environ.get("USERPROFILE")
        if user_profile:
            desktop_dir = Path(user_profile) / "Desktop"
            shortcut_file = desktop_dir / "Packwire.lnk"
            if create_windows_shortcut(gui_target, shortcut_file, arguments=gui_args, icon_path=ico_path, description="Packwire Package Manager"):
                messages.append(f"Acceso directo creado en el Escritorio: {shortcut_file.name}")

    summary = "¡Packwire instalado e integrado con éxito!\n" + "\n".join(f"• {m}" for m in messages)
    return True, summary


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Instalador y configurador del sistema para Packwire")
    parser.add_argument("--desktop", action="store_true", help="Crea acceso directo en el Escritorio")
    parser.add_argument("--no-start-menu", action="store_true", help="No crea acceso directo en el Menú Inicio")
    parser.add_argument("--no-path", action="store_true", help="No añade Packwire a la variable PATH")

    args = parser.parse_args()

    print("[*] Iniciando instalación y configuración de Packwire...")
    ok, msg = install_self(
        add_to_path=not args.no_path,
        create_start_menu=not args.no_start_menu,
        create_desktop=args.desktop
    )
    if ok:
        print(f"[+] {msg}")
    else:
        print(f"[-] {msg}")
        sys.exit(1)


if __name__ == "__main__":
    main()
