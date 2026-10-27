# 03. User Interface

> **Current Version:** `1.0.0`
> **Engines:** Microsoft Edge WebView2 (Windows), WebKitGTK (Linux), Safari WebKit (macOS)

This section documents Packwire's modern webview-based graphical frontend, its bidirectional IPC bridge, the fallback local HTTP server, and the dynamic CSS theming system.

---

## Section Overview

| Guide | Summary |
| :--- | :--- |
| **[01. GUI Architecture](./01-gui-architecture.md)** | Native WebView2/WebKit integration through Rust, local HTTP server, window lifecycle, and windowed runner (`packwirew.exe`). |
| **[02. IPC Bridge & Communication](./02-ipc-bridge.md)** | JSON interface between JavaScript and Rust, state synchronization, and background job telemetry. |
| **[03. Themes & Styling](./03-themes-and-styling.md)** | Modular CSS styling system (`structure.css`, backgrounds, cards, buttons) with preference persistence in `configs.json`. |

---

## Quick Summary

1. **Zero Electron Overhead:** Uses the host operating system's native webview engine, significantly lowering memory consumption and startup latency.
2. **Console-free window:** Runs through the separate `packwirew.exe` binary.
3. **Dynamic Theming:** Seamless switching between Light and Dark themes without page reloads.
