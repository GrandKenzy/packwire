<p align="center">
  <img src="discentra_central/badge/banner.jpg" alt="Packwire" width="100%">
</p>

# Packwire

Gestor nativo de paquetes para Windows, escrito en Rust. Packwire permite descubrir, instalar y administrar herramientas, lenguajes de programación, runtimes y sistemas operativos desde una CLI y una interfaz gráfica.

## Descargar

La versión estable actual es **1.0.0**.

- [Instalador para Windows](https://github.com/GrandKenzy/packwire/releases/latest)
- [Versiones y archivos disponibles](https://github.com/GrandKenzy/packwire/releases)

El instalador incluye la CLI (`packwire.exe`), la GUI (`packwirew.exe`), el catálogo de manifests y los recursos web. No requiere Python.

## Uso básico

Desde una terminal:

```powershell
# Abrir la interfaz gráfica
packwirew.exe

# Consultar el catálogo
packwire available

# Instalar un paquete
packwire install git

# Ver aplicaciones instaladas
packwire list

# Consultar un manifest
packwire info python
```

La GUI permite explorar el catálogo, instalar paquetes, consultar versiones y revisar las aplicaciones detectadas en el sistema.

## Desarrollo

Requisitos: Rust estable y Windows con WebView2 para la interfaz gráfica.

```powershell
cargo test --manifest-path packwire-rs/Cargo.toml --lib
cargo build --release --manifest-path packwire-rs/Cargo.toml --bins
```

El instalador se genera con Inno Setup usando `installer.iss`. El catálogo se encuentra en `assets/manifests/` y los recursos de la GUI en `assets/web/`.

## Estructura

```text
packwire/
├── assets/                 # Catálogo y recursos de la GUI
├── discentra_central/      # Metadatos e imágenes para Central
├── docs/                   # Documentación en español e inglés
├── packwire-rs/            # CLI, GUI y núcleo nativos en Rust
└── installer.iss           # Instalador de Windows
```

## Estado

Packwire 1.0.0 es la primera versión estable. La implementación antigua en Python y su cadena de empaquetado fueron eliminadas del proyecto.

<p align="center">
  <img src="discentra_central/badge/footer.png" alt="Packwire" width="100%">
</p>
