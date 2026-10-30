# Full Installation Guide

Packwire is a native Rust application for Windows, Linux, and macOS. End users should use the official release artifacts; developers can build the binaries with Cargo.

## Official releases

Download the latest release from [GitHub Releases](https://github.com/GrandKenzy/packwire/releases/latest):

- Windows: `Packwire-Setup.exe` or `packwire-windows-x64.zip`.
- Linux: `packwire-linux-x64.tar.gz`.
- macOS: `packwire-macos-universal.tar.gz`.

The Windows installer creates the shortcuts, configures the optional PATH entry, and registers the uninstaller.

## Build from source

Install stable Rust and run:

```powershell
cargo test --manifest-path packwire-rs/Cargo.toml
cargo build --release --manifest-path packwire-rs/Cargo.toml --bins
```

For a local build, the source catalog is read from `assets/manifests/` and the dashboard from `assets/web/`. Distributed builds synchronize the catalog from GitHub at startup and cache it in `%APPDATA%\packwire\manifests`; the dashboard is bundled with the release.

## Verify the installation

```text
packwire --help
packwire available
packwire path
packwirew.exe
```

To install a package:

```text
packwire install python@stable --mode here
```

To remove a managed package:

```text
packwire list
```

Use the Applications section in the GUI for managed-package removal.
