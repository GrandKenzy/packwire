# Manifests and Data Models

Packwire's typed contracts are implemented in `packwire-rs/src/models.rs`. JSON loading and tolerant manifest parsing are implemented in `packwire-rs/src/manifest.rs`.

The models cover `Manifest`, `InstallConfig`, `PackageState`, and installed-package metadata. A canonical SHA-256 hash detects manifest changes while preserving compatibility with existing persistent state.

Catalog source files live in `assets/manifests/` inside the Packwire repository. Distributed Packwire builds synchronize that folder from GitHub at startup and store the validated copy in `%APPDATA%\packwire\manifests`. The local cache keeps the catalog usable offline.
