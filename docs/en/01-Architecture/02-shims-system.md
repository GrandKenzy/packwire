# Shims System & PATH

Packwire avoids system environment bloat by channeling all installed executables through a centralized proxy directory (`shims`).

---

## 1. Why Shims?

Traditional installers append arbitrary directories to the machine or user `PATH`, resulting in:
- Bloated environment strings that can exceed system limits.
- Conflicting binaries when multiple versions of the same tool are installed (e.g., Python 3.12 vs 3.13).
- Lingering paths after manual uninstallation.

Packwire solves this by adding **only one directory** to the user PATH:
- **Windows:** `%APPDATA%\packwire\shims` (or user-defined `PACKWIRE_ROOT\shims`).
- **Linux / macOS:** `~/.local/share/packwire/shims`.

---

## 2. Shim File Formats

For every binary declared by an installed package, Packwire creates platform-specific proxy wrappers:

### On Windows
Two proxy files are created for maximum terminal interoperability:
1. **`.cmd` shim (for CMD & classic shells):**
   ```cmd
   @echo off
   "C:\Users\...\AppData\Roaming\packwire\apps\pkg\bin\target.exe" %*
   ```
2. **`.ps1` shim (for PowerShell 5/7):**
   ```powershell
   & "C:\Users\...\AppData\Roaming\packwire\apps\pkg\bin\target.exe" $args
   ```

### On Linux & macOS (POSIX)
Executable shell scripts with `chmod +x` permissions:
```sh
#!/bin/sh
exec "/home/.../.local/share/packwire/apps/pkg/bin/target" "$@"
```

---

## 3. Persistent PATH Integration

### Windows
- Written to `HKCU\Environment\Path` via Python's native `winreg` module.
- Broadcasts `WM_SETTINGCHANGE` to notify active desktop processes and Explorer.
- Fallback synchronization via PowerShell `[System.Environment]::SetEnvironmentVariable`.

### Linux / macOS
- Appends `export PATH="<shims>:$PATH"` to `~/.bashrc`, `~/.zshrc`, or `~/.profile`.
- In-process `os.environ["PATH"]` updated immediately for active sessions.
