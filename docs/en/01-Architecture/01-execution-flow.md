# Execution Flow

The main flow lives in `packwire-rs/src/cli.rs`, `packwire-rs/src/task_queue.rs`, `packwire-rs/src/installer.rs`, and `packwire-rs/src/manifest.rs`:

1. At startup, `sync_remote_catalog` checks the `assets/manifests/` folder in Packwire's GitHub repository.
2. New or modified manifests are validated and saved in the local cache; when offline, the previous cache is preserved.
3. The CLI or GUI receives a package identifier.
4. `manifest.rs` finds and validates the manifest.
5. `resolver.rs` obtains the official version and artifacts when required.
6. `task_queue.rs` runs the operation outside the UI thread.
7. `installer.rs` downloads, validates, extracts, and records the package.
8. `pather.rs` creates or updates shims.

State is written atomically to `%APPDATA%\packwire\installed.json`, and the synchronized catalog is stored in `%APPDATA%\packwire\manifests` on Windows or the platform-equivalent data paths.
