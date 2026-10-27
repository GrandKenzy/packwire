# GUI Architecture

Packwire's GUI is implemented in Rust in `packwire-rs/src/gui.rs`. The frontend is a HTML/CSS/JavaScript SPA stored in `assets/web/` and rendered through `wry`/WebView2 on Windows or the system WebKit engine on Linux and macOS.

## Executables

- `packwire.exe`: console CLI.
- `packwirew.exe`: windowed GUI without a console.

The backend starts a local HTTP server on `127.0.0.1:5050`, serves `assets/web/`, and exposes Packwire operations as JSON endpoints. The native GUI loads the same address inside the system WebView.

If the default port is occupied, the backend selects another available port. `PACKWIRE_WEB_DIR` can point to an alternative web-resource directory during development or packaging.

The application does not use Electron, Python, PyInstaller, or an external web server.
