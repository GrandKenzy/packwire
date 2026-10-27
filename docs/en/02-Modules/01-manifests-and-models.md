# Manifests & Models

Packwire uses typed Python dataclasses to model package manifests, installation configurations, and system state.

---

## 1. Core Data Models (`packwire.core.models`)

### `Manifest`
Top-level definition of a package in the catalog:
- `id` (str): Unique semantic identifier (e.g., `python@stable`, `git`, `nodejs@24.21.0`).
- `name` (str): Display name of the software.
- `type` (str): Versioning channel (`stable` or `fixed`).
- `version` (Optional[str]): Exact version string or resolved latest version.
- `category` (str): Logical group (`programming languages`, `compilers`, `system tools`, etc.).
- `description` (str): Brief summary in Spanish or English.
- `aliases` (List[str]): Alternative lookup identifiers.
- `install` (`InstallConfig`): Deployment settings across modes and OS platforms.
- `manifest_hash` (str): Deterministic SHA-256 integrity hash.

### `InstallConfig`
Defines operational behavior during deployment:
- `mode` (str): Default mode (`here`, `command`, `site`).
- `clean` (bool): Whether to purge temporary download archives from cache after extraction.
- `here` (`HereConfig`): Direct download URL, archive format, binaries, and checksum.
- `command` (`CommandConfig`): Native scripts for Windows, Linux, and macOS.
- `site` (`SiteConfig`): Official download website URL.
- `os` (Dict[str, Any]): Platform-specific overrides keyed by operating system name.

### `PackageState`
Tracks installed software in `installed.json`:
- `id`: Installed package identifier.
- `version`: Installed version string.
- `install_path`: Local filesystem destination path.
- `binaries`: Executable filenames.
- `shims`: Generated proxy wrapper filenames.
- `installed_at`: ISO timestamp of installation.

---

## 2. Canonical Hash Integrity

To ensure determinism and detect tampering, `Manifest.calculate_hash()` serializes canonical fields into sorted JSON and calculates a SHA-256 digest:

```python
canonical_data = {
    "name": self.name.lower(),
    "type": self.type,
    "version": self.version,
    "install": {
        "mode": self.install.mode,
        "clean": self.install.clean,
        "here_url": self.install.here.url if self.install.here else None,
        "here_hash": self.install.here.expected_hash if self.install.here else None,
        "site_url": self.install.site.url if self.install.site else None,
        "command_script": self.install.command.script if self.install.command else None,
        "binaries": self.install.here.binaries if self.install.here else [],
        "addpath": self.install.here.addpath if self.install.here else False,
    }
}
```
