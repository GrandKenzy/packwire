# 02. Core Modules

> **Current Version:** `1.2.0`
> **Core:** `packwire-rs/src/`

The Core Modules section describes Packwire's internal contracts, dynamic version resolution engine, and deployment modes implemented in Rust.

---

## Section Overview

| Guide | Summary |
| :--- | :--- |
| **[01. Manifests & Models](./01-manifests-and-models.md)** | Specification of data contracts (`Manifest`, `InstallConfig`, `PackageState`), schema fields, and deterministic SHA-256 integrity hash calculation. |
| **[02. Version Resolution](./02-version-resolution.md)** | Live query engine interfacing with official upstream providers (python.org, nodejs.org, GitHub Releases) for `stable` vs `fixed` packages. |
| **[03. Installation Modes](./03-installation-modes.md)** | Mechanics of the three deployment strategies: headless extraction (`here`), official script runner (`command`), and assisted web portals (`site`). |

---

## Quick Summary

1. **Version Channels:** `stable` queries official endpoints for the latest production release; `fixed` pins exact immutable versions.
2. **Specialized modules:** Downloads, shims, uninstallation, and updates are separated into typed Rust modules.
3. **Integrity:** Every manifest calculates a canonical SHA-256 hash across normalized fields to prevent tampering.
