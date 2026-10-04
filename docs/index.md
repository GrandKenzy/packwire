# Packwire

Packwire es un gestor y descargador autónomo de paquetes y entornos de desarrollo para sistemas operativos Windows, Linux y macOS, implementado en Python 3.9+ bajo una arquitectura desacoplada basada en el patrón Visitor y una cola asíncrona de tareas en segundo plano. Su objetivo primordial es resolver la dispersión de versiones y dependencias en entornos de desarrollo mediante identificadores semánticos unificados (`<nombre>@<canal_o_versión>`), garantizando aislamiento, reproducibilidad y ausencia de colisiones en el sistema de archivos del usuario.

A diferencia de los gestores convencionales que modifican invasivamente las variables globales del sistema, Packwire centraliza el acceso a todos los ejecutables a través de un único directorio de lanzadores proxy (*shims* en formatos `.cmd`/`.ps1` en Windows y scripts POSIX ejecutables en Linux/macOS). El sistema integra soporte nativo para resolución dinámica de versiones en canales `stable` y versiones fijas inmutables `fixed`, orquestando tres modos de despliegue: extracción directa desatendida (`here`), ejecución de scripts oficiales con elevación de privilegios (`command` vía UAC, `osascript` o `pkexec`/`sudo`) y navegación asistida a portales oficiales (`site`). Además, provee una interfaz gráfica moderna acelerada por hardware (Microsoft Edge WebView2 en Windows, WebKitGTK en Linux y Safari WebKit en macOS) sin el sobrecoste de memoria asociado a Electron.

---

## ⚙️ Arquitectura del Sistema

El núcleo de Packwire se organiza en capas jerárquicas estrictamente delimitadas:

1. **Capa de Abstracción y Modelos (`packwire.core.models`):** Define los contratos inmutables de datos mediante estructuras tipadas (`Manifest`, `InstallConfig`, `PackageState`). Implementa un algoritmo determinista de cálculo de integridad basado en hashing criptográfico SHA-256 de campos canónicos normalizados y soporte para sobreescrituras por sistema operativo (`os`).
2. **Capa de Resolución Dinámica (`packwire.core.resolver`):** Interroga de forma segura las APIs y repositorios upstream oficiales (como python.org y nodejs.org) para determinar URLs de descarga, sumas de verificación y arquitecturas binarias antes de la adquisición.
3. **Capa de Ejecución y Visitantes (`packwire.core.visitors`):** Desacopla la lógica algorítmica de los modelos mediante componentes especializados: `DownloaderVisitor` (descargas atómicas con fragmentos `.part` y validación de hash), `PatherVisitor` (generación de shims `.cmd`/`.ps1`/POSIX y manipulación del registro `HKCU\Environment` o `~/.bashrc`/`~/.zshrc`), `ManifestVisitor` (análisis sintáctico y validación de esquemas), `UninstallerVisitor` (remoción limpia y reversión de estado) y `UpdaterVisitor` (detección de desviaciones de versión upstream).
4. **Capa de Concurrencia (`packwire.core.task_queue`):** Cola de prioridades ejecutada en hilos demonio secundarios, que procesa trabajos secuenciales o concurrentes sin bloquear el hilo principal ni la interfaz de usuario.
5. **Capa de Presentación e Interfaz (`packwire.core.gui`):** Dashboard moderno renderizado en motor web nativo conectado a un puente bidireccional IPC (`GuiBridge`), soportando persistencia de temas visuales en `configs.json` y fallback automático a servidor HTTP local.
6. **Capa de Distribución y Despliegue (`packwire.installer_self` y `packwire.packager`):** Generador de ejecutables autónomos PE mediante PyInstaller, scripts de instalación para Windows (`install.bat` / `install.ps1`), script POSIX para Linux/macOS (`install.sh`) y empaquetado para instaladores nativos con Inno Setup.

---

## 📋 Mapa de Documentación

### 1. Arquitectura y Fundamentos
* **[General de Arquitectura](01-Arquitectura/index.md):** Fundamentos conceptuales, directrices de diseño y topología del sistema.
* **[Flujo de Ejecución](01-Arquitectura/flujo_ejecucion.md):** Ciclo de vida integral del proceso de instalación, verificación y persistencia.
* **[Sistema de Shims y PATH](01-Arquitectura/sistema_shims.md):** Aislamiento de binarios, redirección por proxies y propagación del entorno multiplataforma.
* **[Cola de Tareas y Concurrencia](01-Arquitectura/cola_tareas.md):** Operaciones asíncronas en segundo plano, subprocesos dedicados y telemetría de eventos.

### 2. Módulos del Núcleo
* **[General de Módulos](02-Modulos/index.md):** Estructura del núcleo `packwire.core` y responsabilidades por subsistema.
* **[Modelos y Manifiestos](02-Modulos/modelos_manifiestos.md):** Especificación de contratos de datos, cálculo de hash canónico y validación.
* **[Resolución de Versiones](02-Modulos/resolucion_versiones.md):** Conexión con upstream oficial y resolución de canales `stable` vs `fixed`.
* **[Modos de Instalación](02-Modulos/instalador_modos.md):** Mecanismos operativos de los modos `here`, `command` y `site`.
* **[Subsistema de Visitantes](02-Modulos/visitantes.md):** DownloaderVisitor, PatherVisitor, UninstallerVisitor y UpdaterVisitor.

### 3. Interfaz de Usuario
* **[General de Interfaz](03-Interfaz/index.md):** Arquitectura visual, desacoplamiento y experiencia de usuario.
* **[Arquitectura GUI](03-Interfaz/arquitectura_gui.md):** Integración nativa de pywebview, WebView2 y servidor Bottle/fallback.
* **[Puente IPC y Comunicación](03-Interfaz/puente_ipc.md):** API RPC JSON expuesta entre JavaScript y Python.
* **[Sistema de Temas y Estilos](03-Interfaz/temas_estilos.md):** Hojas de estilo modulares y persistencia en `configs.json`.

### 4. Empaquetado y Distribución
* **[General de Distribución](04-Distribucion/index.md):** Modelos de despliegue para administradores y usuarios finales.
* **[Guía Completa de Instalación](04-Distribucion/guia_instalacion.md):** Manual paso a paso multiplataforma para Windows, Linux y macOS (scripts rápidos, pip, standalone y setup nativo).
* **[Integración en el Sistema](04-Distribucion/instalacion_sistema.md):** Arquitectura de shims, registro en `PATH` y accesos directos en Windows, Linux y macOS.
* **[Empaquetador PyInstaller](04-Distribucion/empaquetador_pyinstaller.md):** Construcción de `packwire.exe` standalone con resolución `_MEIPASS`.
* **[Instalador Inno Setup](04-Distribucion/instalador_inno_setup.md):** Compilación del instalador gráfico nativo para entornos empresariales.

### 5. Referencia y Diagnósticos
* **[General de Referencia](05-Referencia/index.md):** Índices rápidos, interfaces de programación y guías de soporte.
* **[Interfaz de Línea de Comandos (CLI)](05-Referencia/cli.md):** Manual formal exhaustivo de comandos, parámetros y banderas.
* **[API de Python](05-Referencia/api_python.md):** Firmas de métodos públicos, tipos exportados y contratos del paquete `packwire`.
* **[Catálogo de Diagnósticos y Errores](05-Referencia/catalogo_errores.md):** Matriz sistemática de códigos de falla, causas raíces y mitigación.

---

## 📦 Inicio Rápido e Instalación

Para ver la guía exhaustiva con resolución de problemas y opciones avanzadas, consulta la **[Guía Completa de Instalación](04-Distribucion/guia_instalacion.md)**.

### Instalación Rápida en Windows

Desde el Explorador de Archivos (doble clic) o consola:

```cmd
install.bat
```

O utilizando PowerShell directamente:

```powershell
.\install.ps1 -DesktopShortcut
```

### Instalación Rápida en Linux y macOS

Mediante el instalador POSIX:

```bash
chmod +x install.sh
./install.sh --desktop
source ~/.bashrc  # o source ~/.zshrc en macOS
```

### Instalación Manual con Pip

```bash
pip install -e .
packwire setup --desktop
```

### Uso Básico desde la Consola

```bash
# Consultar el catálogo de software disponible
packwire available

# Instalar Git o Docker
packwire install git
packwire install docker

# Instalar la última versión estable oficial de Python
packwire install python --channel stable --mode here

# Instalar una versión fija específica de Python
packwire install python@3.13 --mode here

# Abrir el panel de control gráfico moderno
packwire ui
```

### Integración en Scripts de Python

```python
import packwire

# Instalación desatendida de paquetes
packwire.install("git")
exito, mensaje = packwire.install("nodejs", channel="stable", mode="here")
if exito:
    print(f"Instalación completada: {mensaje}")

# Listar paquetes registrados en el sistema
instalados = packwire.list_installed()
for pkg_id, datos in instalados.items():
    print(f"{pkg_id} -> Versión: {datos['version']} en {datos['install_path']}")
```
