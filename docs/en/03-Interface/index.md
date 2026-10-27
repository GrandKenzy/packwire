# 03. User Interface

> **Current Version:** `0.1.4`
> **Engines:** Microsoft Edge WebView2 (Windows), WebKitGTK (Linux), Safari WebKit (macOS)

This section documents Packwire's modern webview-based graphical frontend, its bidirectional IPC bridge, the fallback local HTTP server, and the dynamic CSS theming system.

---

## Section Overview

| Guide | Summary |
| :--- | :--- |
| **[01. GUI Architecture](./01-gui-architecture.md)** | Native pywebview integration, fallback HTTP server, window lifecycle, and windowed headless runner (`packwirew.exe`). |
| **[02. IPC Bridge & Communication](./02-ipc-bridge.md)** | JSON RPC specification between JavaScript and Python (`GuiBridge`), state synchronization, and background job telemetry. |
| **[03. Themes & Styling](./03-themes-and-styling.md)** | Modular CSS styling system (`structure.css`, backgrounds, cards, buttons) with preference persistence in `configs.json`. |

---

## Quick Summary

1. **Zero Electron Overhead:** Uses the host operating system's native webview engine, significantly lowering memory consumption and startup latency.
2. **Dual Mode:** Runs either as a native desktop window (`packwire ui`) or as a local browser-accessible web service (`packwire ui --web`).
3. **Dynamic Theming:** Seamless switching between Light and Dark themes without page reloads.
