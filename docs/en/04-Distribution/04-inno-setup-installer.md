# Inno Setup Installer

For Windows enterprise and desktop environments, Packwire provides an Inno Setup script ([`installer.iss`](file:///installer.iss)) to compile `Packwire-Setup.exe`.

---

## 1. Specifications

- **Target Location:** `{localappdata}\Programs\Packwire` (no administrator privileges required).
- **Compression:** `lzma2/ultra64` with solid compression.
- **Shortcuts:** Start Menu programs group and optional desktop icon.
- **Environment Updates:** Configures User PATH in registry and runs `packwire setup` silently on post-install.
- **Clean Uninstallation:** Invokes `packwire uninstall --all` on uninstallation to cleanly purge packages, shims, and cache.

---

## 2. Compiling

Using the Inno Setup 6 command-line compiler:
```cmd
iscc installer.iss
```
The resulting installer is saved to `dist\Packwire-Setup.exe`.
