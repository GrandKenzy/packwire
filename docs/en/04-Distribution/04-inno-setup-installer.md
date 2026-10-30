# Inno Setup Installer

Packwire uses [`installer.iss`](../../../installer.iss) to generate `Packwire-Setup.exe` from the Rust binaries and the resources under `assets/`.

## Installed contents

- `packwire.exe`: native CLI.
- `packwirew.exe`: windowed GUI without a console.
- `web/`: dashboard frontend.
- The catalog is synchronized from `assets/manifests/` on GitHub at startup and cached in `%APPDATA%\packwire\manifests` for offline use.
- `front.ico`: Packwire icon for shortcuts and the uninstaller.

The default destination is `{localappdata}\Programs\Packwire`, so administrator privileges are not required. The installer can add the application directory to the user PATH and creates Start Menu and optional Desktop shortcuts.

## Build

Build the binaries first:

```powershell
cargo build --release --manifest-path packwire-rs/Cargo.toml --bins
```

Then run Inno Setup 6:

```text
iscc installer.iss
```

The output is written to `dist/Packwire-Setup.exe`.
