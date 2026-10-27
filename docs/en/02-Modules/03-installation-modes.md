# Installation Modes

Packwire provides three distinct deployment modes tailored to different software distribution formats.

---

## 1. Mode `here` (Automated Headless Deployment)

The `here` mode is ideal for developer tools, runtimes, and command-line utilities distributed as ZIP, TAR, 7z-SFX, or portable executables.
- **Download:** Atomic transfer to `cache/` with `.part` buffering.
- **Hash Validation:** Verifies SHA-256 against `expected_hash` when available.
- **Safe Extraction:** Validates archive entry paths against directory traversal attacks (Zip-Slip).
- **Embedded Python Patching:** For Python embeddable distributions, automatically uncomments `import site` in `._pth` files to enable `pip` and third-party libraries.
- **Shim Generation:** Immediately creates proxy wrappers in `shims/`.

---

## 2. Mode `command` (Official Script Runner)

The `command` mode orchestrates native package managers (`winget`, PowerShell, `apt`, `pacman`, `brew`):
- Executes official vendor scripts with platform detection (`windows`, `linux`, `darwin`).
- **Privilege Elevation:** Automatically prompts for administrative elevation (UAC in Windows, `pkexec`/`sudo` on Linux, `osascript` on macOS) when `elevated: true` is specified.
- Captures output logs into timestamped temporary transcript files.

---

## 3. Mode `site` (Assisted Official Web Portal)

The `site` mode is designed for large desktop applications requiring manual interactive configuration (e.g., Visual Studio Code, Docker Desktop):
- Opens the user's default browser directly to the official download page.
- Records the package state in `installed.json` under mode `site` for tracking.
