# Ficha técnica

## Descripción

Packwire es un gestor de aplicaciones, herramientas y entornos de desarrollo para Windows. Permite consultar un catálogo de manifests, resolver versiones, descargar paquetes e instalar aplicaciones desde una interfaz gráfica o desde la consola.

## Versión

- Versión estable: `1.0.0`
- Plataforma distribuida: Windows x64
- Arquitectura: aplicación nativa escrita en Rust
- Interfaz: GUI nativa y CLI

## Requisitos

- Windows 10 u 11 de 64 bits.
- Microsoft Edge WebView2 Runtime para la interfaz gráfica.
- Conexión a Internet para consultar versiones y descargar paquetes.
- Permisos de administrador únicamente cuando el instalador del paquete o la modificación solicitada lo requiera.

## Instalación

1. Descarga `Packwire-Setup.exe` desde la release estable de GitHub.
2. Ejecuta el instalador y sigue las instrucciones.
3. Abre Packwire desde el acceso directo o ejecuta `packwirew.exe`.

También existe una versión portable en `packwire-windows-x64.zip`. Descomprímela y ejecuta `packwirew.exe` para abrir la interfaz gráfica o `packwire.exe` para utilizar la consola.

## Funciones principales

- Catálogo de paquetes organizado por categorías.
- Instalación de lenguajes de programación, runtimes, herramientas y utilidades.
- Resolución de versiones estables y versiones específicas.
- Instalación desde paquetes portables, comandos oficiales o sitios oficiales.
- Detección de aplicaciones instaladas en Windows.
- Administración de aplicaciones instaladas por Packwire.
- Creación y mantenimiento de shims para acceder a los binarios desde el PATH.
- Consulta de actualizaciones de Packwire.

## Uso desde la consola

```powershell
# Mostrar los paquetes disponibles
packwire available

# Instalar un paquete
packwire install git

# Instalar una versión específica
packwire install python --version 3.14

# Listar las aplicaciones instaladas
packwire list

# Consultar un paquete
packwire info python
```

## Configuración y datos

Packwire guarda sus datos de usuario en `%APPDATA%\packwire`, incluyendo el catálogo local, la caché, el estado de instalaciones y los shims.

Los recursos de la aplicación se distribuyen junto al ejecutable:

- `manifests/`: catálogo de paquetes.
- `web/`: interfaz gráfica y recursos frontend.

## Compatibilidad

La versión estable actual está preparada y distribuida oficialmente para Windows x64. Los paquetes del catálogo pueden incluir referencias a otros sistemas operativos, pero eso no implica que el ejecutable de Packwire esté distribuido o validado para esas plataformas.
