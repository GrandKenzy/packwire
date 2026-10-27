# PyInstaller Packager

Packwire compiles into standalone single-file executables using PyInstaller and [`packwire.spec`](file:///packwire.spec).

---

## 1. Dual Executable Architecture

To provide both a rich terminal CLI and a console-free GUI on Windows, the build specification outputs two separate executables:
1. **`packwire.exe` (`console=True`):** Intended for command-line usage. Emits stdout/stderr directly into the invoking terminal.
2. **`packwirew.exe` (`console=False`):** Intended for the graphical interface. Suppresses the black background console window on launch.

---

## 2. Building Executables

Build via CLI:
```bash
packwire build --exe
```
Or directly using Python:
```bash
python scripts/build_exe.py
```
Compiled outputs are placed in the `dist/` directory.
