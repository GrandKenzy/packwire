# System Integration

The `packwire.installer_self` module manages local system integration, shortcut creation, and environment variable configuration.

---

## 1. What `packwire setup` Does

Running `packwire setup` performs the following automated steps:
1. Ensures all required directories exist (`%APPDATA%\packwire\shims`, `apps`, `cache`, `manifests`).
2. Generates root CLI launcher shims:
   - `packwire.cmd` and `packwire.ps1` on Windows.
   - Executable shell scripts on Linux and macOS.
3. Adds the shims directory to the User PATH in the Windows Registry (`HKCU\Environment`) and broadcasts `WM_SETTINGCHANGE`.
4. Creates Windows shell shortcuts (`.lnk`) using PowerShell `WScript.Shell`:
   - Start Menu: `Packwire`, `Packwire CLI (Console)`.
   - Desktop (optional with `--desktop`).
