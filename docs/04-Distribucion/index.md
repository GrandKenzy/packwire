# Distribución y Despliegue en Windows

Esta sección detalla los mecanismos y artefactos de distribución provistos por Packwire para su aprovisionamiento en sistemas operativos Windows. El proyecto ofrece una suite integral de empaquetado que cubre desde la instalación en entornos con Python preinstalado hasta la entrega de binarios completamente autónomos y asistentes gráficos para entornos corporativos o usuarios finales.

A través de estos módulos se expone el funcionamiento del auto-instalador de sistema, la receta de compilación de PyInstaller y el script de generación de instaladores MSI/Setup mediante Inno Setup.

---

## Módulos y Especificaciones

* **[Integración en el Sistema](instalacion_sistema.md):** Auto-instalador `installer_self.py`, generación de shims del sistema, registro en el `PATH` del usuario y creación de accesos directos silenciosos mediante `pythonw.exe`.
* **[Empaquetador PyInstaller](empaquetador_pyinstaller.md):** Automatización de compilación con `packwire.spec`, resolución de recursos congelados (`sys._MEIPASS`) y generación de `packwire.exe` standalone.
* **[Instalador Inno Setup](instalador_inno_setup.md):** Especificación del instalador nativo `installer.iss`, integración en *Programas y Características*, tareas personalizables y desinstalador automático.

---

## Flujo de Trabajo

El pipeline de compilación y empaquetado de Packwire opera a través de las siguientes etapas:

```
[Código Fuente de Packwire]
           │
           ├───► [packwire build --wheel] ──────► dist/packwire-0.1.0-py3-none-any.whl
           │                                       (Distribución estándar PyPI / pip)
           │
           └───► [packwire build --exe]   ──────► dist/packwire.exe (14.2 MB)
                       │                           (Ejecutable autónomo PE con icono)
                       │
                       ▼
           [iscc installer.iss]           ──────► dist/Packwire-Setup.exe
                                                   (Instalador nativo gráfico de Windows)
```
