# Packwire Project Documentation

> **Version:** `1.2.0`
> **Implementation:** Native Rust (Windows, Linux, macOS)
> **License:** MIT  

Welcome to the official English documentation for **Packwire**, a modern, lightweight, and autonomous package manager and developer tool downloader for Windows, Linux, and macOS.

Packwire solves developer tool version fragmentation using unified semantic identifiers (`<name>@<channel_or_version>`), ensuring complete binary isolation through a centralized proxy shim architecture (`shims`) and an asynchronous background task queue.

---

## Documentation Index

### [01. System Architecture](./01-Architecture/index.md)
Architectural foundations, design principles, and engineering patterns governing Packwire's internal operation.
- **[01. Execution Flow](./01-Architecture/01-execution-flow.md)** — End-to-end lifecycle from manifest discovery to shim compilation and state persistence.
- **[02. Shims System & PATH](./01-Architecture/02-shims-system.md)** — Non-invasive proxy mechanism on Windows (`.cmd`/`.ps1`), Linux, and macOS (POSIX).
- **[03. Task Queue & Concurrency](./01-Architecture/03-task-queue.md)** — Non-blocking worker queue with daemon threads and timestamped event logs.

### [02. Core Modules](./02-Modules/index.md)
Data models specification, upstream resolution, deployment modes, and the native Rust module architecture.
- **[01. Manifests & Models](./02-Modules/01-manifests-and-models.md)** — Typed contracts, schema definitions, and canonical SHA-256 integrity calculation.
- **[02. Version Resolution](./02-Modules/02-version-resolution.md)** — Live querying of vendor APIs for dynamic `stable` vs immutable `fixed` channels.
- **[03. Installation Modes](./02-Modules/03-installation-modes.md)** — Operational behavior for `here`, `command`, and `site` modes.

### [03. User Interface](./03-Interface/index.md)
Modern SPA webview frontend accelerated by host web engines, IPC communication, and theming.
- **[01. GUI Architecture](./03-Interface/01-gui-architecture.md)** — Native rendering via WebView2 / WebKit without Electron memory overhead.
- **[02. IPC Bridge & Communication](./03-Interface/02-ipc-bridge.md)** — JSON RPC call specifications between JavaScript and the Rust backend with async job support.
- **[03. Themes & Styling](./03-Interface/03-themes-and-styling.md)** — Decoupled CSS stylesheets and visual theme persistence in `configs.json`.

### [04. Packaging & Distribution](./04-Distribution/index.md)
Deployment strategies for end-users and systems administrators across Windows, Linux, and macOS.
- **[01. Full Installation Guide](./04-Distribution/01-installation-guide.md)** — Comprehensive cross-platform guide for all deployment modalities.
- **[02. System Integration](./04-Distribution/02-system-integration.md)** — PATH registration, shortcut creation, and user proxy shims.
- **[04. Inno Setup Installer](./04-Distribution/04-inno-setup-installer.md)** — Compiling the native Windows installer wizard (`Packwire-Setup.exe`).

### [05. Reference & Diagnostics](./05-Reference/index.md)
Formal reference manuals for CLI commands, error troubleshooting, and Linux catalog.
- **[01. Command-Line Interface (CLI)](./05-Reference/01-cli.md)** — Exhaustive reference of commands, parameters, and flags.
- **[03. Diagnostics & Error Catalog](./05-Reference/03-diagnostics-catalog.md)** — Formal error codes, common causes, and step-by-step mitigations.
- **[04. Linux Catalog Guide](./05-Reference/04-linux-catalog.md)** — Recommended Linux distributions organized by machine profile and purpose.
- **[05. System Data Plan](./05-Reference/05-system-data-plan.md)** — Locally evaluated hardware diagnostics and zero-telemetry privacy guarantees.
