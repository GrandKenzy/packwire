# 04. Packaging & Distribution

> **Current Version:** `0.1.4`
> **Formats:** Standalone PE (.exe), Python Wheel (.whl), Inno Setup (.exe), Installation Scripts (.bat, .ps1, .sh)

This section covers all methods for deploying and packaging Packwire, from compiling standalone executables to automated installation scripts and system integration.

---

## Section Overview

| Guide | Summary |
| :--- | :--- |
| **[01. Full Installation Guide](./01-installation-guide.md)** | Step-by-step cross-platform guide for Windows, Linux, and macOS: precompiled releases, automated scripts, and pip. |
| **[02. System Integration](./02-system-integration.md)** | Technical reference for PATH registration, user shims, and Start Menu / Desktop shortcut generation. |
| **[03. PyInstaller Packager](./03-pyinstaller-packager.md)** | Compiling standalone `packwire.exe` and windowed `packwirew.exe` using `packwire.spec`. |
| **[04. Inno Setup Installer](./04-inno-setup-installer.md)** | Building the native Windows GUI installer (`Packwire-Setup.exe`) with LZMA2 compression. |

---

## Quick Summary

1. **End Users:** Use the native Windows installer `Packwire-Setup.exe` or standalone portable archives (zero Python requirement).
2. **Python Developers:** Install via `pip install packwire` or clone the repository and run `install.bat`, `install.ps1`, or `install.sh`.
3. **Dual Binaries:** `packwire.exe` for terminal CLI usage; `packwirew.exe` for background-console-free GUI launching.
