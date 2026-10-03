import shutil
from pathlib import Path
from typing import Optional, Tuple, List

from packwire.core.config import APPS_DIR, CACHE_DIR, SHIMS_DIR, STATE_FILE, PACKWIRE_ROOT
from packwire.core.state import get_installed, unregister_installed, load_state, save_state
from packwire.core.visitors.pather import PatherVisitor
from packwire.core.models import PackageState


class UninstallerVisitor:
    """Visitor responsible for cleanly removing installed packages, shims, and all Packwire data."""

    def __init__(self, pather: Optional[PatherVisitor] = None):
        self.pather = pather or PatherVisitor()

    def uninstall(self, pkg_id: str) -> Tuple[bool, str]:
        """
        Uninstall a package by ID.
        Returns (success, message).
        """
        installed = get_installed(pkg_id)
        if not installed:
            return False, f"El paquete '{pkg_id}' no está registrado como instalado."

        # 1. Remove shims
        if installed.shims:
            self.pather.remove_shims(installed.shims)

        # 2. Remove files if 'here' mode
        if installed.install_path:
            install_dir = Path(installed.install_path)
            if install_dir.exists():
                try:
                    shutil.rmtree(install_dir)
                    print(f"[Packwire Uninstaller] Directorio eliminado: {install_dir}")
                except Exception as e:
                    print(f"[Packwire Uninstaller] Advertencia al eliminar directorio: {e}")

        # 3. Unregister from state.json
        unregister_installed(pkg_id)

        return True, f"Paquete '{pkg_id}' (versión {installed.version}) desinstalado correctamente."

    def uninstall_all(self, remove_path: bool = True) -> Tuple[bool, str]:
        """
        Completely uninstall all Packwire packages, shims, cache, and remove from PATH.
        Returns (success, message).
        """
        cleaned_items: List[str] = []
        errors: List[str] = []

        # 1. Uninstall all tracked packages individually
        state = load_state()
        for pkg_id in list(state.keys()):
            try:
                ok, msg = self.uninstall(pkg_id)
                if ok:
                    cleaned_items.append(f"Paquete '{pkg_id}' desinstalado.")
                else:
                    errors.append(msg)
            except Exception as e:
                errors.append(f"Error al desinstalar '{pkg_id}': {e}")

        # 2. Clean any remaining shims in SHIMS_DIR
        if self.pather.shims_dir.exists():
            try:
                shim_count = 0
                for item in self.pather.shims_dir.iterdir():
                    if item.is_file():
                        item.unlink(missing_ok=True)
                        shim_count += 1
                cleaned_items.append(f"{shim_count} archivos shim eliminados.")
            except Exception as e:
                errors.append(f"Error al limpiar carpeta de shims: {e}")

        # 3. Clean apps directory
        if APPS_DIR.exists():
            try:
                shutil.rmtree(APPS_DIR, ignore_errors=True)
                APPS_DIR.mkdir(parents=True, exist_ok=True)
                cleaned_items.append("Carpeta de aplicaciones (apps) limpiada.")
            except Exception as e:
                errors.append(f"Error al limpiar apps: {e}")

        # 4. Clean cache directory
        if CACHE_DIR.exists():
            try:
                shutil.rmtree(CACHE_DIR, ignore_errors=True)
                CACHE_DIR.mkdir(parents=True, exist_ok=True)
                cleaned_items.append("Caché temporal de descargas eliminada.")
            except Exception as e:
                errors.append(f"Error al limpiar caché: {e}")

        # 5. Remove shims directory from user PATH
        if remove_path:
            try:
                if self.pather.remove_shims_from_user_path():
                    cleaned_items.append("Ruta de Packwire retirada de la variable PATH del sistema.")
                else:
                    errors.append("No se pudo remover del PATH o no estaba presente.")
            except Exception as e:
                errors.append(f"Error al remover del PATH: {e}")

        # 6. Reset installed.json state
        try:
            save_state({})
            cleaned_items.append("Registro de paquetes (installed.json) restablecido a cero.")
        except Exception as e:
            errors.append(f"Error al restablecer installed.json: {e}")

        summary = "Desinstalación completa de Packwire realizada con éxito:\n"
        summary += "\n".join(f"• {item}" for item in cleaned_items)
        if errors:
            summary += "\n\nAvisos secundarios:\n" + "\n".join(f"• {err}" for err in errors)

        return True, summary

