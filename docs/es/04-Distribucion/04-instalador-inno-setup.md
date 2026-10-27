# Instalador Inno Setup

Packwire usa [`installer.iss`](../../../installer.iss) para generar `Packwire-Setup.exe` a partir de los binarios Rust y los recursos de `assets/`.

## Contenido instalado

- `packwire.exe`: CLI nativa.
- `packwirew.exe`: GUI nativa sin consola.
- `web/`: frontend del dashboard.
- `manifests/`: catálogo de paquetes.
- `front.ico`: icono de Packwire para accesos directos y desinstalador.

El destino predeterminado es `{localappdata}\Programs\Packwire`, sin requerir permisos administrativos. El instalador puede añadir la carpeta de la aplicación al PATH del usuario y crea accesos directos del Menú Inicio y del Escritorio.

## Compilación

Primero compila los binarios:

```powershell
cargo build --release --manifest-path packwire-rs/Cargo.toml --bins
```

Después ejecuta Inno Setup 6:

```text
iscc installer.iss
```

El resultado se genera en `dist/Packwire-Setup.exe`.
