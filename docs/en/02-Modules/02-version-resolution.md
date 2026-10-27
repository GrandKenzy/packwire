# Version Resolution Engine

The `packwire.core.resolver` module handles dynamic query and discovery of official releases for packages on the `stable` channel.

---

## 1. Resolvers by Ecosystem

### Python Resolver (`PythonResolver`)
- Queries the official Python Software Foundation API: `https://www.python.org/api/v2/downloads/release/?is_published=true`.
- Identifies the latest non-prerelease Python 3 release.
- Inspects release assets to locate the official Windows 64-bit embeddable package (`python-X.Y.Z-embed-amd64.zip`).

### Node.js Resolver (`NodeResolver`)
- Queries the official Node.js distribution index: `https://nodejs.org/dist/index.json`.
- Filters for active LTS releases containing `win-x64-zip` packages.
- Resolves download URL and official release notes page.

### GitHub Releases Resolver (`GitHubReleaseResolver`)
Generic engine querying official GitHub repositories via REST API:
- `W64DevKitResolver`: Resolves releases from `skeeto/w64devkit` (GCC, G++, Make, GDB for Windows).
- `LLVMResolver`: Resolves official LLVM/Clang Windows toolchains from `llvm/llvm-project`.
- `RufusResolver`: Resolves portable bootable USB tools from `pbatard/rufus`.
- `VentoyResolver`: Resolves multiboot utility archives from `ventoy/Ventoy`.
- `PowerShellResolver`: Resolves latest stable PowerShell 7 from `PowerShell/PowerShell`.
