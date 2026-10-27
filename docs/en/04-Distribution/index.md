# 04. Packaging & Distribution

> **Current Version:** `1.0.0`
> **Formats:** Native binaries, portable ZIP archives, and Inno Setup (.exe)

This section covers all methods for deploying and packaging Packwire, from compiling standalone executables to automated installation scripts and system integration.

---

## Section Overview

| Guide | Summary |
| :--- | :--- |
| **[01. Full Installation Guide](./01-installation-guide.md)** | Step-by-step cross-platform guide for Windows, Linux, and macOS using native binaries. |
| **[02. System Integration](./02-system-integration.md)** | Technical reference for PATH registration, user shims, and Start Menu / Desktop shortcut generation. |
| **[04. Inno Setup Installer](./04-inno-setup-installer.md)** | Building the native Windows GUI installer (`Packwire-Setup.exe`) with LZMA2 compression. |

---

## Quick Summary

1. **End Users:** Use the native Windows installer `Packwire-Setup.exe` or standalone portable archives (zero Python requirement).
2. **Developers:** Build the binaries from `packwire-rs/Cargo.toml` with stable Rust.
3. **Dual Binaries:** `packwire.exe` for terminal CLI usage; `packwirew.exe` for background-console-free GUI launching.
