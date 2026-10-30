# Migración de Packwire a Rust nativo

La migración se completó. Packwire ya no contiene una implementación Python ni depende de un runtime Python para compilar, distribuir o ejecutar la aplicación.

## Arquitectura actual

| Responsabilidad | Implementación |
|---|---|
| Configuración y rutas | `packwire-rs/src/config.rs` |
| Modelos y manifiestos | `packwire-rs/src/models.rs`, `packwire-rs/src/manifest.rs` |
| Estado persistente | `packwire-rs/src/state.rs` |
| Descargas y resolución | `packwire-rs/src/downloader.rs`, `packwire-rs/src/resolver.rs` |
| Instalación y desinstalación | `packwire-rs/src/installer.rs` |
| Shims y PATH | `packwire-rs/src/pather.rs` |
| Cola de tareas | `packwire-rs/src/task_queue.rs` |
| CLI | `packwire-rs/src/cli.rs` |
| GUI, servidor local e IPC | `packwire-rs/src/gui.rs` |
| Aplicaciones del sistema | `packwire-rs/src/system_apps.rs` |
| Recursos web | `assets/web/` |
| Fuente del catálogo de manifests | `assets/manifests/` en GitHub |
| Caché local del catálogo | `%APPDATA%\packwire\manifests` en Windows |

## Distribución

- `packwire.exe`: CLI nativa.
- `packwirew.exe`: GUI nativa sin consola.
- `installer.iss`: instalador Windows con Inno Setup.
- `.github/workflows/release.yml`: builds nativos para Windows, Linux y macOS.

Los binarios se compilan con Cargo y la interfaz se copia desde `assets/web/`. La carpeta `assets/manifests/` permanece en el repositorio como fuente de catálogo, pero ya no se incluye en el instalador ni en el paquete portable: Packwire la sincroniza al abrirse y mantiene una caché local compatible con instalaciones existentes.

## Verificación

```powershell
cargo fmt --manifest-path packwire-rs/Cargo.toml --check
cargo test --manifest-path packwire-rs/Cargo.toml
cargo build --release --manifest-path packwire-rs/Cargo.toml --bins
```

La carpeta `packwire/`, los metadatos Python, PyInstaller, los scripts `install.*` y el empaquetado Wheel fueron eliminados deliberadamente para evitar mantener dos implementaciones.
