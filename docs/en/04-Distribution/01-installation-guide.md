# Full Installation Guide

Packwire can be installed on Windows, Linux, and macOS using precompiled binaries, automated repository scripts, or standard Python packaging.

---

## 1. Option A: Precompiled Standalone Binaries (Recommended)

No Python runtime installation is required:
1. Navigate to the official releases page: `https://github.com/GrandKenzy/packwire/releases/latest`.
2. **Windows:** Download `Packwire-Setup.exe` (full wizard) or `packwire.exe` (portable).
3. **Linux:** Download `packwire-linux-x64.tar.gz`, extract, and move the binary to `~/.local/bin/`.
4. **macOS:** Download `packwire-macos-universal.tar.gz`, extract, and move to `/usr/local/bin/`.

---

## 2. Option B: Automated Scripts (From Repository)

### On Windows
Run `install.bat` via double-click in File Explorer, or via PowerShell:
```powershell
.\install.ps1 -DesktopShortcut
```

### On Linux & macOS
Run the POSIX installer script:
```bash
chmod +x install.sh
./install.sh --desktop
source ~/.bashrc  # or source ~/.zshrc on macOS
```

---

## 3. Option C: Python Package (`pip`)

```bash
pip install packwire
packwire setup --desktop
```
