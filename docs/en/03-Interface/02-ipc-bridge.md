# IPC Bridge & Communication

The `packwire.core.gui.bridge.GuiBridge` class acts as the bridge connecting the webview frontend to the core package manager.

---

## 1. API Methods

| Method | Parameters | Returns | Description |
| :--- | :--- | :--- | :--- |
| `get_system_info` | None | `dict` | OS name, architecture, PATH status, and hardware diagnostics. |
| `list_available` | None | `list[dict]` | All manifests available in the catalog with resolved versions. |
| `list_installed` | None | `dict` | Dictionary of currently installed packages and state. |
| `queue_install` | `id, mode, version, force` | `dict` | Submits an install task to the background `TaskQueue` and returns `job_id`. |
| `get_job` | `job_id` | `dict` | Retrieves live status, percentage, and log messages for a task. |
| `uninstall` | `id` | `dict` | Uninstalls a package synchronously. |
| `uninstall_all` | `remove_path` | `dict` | Completely wipes all packages, shims, cache, and PATH registration. |
| `reinstall` | `id` | `dict` | Performs a clean reinstallation of an existing package. |
| `check_updates` | None | `dict` | Checks for available upstream updates for installed packages. |
| `update` | `id` (optional) | `list[dict]` | Updates specified or all packages on the `stable` channel. |
| `add_path` | None | `dict` | Adds the shims directory to user PATH environment variables. |
| `add_defender_exclusion` | None | `dict` | Prompts UAC elevation to configure Windows Defender exclusions for GCC. |
| `open_guide` | `guide_name` | `dict` | Opens a local HTML guide (such as the Windows 11 GCC setup guide). |
