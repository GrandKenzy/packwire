# GUI and Backend Communication

Packwire's bridge is implemented in `packwire-rs/src/gui.rs`. The web frontend and the Rust core exchange JSON objects through local HTTP endpoints; the native GUI loads the same server inside `packwirew.exe`.

## Main operations

| Operation | Purpose |
|---|---|
| `get_available` | Lists catalog manifests. |
| `get_installed` | Reads installed-package state. |
| `install` | Queues an installation. |
| `uninstall` | Removes a managed package. |
| `get_jobs` / `get_job` | Reads task progress. |
| `get_config` / `set_config` | Reads or persists configuration. |
| `get_system_info` | Returns local system diagnostics. |
| `list_system_apps` | Lists applications detected by Windows. |

Responses are serialized with `serde_json`. Network, extraction, and installation work runs outside the render thread through `task_queue.rs`, keeping the window responsive.
