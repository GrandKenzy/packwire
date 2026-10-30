# System Integration

Packwire is distributed as a native Rust application. System integration is handled by the Inno Setup installer and the native PATH/shim implementation; Python and helper installation scripts are not required.

## Windows

`Packwire-Setup.exe` installs:

- `packwire.exe`, the native CLI.
- `packwirew.exe`, the windowed GUI.
- `web/`, the dashboard resources.
- the catalog synchronized from GitHub into `%APPDATA%\packwire\manifests`.
- Start Menu and optional Desktop shortcuts using the Packwire icon.
- The application directory in the user PATH when selected.

The default destination is `%LOCALAPPDATA%\Programs\Packwire`, so standard installation does not require administrator privileges.

## PATH and shims

Managed-package shims are stored at:

```text
%APPDATA%\packwire\shims       # Windows
~/.local/share/packwire/shims   # Linux and macOS
```

Use the native CLI to inspect or add the shim directory:

```text
packwire path
packwire path --add
```

On Windows, user PATH updates use `HKCU\Environment` and broadcast `WM_SETTINGCHANGE`.

## Local builds

```powershell
cargo build --release --manifest-path packwire-rs/Cargo.toml --bins
```

Compile `installer.iss` with Inno Setup after building the binaries.
