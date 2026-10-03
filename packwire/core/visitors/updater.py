from typing import List, Tuple, Optional

from packwire.core.state import load_state, get_installed
from packwire.core.models import PackageState
from packwire.core.visitors.process_manifest import ManifestVisitor
from packwire.core.visitors.uninstaller import UninstallerVisitor
from packwire.core.installer import Installer


class UpdaterVisitor:
    """Visitor responsible for checking and applying updates to installed packages."""

    def __init__(
        self,
        manifest_visitor: Optional[ManifestVisitor] = None,
        installer: Optional[Installer] = None,
        uninstaller: Optional[UninstallerVisitor] = None
    ):
        self.manifest_visitor = manifest_visitor or ManifestVisitor()
        self.installer = installer or Installer()
        self.uninstaller = uninstaller or UninstallerVisitor()

    def check_updates(self) -> List[Tuple[PackageState, Optional[str]]]:
        """
        Check all installed packages for available updates.
        Returns a list of tuples: (current_state, latest_version_if_newer)
        """
        state = load_state()
        updates_available = []

        for pkg_id, pkg in state.items():
            if pkg.type != "stable":
                continue

            manifest = self.manifest_visitor.find_manifest(pkg_id)
            if not manifest:
                manifest = self.manifest_visitor.find_manifest(pkg.name)

            if manifest and manifest.version and manifest.version != pkg.version:
                updates_available.append((pkg, manifest.version))

        return updates_available

    def update(self, pkg_id: Optional[str] = None) -> List[Tuple[str, bool, str]]:
        """
        Update specific package or all eligible stable packages.
        Returns list of (pkg_id, success, message).
        """
        results = []
        state = load_state()

        targets = [state[pkg_id]] if pkg_id and pkg_id in state else list(state.values())

        if pkg_id and pkg_id not in state:
            return [(pkg_id, False, f"El paquete '{pkg_id}' no está instalado.")]

        for pkg in targets:
            if pkg.type == "fixed":
                results.append((
                    pkg.id,
                    False,
                    f"El paquete '{pkg.id}' es de tipo 'fixed' (fijo en v{pkg.version}). No se actualiza automáticamente."
                ))
                continue

            # Find latest manifest
            manifest = self.manifest_visitor.find_manifest(pkg.id) or self.manifest_visitor.find_manifest(pkg.name)
            if not manifest:
                results.append((pkg.id, False, f"No se encontró manifest para '{pkg.id}'."))
                continue

            if manifest.version == pkg.version:
                results.append((pkg.id, True, f"'{pkg.id}' ya está en la versión más reciente ({pkg.version})."))
                continue

            print(f"[Packwire Updater] Actualizando '{pkg.name}': {pkg.version} -> {manifest.version}")
            old_version = pkg.version
            # Run installation of new version
            success, msg = self.installer.install(manifest, mode_override=pkg.mode, force=True)
            if success:
                # Optionally clean up old version directory
                if pkg.install_path and pkg.mode == "here":
                    import shutil
                    from pathlib import Path
                    old_path = Path(pkg.install_path)
                    if old_path.exists() and old_path != Path(manifest.install.here.url or ""):
                        shutil.rmtree(old_path, ignore_errors=True)
                results.append((pkg.id, True, f"Actualizado a {manifest.version} correctamente."))
            else:
                results.append((pkg.id, False, f"Error al actualizar: {msg}"))

        return results
