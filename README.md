# Packwire 📦⚡

Gestor y descargador moderno de paquetes y entornos para Windows.

---

## 🚀 Características Principales

- **Identificación Canónica sin Colisiones:** Los paquetes usan identificadores canónicos semánticos estructurados (`<nombre>@<canal_o_versión>`, ej. `python@stable`, `python@3.13`), evitando duplicados y colisiones de binarios en el sistema.
- **Dos Canales de Versión:**
  - `stable`: Resuelve dinámicamente la última versión estable oficial del lenguaje (consultando upstream oficial).
  - `fixed`: Pinned a una versión exacta inmutable (ej. 3.12.9, 3.13.2, 3.14.8).
- **Dos Modos de Instalación:**
  - `here`: Descarga e instalación 100% automatizada (headless) usando paquetes embebidos/portables o instaladores silenciosos, sin abrir el navegador.
  - `site`: Redirige al sitio oficial de descarga para instalación guiada o manual.
- **Arquitectura de Shims y PATH Limpio:**
  - En lugar de saturar el registro del sistema con múltiples carpetas en el `PATH`, Packwire mantiene una única carpeta de Shims (`%APPDATA%/packwire/shims`).
  - Permite que múltiples versiones convivan sin conflicto (`python.cmd`, `python3132.cmd`, `python3129.cmd`).
- **Arquitectura de Visitors / Pipeline:**
  - `DownloaderVisitor`: Descarga con barra de progreso, descargas atómicas `.part` y caché.
  - `PatherVisitor`: Generación de shims y enlace al PATH del usuario (`HKCU\Environment`).
  - `ManifestVisitor`: Parser tolerante, validador de manifests y cálculo de hash canónico SHA256.
  - `UninstallerVisitor`: Limpieza de binarios, carpetas y shims.
  - `UpdaterVisitor`: Detección y aplicación de nuevas versiones para canales `stable`.

---

## 🛠️ Uso desde Python

```python
import packwire

# 1. Instalar última versión estable en modo automático ('here')
packwire.install("python", channel="stable", mode="here")

# 2. Instalar una versión fija específica
packwire.install("python@3.13", mode="here")

# 3. Redirigir al sitio oficial para descarga manual ('site')
packwire.install("python@3.14", mode="site")

# 4. Listar paquetes instalados
installed = packwire.list_installed()
print(installed)

# 5. Comprobar actualizaciones disponibles en segundo plano
updates = packwire.check_updates()
print(updates)

# 6. Actualizar paquetes estables
packwire.update()

# 7. Reinstalar un paquete
packwire.reinstall("python@stable")

# 8. Desinstalar
packwire.uninstall("python@3.13.2")
```

---

## 💻 Uso desde la Terminal (CLI)

```bash
# Ver paquetes disponibles
python -m packwire available

# Ver información de un paquete resuelto
python -m packwire info python@stable

# Instalar Python stable en modo headless
python -m packwire install python --channel stable --mode here

# Instalar versión fija
python -m packwire install python@3.12 --mode here

# Listar instalaciones actuales
python -m packwire list

# Comprobar actualizaciones
python -m packwire update

# Desinstalar un paquete
python -m packwire remove python@3.12.9

# Instalar e integrar Packwire en Windows (Shims, PATH, Menú Inicio y Escritorio)
packwire setup --desktop

# Compilar ejecutable standalone (.exe)
packwire build --exe

# Compilar paquete de distribución Python (.whl y .tar.gz)
packwire build --wheel

# O compilar ambos simultáneamente
packwire build --all

# Verificar o agregar shims al PATH del usuario
packwire path --add

# Iniciar la interfaz gráfica de usuario (GUI) en ventana nativa
packwire ui

# O abrirla en el navegador en modo servidor web
packwire ui --web
```

---

## 📦 Instalación y Empaquetado

### 1. Instalación Rápida en Windows
- **Doble clic en `install.bat`** o ejecutar en PowerShell:
  ```powershell
  .\install.ps1 -DesktopShortcut
  ```
- O si ya tienes el paquete o repositorio:
  ```bash
  pip install -e .
  packwire setup --desktop
  ```

### 2. Generación del Instalador Nativo de Windows (.exe Setup)
Packwire incluye un script para compilar un asistente de instalación nativo con **Inno Setup**:
1. Genera el ejecutable standalone:
   ```bash
   packwire build --exe
   ```
2. Compila el instalador con Inno Setup usando `installer.iss`:
   ```bash
   iscc installer.iss
   ```
   Esto generará `dist/Packwire-Setup.exe`, listo para distribuir a cualquier usuario sin necesidad de tener Python instalado.

---

## 🖥️ Interfaz Gráfica (GUI) con HTML5 y CSS3

Packwire incluye una interfaz moderna tipo Dashboard desarrollada íntegramente en **HTML5 + CSS3 + JavaScript**, con diseño oscuro inspirado en herramientas de desarrollo modernas.

- **Modo Ventana Nativa:** Se ejecuta a través de `pywebview` usando el motor web preinstalado del sistema operativo (Microsoft Edge WebView2 en Windows, Safari WebKit en macOS, WebKitGTK en Linux). Cero peso extra de Electron.
- **Modo Servidor Web (`--web`):** Cuenta con un servidor HTTP ligero incorporado sin dependencias externas que puede servir la interfaz a cualquier navegador web si se está en un entorno headless o remoto.

```python
import packwire

# Lanzar interfaz nativa
packwire.launch_gui()

# O en modo servidor web
packwire.launch_gui(web_mode=True)
```

---

## 📁 Estructura del Proyecto

```text
packwire/
├── packwire/                     # Paquete principal autosuficiente
│   ├── __init__.py               # API pública de alto nivel
│   ├── __main__.py               # Entrypoint para python -m packwire
│   ├── cli.py                    # Interfaz de línea de comandos (CLI)
│   ├── installer_self.py         # Instalador propio en Windows (shims, PATH, accesos directos)
│   ├── packager.py               # Automatización de build (PyInstaller & Wheel)
│   └── core/                     # Núcleo modular de Packwire
│       ├── config.py             # Rutas globales (%APPDATA%/packwire)
│       ├── models.py             # Modelos de datos (Manifest, PackageState)
│       ├── resolver.py           # Resolvedor de versiones (Python API oficial, Node, etc.)
│       ├── state.py              # Detección de colisiones y registro de estado
│       ├── installer.py          # Orquestador de modos 'here', 'command' y 'site'
│       ├── task_queue.py         # Cola de descargas e instalaciones concurrentes
│       ├── manifests/            # Catálogo de manifests incluidos (Python, C/GCC, Node, FFMPEG, etc.)
│       ├── gui/                  # Interfaz Gráfica (HTML/CSS)
│       │   ├── bridge.py         # Puente Python <-> JS
│       │   ├── server.py         # Servidor HTTP local ligero
│       │   ├── window.py         # Lanzador nativo pywebview
│       │   └── web/              # Dashboard HTML5/CSS3 con soporte de temas
│       └── visitors/             # Pipeline de operaciones
│           ├── downloader.py     # Descarga con progreso y atomic part
│           ├── pather.py         # Shims .cmd/.ps1 y registro PATH
│           ├── process_manifest.py
│           ├── uninstaller.py    # Desinstalador limpio y completo
│           └── updater.py        # Actualizador de paquetes stable
├── install.bat                   # Instalador rápido de un clic para Windows
├── install.ps1                   # Script de instalación automatizada en PowerShell
├── installer.iss                 # Script de Inno Setup para crear Packwire-Setup.exe
├── packwire.spec                 # Especificación de PyInstaller para packwire.exe
├── pyproject.toml                # Metadatos del paquete (PEP 517/621)
├── setup.py                      # Configuración estándar de distribución
├── tests/
│   └── test_packwire.py          # Pruebas unitarias automatizadas
├── example.py                    # Script de demostración interactiva
└── README.md
```
