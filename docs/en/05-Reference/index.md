# 05. Reference & Diagnostics

> **Current Version:** `1.0.0`
> **Compatibility:** Native Rust and the system WebView2/WebKit runtime

Comprehensive technical reference for the Command-Line Interface (CLI), diagnostics and error catalog, Linux distribution guide, and local hardware telemetry privacy plan.

---

## Section Overview

| Guide | Summary |
| :--- | :--- |
| **[01. Command-Line Interface (CLI)](./01-cli.md)** | Complete manual of commands, subcommands, positional arguments, and flags for `packwire`. |
| **[03. Diagnostics & Error Catalog](./03-diagnostics-catalog.md)** | Standard error codes, common causes, and recommended mitigation steps. |
| **[04. Linux Catalog Guide](./04-linux-catalog.md)** | Curated official Linux distribution catalog organized by hardware profile and use case. |
| **[05. System Data Plan](./05-system-data-plan.md)** | Breakdown of locally evaluated hardware diagnostics (CPU, GPU, RAM, UEFI) and privacy safeguards. |

---

## Quick Summary

1. **Comprehensive CLI:** Commands for installation (`install`), removal (`remove`), reinstall (`reinstall`), updates (`update`), catalog browsing (`available`, `list`, `info`), PATH check (`path`), Defender exclusions (`fix-defender`), and GUI launching (`ui`).
2. **Automation:** All capabilities are available through the native CLI with stable exit codes.
3. **Privacy First:** Hardware telemetry collects no personal identifiers, credentials, or usernames; diagnostics are evaluated strictly on the local machine.
