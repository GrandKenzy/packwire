# Manifests and Data Models

Packwire's typed contracts are implemented in `packwire-rs/src/models.rs`. JSON loading and tolerant manifest parsing are implemented in `packwire-rs/src/manifest.rs`.

The models cover `Manifest`, `InstallConfig`, `PackageState`, and installed-package metadata. A canonical SHA-256 hash detects manifest changes while preserving compatibility with existing persistent state.

Catalog files are distributed in `assets/manifests/`.
