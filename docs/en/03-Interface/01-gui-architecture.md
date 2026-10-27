# GUI Architecture

Packwire's graphical frontend is built as a lightweight Single-Page Application (SPA) using HTML5, CSS3, and modern Vanilla JavaScript, embedded via `pywebview`.

---

## 1. Window Lifecycle & Launch Modes

### Desktop Window Mode (`packwire ui`)
- Creates a dedicated native window using Edge WebView2 (Windows), WebKitGTK (Linux), or Safari WebKit (macOS).
- On Windows, launching via `packwirew.exe` completely suppresses the background terminal console window.
- Injects a JavaScript API bridge object (`window.pywebview.api`) that exposes Python methods directly to frontend scripts.

### Fallback Web Server Mode (`packwire ui --web`)
- Starts a local HTTP daemon on `http://localhost:5050` using Python's standard `http.server` or `bottle`.
- Automatically opens the default web browser.
- Exposes REST endpoints under `/api/<method_name>` that mirror the native IPC bridge.

---

## 2. Frontend Structure (`packwire/core/gui/web/`)

- `index.html`: Main SPA layout containing Catalog, Installed Packages, and System Diagnostics tabs.
- `app.js`: Application controller, search filters, modal management, and live terminal streaming.
- `structure.css`: Shared layout, navigation bar, typography, and modal dialog rules.
- `global/`: Modular theme styles (`default_*` for light mode, `dark_*` for dark mode).
