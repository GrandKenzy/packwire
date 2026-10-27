# 01. System Architecture

> **Current Version:** `0.1.4`
> **Platforms:** Windows (Win32), Linux (POSIX), macOS (Darwin)

This section outlines the architectural foundations, design principles, and engineering patterns governing Packwire's internal operation. The system is designed around strict modularity, deterministic execution environments, and idempotent operations on the host filesystem without requiring superuser privileges for everyday package management.

---

## Section Overview

| Guide | Summary |
| :--- | :--- |
| **[01. Execution Flow](./01-execution-flow.md)** | End-to-end lifecycle trace of a package from manifest resolution to shim generation and state registration. |
| **[02. Shims System & PATH](./02-shims-system.md)** | Lightweight proxy mechanics (`.cmd`/`.ps1` on Windows, POSIX shell scripts on Linux/macOS), registry management in `HKCU\Environment`, and setting broadcast. |
| **[03. Task Queue & Concurrency](./03-task-queue.md)** | Background worker architecture (`TaskQueue`), thread isolation, timestamped log buffering, and non-blocking event dispatch. |

---

## Quick Summary

1. **Centralized Proxy Shims:** Rather than polluting system PATH variables with dozens of directories, Packwire keeps a single user-level proxy shim folder (`%APPDATA%\packwire\shims` or `~/.local/share/packwire/shims`).
2. **Deterministic Versioning:** Packages are tracked using canonical identifiers (`<name>@<channel_or_version>`), ensuring reproducible installations without binary collisions.
3. **Cross-Platform:** Native support for Windows (Registry/UAC), Linux (`~/.bashrc`, XDG `.desktop`), and macOS (`~/.zshrc`, Safari WebKit, `osascript`).
