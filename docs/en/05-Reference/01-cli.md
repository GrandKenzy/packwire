# Command-Line Interface (CLI)

The `packwire` CLI provides a unified interface for package management, environment configuration, and packaging.

---

## 1. Syntax

```bash
packwire [command] [options]
```

---

## 2. Commands Summary

### `install`
Installs a package by name or identifier:
```bash
packwire install <package> [--version <ver>] [--channel {stable,fixed}] [--mode {here,command,site}] [--force]
```
Examples:
```bash
packwire install python
packwire install python@3.14
packwire install c-gcc --mode here
packwire install vscode --mode site
```

### `remove` / `uninstall`
Uninstalls a package or wipes the entire Packwire environment:
```bash
packwire remove <package>
packwire remove --all [--keep-path]
```

### `reinstall`
Reinstalls an already installed package cleanly:
```bash
packwire reinstall <package>
```

### `update`
Checks and updates packages on the `stable` channel:
```bash
packwire update [package]
```

### `list`
Lists currently installed packages, versions, and filesystem paths:
```bash
packwire list
```

### `available`
Lists all available packages and compilers in the catalog:
```bash
packwire available
```

### `info`
Displays raw JSON manifest metadata for a package:
```bash
packwire info <package>
```

### `path`
Checks or updates the shims directory in user PATH:
```bash
packwire path
packwire path --add
```

### `fix-defender` / `defender`
Configures Windows Defender exclusion for Packwire and `as.exe` / `gcc.exe` to prevent false positive deletions:
```bash
packwire fix-defender [--path <custom_path>]
```

### `ui` / `gui`
Launches the modern graphical interface:
```bash
packwire ui [--web] [--port 5050]
```

### `setup` / `self-install`
Integrates Packwire into the system environment:
```bash
packwire setup [--desktop] [--no-start-menu] [--no-path]
```

### `build` / `pack`
Packages Packwire into standalone executables or Python wheels:
```bash
packwire build [--exe] [--wheel] [--all]
```
