# Execution Flow

The main flow lives in `packwire-rs/src/cli.rs`, `packwire-rs/src/task_queue.rs`, and `packwire-rs/src/installer.rs`:

1. The CLI or GUI receives a package identifier.
2. `manifest.rs` finds and validates the manifest.
3. `resolver.rs` obtains the official version and artifacts when required.
4. `task_queue.rs` runs the operation outside the UI thread.
5. `installer.rs` downloads, validates, extracts, and records the package.
6. `pather.rs` creates or updates shims.

State is written atomically to `%APPDATA%\packwire\installed.json` on Windows or the platform-equivalent data path.
