"""
Packwire Packager & Build Automation.
Builds standalone Windows executables (packwire.exe) and Python distribution packages (wheel/sdist).
"""

import sys
import os
import shutil
import struct
import subprocess
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent


def ensure_icon() -> Path:
    """Ensure logo.ico exists in packwire/core/gui/web/, generating it from logo.png if needed."""
    web_dir = ROOT_DIR / "packwire" / "core" / "gui" / "web"
    ico_path = web_dir / "logo.ico"
    png_path = web_dir / "logo.png"

    if ico_path.exists() and ico_path.stat().st_size > 0:
        return ico_path

    if png_path.exists():
        try:
            png_data = png_path.read_bytes()
            header = struct.pack("<HHH", 0, 1, 1)
            entry = struct.pack("<BBBBHHII", 0, 0, 0, 0, 1, 32, len(png_data), 22)
            ico_path.write_bytes(header + entry + png_data)
            print(f"[+] Generado logo.ico ({len(png_data)} bytes) en {ico_path}")
            return ico_path
        except Exception as e:
            print(f"[!] Advertencia al generar logo.ico: {e}")

    return ico_path


def build_wheel() -> bool:
    """Build Python wheel and source distribution in dist/."""
    print("[*] Empaquetando distribución Python (Wheel & SDist)...")
    try:
        cmd = [sys.executable, "setup.py", "sdist", "bdist_wheel"]
        res = subprocess.run(cmd, cwd=str(ROOT_DIR), check=True)
        print("[+] Distribución Python construida con éxito en dist/")
        return True
    except Exception as e:
        print(f"[-] Error al construir wheel: {e}")
        return False


def build_exe(clean: bool = True) -> bool:
    """Build standalone packwire.exe using PyInstaller and packwire.spec."""
    ensure_icon()

    # Check if pyinstaller is installed
    try:
        import PyInstaller
    except ImportError:
        print("[*] PyInstaller no está instalado. Instalándolo vía pip...")
        try:
            subprocess.run([sys.executable, "-m", "pip", "install", "pyinstaller"], check=True)
        except Exception as e:
            print(f"[-] No se pudo instalar PyInstaller automáticamente: {e}")
            print("[!] Por favor ejecuta manualmente: pip install pyinstaller")
            return False

    spec_file = ROOT_DIR / "packwire.spec"
    if not spec_file.exists():
        print(f"[-] No se encontró el archivo spec: {spec_file}")
        return False

    dist_dir = ROOT_DIR / "dist"
    build_dir = ROOT_DIR / "build"

    if clean:
        if build_dir.exists():
            shutil.rmtree(build_dir, ignore_errors=True)
        exe_name = "packwire.exe" if sys.platform == "win32" else "packwire"
        dist_exe = dist_dir / exe_name
        if dist_exe.exists():
            try:
                dist_exe.unlink()
            except Exception:
                pass

    print(f"[*] Compilando ejecutable nativo con PyInstaller ({spec_file.name})...")
    cmd = [sys.executable, "-m", "PyInstaller", str(spec_file)]
    if clean:
        cmd.append("--clean")

    try:
        subprocess.run(cmd, cwd=str(ROOT_DIR), check=True)
        exe_name = "packwire.exe" if sys.platform == "win32" else "packwire"
        exe_path = dist_dir / exe_name
        if exe_path.exists():
            size_mb = exe_path.stat().st_size / (1024 * 1024)
            print(f"\n========================================================")
            print(f"  ¡EJECUTABLE PACKWIRE CONSTRUIDO CON ÉXITO!")
            print(f"  Ubicación : {exe_path}")
            print(f"  Tamaño    : {size_mb:.2f} MB")
            print(f"========================================================\n")
            return True
        else:
            print(f"[-] El proceso de PyInstaller terminó pero no se encontró {exe_name}.")
            return False
    except subprocess.CalledProcessError as e:
        print(f"[-] Falló la compilación de PyInstaller (código {e.returncode}).")
        return False


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Empaquetador y Constructor de Packwire")
    parser.add_argument("--exe", action="store_true", default=True, help="Compila el ejecutable standalone packwire.exe")
    parser.add_argument("--wheel", action="store_true", help="Construye paquete pip (.whl y .tar.gz)")
    parser.add_argument("--all", action="store_true", help="Construye tanto el ejecutable como la distribución wheel")

    args = parser.parse_args()

    if args.all:
        ok1 = build_wheel()
        ok2 = build_exe()
        sys.exit(0 if (ok1 and ok2) else 1)
    elif args.wheel:
        sys.exit(0 if build_wheel() else 1)
    else:
        sys.exit(0 if build_exe() else 1)


if __name__ == "__main__":
    main()
