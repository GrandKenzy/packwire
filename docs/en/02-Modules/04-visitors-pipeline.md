# Visitors Pipeline

Packwire implements the Visitor architectural pattern to decouple domain data models from mutating operations.

---

## 1. The Pipeline Visitors

### `DownloaderVisitor`
Responsible for network downloads:
- Atomic chunking with 64 KB buffers.
- Live progress callbacks (`downloaded_bytes`, `total_bytes`, `speed_bps`).
- Cache reuse when files already exist and match expected SHA-256 digests.
- Cleanup of temporary `.part` files on failure.

### `PatherVisitor`
Responsible for binary shims and system PATH configuration:
- Creates `.cmd` and `.ps1` shims for Windows executables.
- Creates POSIX shell scripts with executable permissions (`chmod +x`) on Linux/macOS.
- Updates Windows Registry (`HKCU\Environment`) and broadcasts `WM_SETTINGCHANGE`.
- Updates shell configuration files (`~/.bashrc`, `~/.zshrc`, `~/.profile`).
- Configures Windows Defender exclusions via `Add-MpPreference` to prevent heuristic false positives on GNU assembler (`as.exe`).

### `ManifestVisitor`
Responsible for manifest lifecycle:
- Tolerant JSON parser with support for inline `//` comments and trailing commas.
- Schema validation against dataclass models.
- Manifest discovery by ID, name, or aliases.

### `UninstallerVisitor`
Responsible for complete cleanups:
- Removes generated shims from `shims/`.
- Purges application folders in `apps/`.
- Updates `installed.json`.
- Supports `uninstall_all` to cleanly remove all Packwire traces from the host machine.

### `UpdaterVisitor`
Responsible for version synchronization:
- Compares installed versions against latest manifests.
- Applies clean in-place updates for packages on the `stable` channel.
