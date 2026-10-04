import argparse
import sys
import json
from typing import List

if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

import packwire


def main(args: List[str] = None):
    parser = argparse.ArgumentParser(
        prog="packwire",
        description="Packwire: Gestor y descargador moderno de paquetes para Windows"
    )
    subparsers = parser.add_subparsers(dest="command", help="Comandos disponibles")

    # Command: install
    install_parser = subparsers.add_parser("install", help="Instala un paquete")
    install_parser.add_argument("package", help="Nombre o identificador del paquete (ej. python, python@stable, python@3.14)")
    install_parser.add_argument("--version", "-v", help="Versión específica a instalar")
    install_parser.add_argument(
        "--channel",
        choices=["stable", "fixed"],
        default="stable",
        help="Canal del paquete (stable o fixed)"
    )
    install_parser.add_argument(
        "--mode",
        "-m",
        choices=["here", "site"],
        default="here",
        help="Modo: 'here' (descarga automática sin navegador) o 'site' (redirige a la web oficial)"
    )
    install_parser.add_argument(
        "--force",
        "-f",
        action="store_true",
        help="Fuerza la reinstalación o sobrescritura de shims"
    )

    # Command: remove / uninstall
    remove_parser = subparsers.add_parser("remove", aliases=["uninstall"], help="Desinstala un paquete o todo Packwire")
    remove_parser.add_argument("package", nargs="?", default=None, help="Identificador del paquete a desinstalar (ej. python@stable)")
    remove_parser.add_argument("--all", action="store_true", help="Desinstala absolutamente todo lo de Packwire (paquetes, shims, caché y PATH)")
    remove_parser.add_argument("--keep-path", action="store_true", help="No remueve la ruta de shims del PATH")

    # Command: reinstall
    reinstall_parser = subparsers.add_parser("reinstall", help="Reinstala un paquete ya instalado")
    reinstall_parser.add_argument("package", help="Identificador del paquete a reinstalar (ej. python@stable)")

    # Command: update
    update_parser = subparsers.add_parser("update", help="Actualiza paquetes stable")
    update_parser.add_argument("package", nargs="?", default=None, help="Paquete específico a actualizar")

    # Command: list
    list_parser = subparsers.add_parser("list", help="Lista los paquetes instalados")

    # Command: available
    available_parser = subparsers.add_parser("available", help="Lista los paquetes disponibles en el catálogo")

    # Command: info
    info_parser = subparsers.add_parser("info", help="Muestra información del manifest de un paquete")
    info_parser.add_argument("package", help="Nombre o identificador del paquete")

    # Command: path
    path_parser = subparsers.add_parser("path", help="Configura o verifica la carpeta de shims en el PATH")
    path_parser.add_argument("--add", action="store_true", help="Añade la carpeta de shims al PATH del usuario")

    # Command: ui / gui
    ui_parser = subparsers.add_parser("ui", aliases=["gui"], help="Inicia la interfaz gráfica moderna de Packwire")
    ui_parser.add_argument("--web", action="store_true", help="Fuerza el modo servidor web abriendo en el navegador")
    ui_parser.add_argument("--port", type=int, default=5050, help="Puerto para la interfaz (por defecto: 5050)")

    # Command: setup / install-self
    setup_parser = subparsers.add_parser("setup", aliases=["self-install"], help="Instala e integra Packwire en el sistema (PATH, shims, accesos)")
    setup_parser.add_argument("--desktop", action="store_true", help="Crea acceso directo en el Escritorio")
    setup_parser.add_argument("--no-start-menu", action="store_true", help="No crea acceso directo en el Menú Inicio")
    setup_parser.add_argument("--no-path", action="store_true", help="No añade Packwire al PATH")

    # Command: build / pack
    build_parser = subparsers.add_parser("build", aliases=["pack"], help="Empaqueta Packwire en ejecutable standalone o wheel")
    build_parser.add_argument("--exe", action="store_true", help="Compila el ejecutable nativo standalone packwire.exe")
    build_parser.add_argument("--wheel", action="store_true", help="Construye paquete de distribución wheel")
    build_parser.add_argument("--all", action="store_true", help="Construye tanto el ejecutable como el wheel")

    if args is None:
        raw_args = sys.argv[1:]
    else:
        raw_args = args

    if not raw_args:
        exe_lower = sys.executable.lower()
        if "packwirew" in exe_lower or "pythonw" in exe_lower:
            raw_args = ["ui"]

    parsed = parser.parse_args(raw_args)

    if not parsed.command:
        parser.print_help()
        sys.exit(0)

    if parsed.command == "install":
        print(f"[*] Iniciando instalación de '{parsed.package}'...")
        success, msg = packwire.install(
            target=parsed.package,
            version=parsed.version,
            channel=parsed.channel,
            mode=parsed.mode,
            force=parsed.force
        )
        if success:
            print(f"[+] Éxito: {msg}")
        else:
            print(f"[-] Falló: {msg}")
            sys.exit(1)

    elif parsed.command in ("remove", "uninstall"):
        if parsed.all:
            print("[*] Desinstalando todo lo de Packwire (paquetes, shims, caché y PATH)...")
            success, msg = packwire.uninstall_all(remove_path=not parsed.keep_path)
            if success:
                print(f"[+] {msg}")
            else:
                print(f"[-] {msg}")
                sys.exit(1)
        elif parsed.package:
            success, msg = packwire.uninstall(parsed.package)
            if success:
                print(f"[+] {msg}")
            else:
                print(f"[-] {msg}")
                sys.exit(1)
        else:
            print("[-] Debes especificar el nombre de un paquete o usar el parámetro --all.")
            sys.exit(1)

    elif parsed.command == "reinstall":
        print(f"[*] Reinstalando '{parsed.package}'...")
        success, msg = packwire.reinstall(parsed.package)
        if success:
            print(f"[+] Éxito: {msg}")
        else:
            print(f"[-] Falló: {msg}")
            sys.exit(1)

    elif parsed.command == "update":
        results = packwire.update(parsed.package)
        for pkg_id, ok, msg in results:
            prefix = "[+]" if ok else "[-]"
            print(f"{prefix} {pkg_id}: {msg}")

    elif parsed.command == "list":
        installed = packwire.list_installed()
        if not installed:
            print("[*] No hay paquetes instalados actualmente en Packwire.")
            return

        print(f"{'ID':<20} {'NOMBRE':<12} {'VERSIÓN':<10} {'TIPO':<8} {'MODO':<8} {'RUTA'}")
        print("-" * 80)
        for pkg_id, info in installed.items():
            path_str = info.get("install_path") or "(Navegador/Site)"
            print(f"{pkg_id:<20} {info.get('name'):<12} {info.get('version', ''):<10} {info.get('type', ''):<8} {info.get('mode', ''):<8} {path_str}")

    elif parsed.command == "available":
        available = packwire.list_available()
        print(f"{'ID':<20} {'NOMBRE':<12} {'TIPO':<8} {'CATEGORÍA':<22} {'DESCRIPCIÓN'}")
        print("-" * 80)
        for m in available:
            print(f"{m.get('id', ''):<20} {m.get('name', ''):<12} {m.get('type', ''):<8} {m.get('category', ''):<22} {m.get('description', '')}")

    elif parsed.command == "info":
        m = packwire.get_manifest(parsed.package)
        if not m:
            print(f"[-] No se encontró información para '{parsed.package}'.")
            sys.exit(1)
        print(json.dumps(m, indent=2, ensure_ascii=False))

    elif parsed.command == "path":
        if parsed.add:
            packwire.add_path()
        else:
            in_path = packwire._pather.is_shims_in_path()
            print(f"Directorio de shims: {packwire.SHIMS_DIR}")
            print(f"¿Está en PATH? {'SÍ' if in_path else 'NO (usa --add para agregarlo)'}")

    elif parsed.command in ("ui", "gui"):
        packwire.launch_gui(web_mode=parsed.web, port=parsed.port)

    elif parsed.command in ("setup", "self-install"):
        from packwire.installer_self import install_self
        ok, msg = install_self(
            add_to_path=not parsed.no_path,
            create_start_menu=not parsed.no_start_menu,
            create_desktop=parsed.desktop
        )
        if ok:
            print(f"[+] {msg}")
        else:
            print(f"[-] {msg}")
            sys.exit(1)

    elif parsed.command in ("build", "pack"):
        from packwire.packager import build_exe, build_wheel
        if parsed.all:
            ok1 = build_wheel()
            ok2 = build_exe()
            sys.exit(0 if (ok1 and ok2) else 1)
        elif parsed.wheel:
            sys.exit(0 if build_wheel() else 1)
        else:
            sys.exit(0 if build_exe() else 1)


if __name__ == "__main__":
    main()
