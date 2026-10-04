# Distribución, Instalación y Despliegue Multiplataforma

Esta sección detalla los mecanismos y artefactos de distribución provistos por Packwire para su aprovisionamiento en sistemas operativos **Windows**, **Linux** y **macOS**. El proyecto ofrece una suite integral de empaquetado y aprovisionamiento que cubre desde scripts de arranque rápido de un solo clic hasta paquetes wheel para PyPI, binarios PE autónomos compilados con PyInstaller y asistentes gráficos para entornos corporativos con Inno Setup.

A través de estos módulos se expone la guía de instalación para usuarios finales, la arquitectura interna del auto-instalador de sistema, la receta de compilación de ejecutables y el script de generación de instaladores MSI/Setup.

---

## Módulos y Especificaciones

* **[Guía Completa de Instalación](guia_instalacion.md):** Manual integral paso a paso para usuarios y administradores: scripts rápidos (`install.bat`, `install.ps1`, `install.sh`), instalación vía `pip`, ejecutable standalone y asistente nativo Inno Setup.
* **[Integración y Auto-Instalación en el Sistema](instalacion_sistema.md):** Arquitectura técnica del subsistema `installer_self.py`, generación de shims del sistema (`.cmd`/`.ps1`/POSIX), registro en el `PATH` del usuario y creación de accesos directos de escritorio (`.lnk`, `.desktop`, `.command`).
* **[Empaquetador PyInstaller](empaquetador_pyinstaller.md):** Automatización de compilación con `packwire.spec`, resolución de recursos congelados (`sys._MEIPASS`) y generación de `packwire.exe` standalone optimizado (14.2 MB).
* **[Instalador Inno Setup](instalador_inno_setup.md):** Especificación del instalador nativo `installer.iss`, integración en *Programas y Características*, tareas personalizables de registro de entorno y desinstalador automático.

---

## Flujo de Trabajo

El pipeline de aprovisionamiento, compilación y empaquetado de Packwire opera a través de las siguientes vías:

```
[Código Fuente de Packwire]
           │
           ├───► [Aprovisionamiento Rápido] ───► install.bat / install.ps1 (Windows)
           │                                ───► install.sh (Linux & macOS)
           │                                ───► pip install -e . && packwire setup
           │
           ├───► [packwire build --wheel]   ───► dist/packwire-0.1.0-py3-none-any.whl
           │                                     (Distribución estándar PyPI / pip)
           │
           └───► [packwire build --exe]     ───► dist/packwire.exe (14.2 MB)
                       │                         (Ejecutable autónomo PE con icono)
                       │
                       ▼
           [iscc installer.iss]             ───► dist/Packwire-Setup.exe
                                                  (Instalador nativo gráfico de Windows)
```
