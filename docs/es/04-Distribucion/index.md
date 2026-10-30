# 04. Empaquetado y Distribución

> **Versión Actual:** `1.2.0`
> **Formatos:** Binarios nativos, ZIP portable e instalador Inno Setup (.exe)

Esta sección cubre todos los métodos de despliegue de Packwire, desde la compilación de ejecutables nativos hasta scripts de automatización e integración con el sistema operativo anfitrión.

---

## Contenido de la Sección

| Guía | Resumen |
| :--- | :--- |
| **[01. Guía Completa de Instalación](./01-guia-instalacion.md)** | Manual exhaustivo de instalación multiplataforma para Windows, Linux y macOS mediante binarios nativos. |
| **[02. Integración en el Sistema](./02-instalacion-sistema.md)** | Detalle técnico del registro en `PATH`, creación de shims y accesos directos en el menú de inicio y escritorio. |
| **[04. Instalador Inno Setup](./04-instalador-inno-setup.md)** | Compilación del instalador gráfico nativo para Windows (`Packwire-Setup.exe`) con compresión LZMA2 y configuración guiada. |

---

## Resumen Rápido

1. **Usuarios Finales:** Se recomienda el instalador asistido `Packwire-Setup.exe` o los portables standalone de GitHub Releases (sin requerir Python preinstalado).
2. **Desarrolladores:** Compilar los binarios desde `packwire-rs/Cargo.toml` con Rust estable.
3. **Binarios duales:** `packwire.exe` para ejecución en terminal/consola y `packwirew.exe` para inicio silencioso de la interfaz gráfica.
