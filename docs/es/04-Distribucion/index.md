# 04. Empaquetado y Distribución

> **Versión Actual:** `0.1.4`
> **Formatos:** Standalone PE (.exe), Wheel Python (.whl), Inno Setup (.exe), Scripts de instalación (.bat, .ps1, .sh)

Esta sección cubre todos los métodos de despliegue de Packwire, desde la compilación de ejecutables nativos hasta scripts de automatización e integración con el sistema operativo anfitrión.

---

## Contenido de la Sección

| Guía | Resumen |
| :--- | :--- |
| **[01. Guía Completa de Instalación](./01-guia-instalacion.md)** | Manual exhaustivo de instalación multiplataforma para Windows, Linux y macOS: binarios precompilados, scripts rápidos y pip. |
| **[02. Integración en el Sistema](./02-instalacion-sistema.md)** | Detalle técnico del registro en `PATH`, creación de shims y accesos directos en el menú de inicio y escritorio. |
| **[03. Empaquetador PyInstaller](./03-empaquetador-pyinstaller.md)** | Generación de ejecutables standalone (`packwire.exe` y `packwirew.exe`) con inclusión de assets vía `packwire.spec`. |
| **[04. Instalador Inno Setup](./04-instalador-inno-setup.md)** | Compilación del instalador gráfico nativo para Windows (`Packwire-Setup.exe`) con compresión LZMA2 y configuración guiada. |

---

## Resumen Rápido

1. **Usuarios Finales:** Se recomienda el instalador asistido `Packwire-Setup.exe` o los portables standalone de GitHub Releases (sin requerir Python preinstalado).
2. **Desarrolladores Python:** Instalación directa mediante `pip install packwire` o clonando el repositorio con `install.bat`, `install.ps1` o `install.sh`.
3. **Binarios duales:** `packwire.exe` para ejecución en terminal/consola y `packwirew.exe` para inicio silencioso de la interfaz gráfica.
