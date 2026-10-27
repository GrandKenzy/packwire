# Guía Completa de Instalación

Packwire es una aplicación nativa Rust para Windows, Linux y macOS. Los usuarios finales deben utilizar los artefactos oficiales; los desarrolladores pueden compilar los binarios con Cargo.

## Releases oficiales

Descarga la versión más reciente desde [GitHub Releases](https://github.com/GrandKenzy/packwire/releases/latest):

- Windows: `Packwire-Setup.exe` o `packwire-windows-x64.zip`.
- Linux: `packwire-linux-x64.tar.gz`.
- macOS: `packwire-macos-universal.tar.gz`.

El instalador de Windows crea los accesos directos, configura el PATH opcional y registra el desinstalador.

## Compilar desde el código fuente

Instala Rust estable y ejecuta:

```powershell
cargo test --manifest-path packwire-rs/Cargo.toml
cargo build --release --manifest-path packwire-rs/Cargo.toml --bins
```

El catálogo y el dashboard se leen desde `assets/manifests/` y `assets/web/`.

## Verificar la instalación

```text
packwire --help
packwire available
packwire path
packwirew.exe
```

Para instalar un paquete:

```text
packwire install python@stable --mode here
```

La eliminación de paquetes gestionados está disponible desde la sección Aplicaciones de la GUI.
