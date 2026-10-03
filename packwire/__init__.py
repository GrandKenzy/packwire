"""
Packwire - Gestor y descargador moderno de paquetes para Windows.
"""

from typing import Optional, List, Dict, Tuple, Any
from pathlib import Path

from packwire.core.models import Manifest, PackageState
from packwire.core.config import (
    PACKWIRE_ROOT, APPS_DIR, SHIMS_DIR, CACHE_DIR, STATE_FILE, CONFIG_FILE,
    load_config, save_config, get_config, set_config
)
from packwire.core.state import load_state, get_installed
from packwire.core.visitors.downloader import DownloaderVisitor
from packwire.core.visitors.pather import PatherVisitor
from packwire.core.visitors.process_manifest import ManifestVisitor
from packwire.core.visitors.uninstaller import UninstallerVisitor
from packwire.core.visitors.updater import UpdaterVisitor
from packwire.core.installer import Installer
from packwire.core.resolver import resolve_manifest

__version__ = "0.1.0"

_downloader = DownloaderVisitor()
_pather = PatherVisitor()
_manifest_visitor = ManifestVisitor()
_uninstaller = UninstallerVisitor(_pather)
_installer = Installer(_downloader, _pather, _manifest_visitor)
_updater = UpdaterVisitor(_manifest_visitor, _installer, _uninstaller)


def install(
    target: str,
    version: Optional[str] = None,
    channel: str = "stable",
    mode: Optional[str] = None,
    force: bool = False,
    logger: Optional[Any] = None
) -> Tuple[bool, str]:
    """
    Installs a package.
    Args:
        target: Package name or spec (e.g. 'python', 'python@stable', 'python@3.14')
        version: Optional explicit version (e.g. '3.14.8')
        channel: 'stable' or 'fixed'
        mode: 'here', 'command', 'site' (or None to use manifest's default)
        force: Overwrite existing installation or ignore collisions
        logger: Optional callback function receiving log messages
    """
    # 1. Check if target already includes channel or version
    if "@" in target:
        manifest = _manifest_visitor.find_manifest(target)
    else:
        # Check specific version or channel requested
        if version:
            manifest = _manifest_visitor.find_manifest(f"{target}@{version}")
            if not manifest:
                manifest = _manifest_visitor.find_manifest(target)
                if manifest:
                    manifest.version = version
                    manifest.type = "fixed"
                    manifest.id = f"{target.lower()}@{version}"
                    manifest = resolve_manifest(manifest)
        elif channel == "stable":
            manifest = _manifest_visitor.find_manifest(f"{target}@stable") or _manifest_visitor.find_manifest(target)
            if manifest:
                manifest.type = "stable"
                manifest.id = f"{target.lower()}@stable"
                manifest = resolve_manifest(manifest)
        else:
            manifest = _manifest_visitor.find_manifest(target)

    if not manifest:
        return False, f"No se encontró ningún manifest para '{target}'."

    return _installer.install(manifest, mode_override=mode, force=force, logger=logger)


def uninstall(pkg_id: str) -> Tuple[bool, str]:
    """Uninstall a package by identifier."""
    return _uninstaller.uninstall(pkg_id)


def uninstall_all(remove_path: bool = True) -> Tuple[bool, str]:
    """Completely uninstall all Packwire packages, shims, cache, and remove from PATH."""
    return _uninstaller.uninstall_all(remove_path=remove_path)


def reinstall(pkg_id: str) -> Tuple[bool, str]:
    """Reinstall an already installed package cleanly."""
    state = load_state()
    if pkg_id not in state:
        return False, f"El paquete '{pkg_id}' no está registrado como instalado."
    pkg = state[pkg_id]
    manifest = _manifest_visitor.find_manifest(pkg.id) or _manifest_visitor.find_manifest(pkg.name)
    if not manifest:
        return False, f"No se encontró el manifest para '{pkg_id}'."
    if pkg.type == "fixed" and pkg.version:
        manifest.version = pkg.version
        manifest.type = "fixed"
        manifest.id = pkg.id
        from packwire.core.resolver import resolve_manifest
        manifest = resolve_manifest(manifest)
    return _installer.install(manifest, mode_override=pkg.mode, force=True)


def update(pkg_id: Optional[str] = None) -> List[Tuple[str, bool, str]]:
    """Check and apply updates for stable packages."""
    return _updater.update(pkg_id)


def check_updates() -> Dict[str, Dict[str, Any]]:
    """Check in background if updates are available for installed non-fixed packages."""
    updates = _updater.check_updates()
    return {
        pkg.id: {
            "has_update": True,
            "current_version": pkg.version,
            "latest_version": latest_ver,
            "name": pkg.name,
            "mode": pkg.mode,
            "type": pkg.type
        }
        for pkg, latest_ver in updates
    }


def list_installed() -> Dict[str, Dict[str, Any]]:
    """List all currently installed packages in packwire."""
    state = load_state()
    return {pkg_id: pkg.to_dict() for pkg_id, pkg in state.items()}


def list_available() -> List[Dict[str, Any]]:
    """List all available manifests."""
    manifests = _manifest_visitor.list_available_manifests()
    return [m.to_dict() for m in manifests]


def get_manifest(target: str) -> Optional[Dict[str, Any]]:
    """Retrieve manifest details for a given package target."""
    m = _manifest_visitor.find_manifest(target)
    return m.to_dict() if m else None


def add_path() -> bool:
    """Ensure packwire shims directory is in User PATH."""
    return _pather.add_shims_to_user_path()


def launch_gui(web_mode: bool = False, port: int = 5050):
    """Launch the modern HTML/CSS Packwire User Interface."""
    from packwire.core.gui.window import launch_gui as _launch
    return _launch(web_mode=web_mode, port=port)


from packwire.core.task_queue import task_queue, TaskQueue, TaskJob

__all__ = [
    "install",
    "uninstall",
    "uninstall_all",
    "reinstall",
    "update",
    "check_updates",
    "list_installed",
    "list_available",
    "get_manifest",
    "add_path",
    "launch_gui",
    "task_queue",
    "TaskQueue",
    "TaskJob",
    "DownloaderVisitor",
    "PatherVisitor",
    "ManifestVisitor",
    "UninstallerVisitor",
    "UpdaterVisitor",
    "Installer",
    "Manifest",
    "PackageState",
    "PACKWIRE_ROOT",
    "APPS_DIR",
    "SHIMS_DIR",
    "CACHE_DIR",
    "CONFIG_FILE",
    "load_config",
    "save_config",
    "get_config",
    "set_config",
]
