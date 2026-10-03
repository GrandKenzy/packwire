"""
Packwire Visitors Package.
Contains workers/visitors for the manifest lifecycle:
- downloader: Handles downloading files with progress
- pather: Manages shims and PATH environment variable
- process_manifest: Validates and loads manifests
- uninstaller: Uninstalls packages and shims
- updater: Checks upstream and updates packages
"""

from .downloader import DownloaderVisitor
from .pather import PatherVisitor
from .process_manifest import ManifestVisitor
from .uninstaller import UninstallerVisitor
from .updater import UpdaterVisitor

__all__ = [
    "DownloaderVisitor",
    "PatherVisitor",
    "ManifestVisitor",
    "UninstallerVisitor",
    "UpdaterVisitor",
]
