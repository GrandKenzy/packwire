# 02. Core Modules

> **Current Version:** `0.1.4`
> **Package:** `packwire.core`

The Core Modules section describes Packwire's internal contracts, dynamic version resolution engine, deployment modes, and processing pipeline based on the Visitor pattern.

---

## Section Overview

| Guide | Summary |
| :--- | :--- |
| **[01. Manifests & Models](./01-manifests-and-models.md)** | Specification of data contracts (`Manifest`, `InstallConfig`, `PackageState`), schema fields, and deterministic SHA-256 integrity hash calculation. |
| **[02. Version Resolution](./02-version-resolution.md)** | Live query engine interfacing with official upstream providers (python.org, nodejs.org, GitHub Releases) for `stable` vs `fixed` packages. |
| **[03. Installation Modes](./03-installation-modes.md)** | Mechanics of the three deployment strategies: headless extraction (`here`), official script runner (`command`), and assisted web portals (`site`). |
| **[04. Visitors Pipeline](./04-visitors-pipeline.md)** | Architecture and contracts for `DownloaderVisitor`, `PatherVisitor`, `UninstallerVisitor`, and `UpdaterVisitor`. |

---

## Quick Summary

1. **Version Channels:** `stable` queries official endpoints for the latest production release; `fixed` pins exact immutable versions.
2. **Visitor Pattern:** Decouples operational algorithms (download, shim creation, uninstallation, updates) from data models.
3. **Integrity:** Every manifest calculates a canonical SHA-256 hash across normalized fields to prevent tampering.
