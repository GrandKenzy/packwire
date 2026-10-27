# Shims and PATH

The shim system is implemented in `packwire-rs/src/pather.rs`. Managed packages can expose `.cmd` and `.ps1` launchers on Windows or POSIX scripts on Linux and macOS.

Shims are stored in `%APPDATA%\packwire\shims` on Windows and `~/.local/share/packwire/shims` on POSIX systems. `packwire path` checks the current state and `packwire path --add` registers the directory for the user.

On Windows, the backend uses the user registry and broadcasts `WM_SETTINGCHANGE` so new terminals receive the update.
