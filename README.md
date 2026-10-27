<p align="center">
  <img src="badge/banner.jpg" alt="Packwire - Centralized Package & Tool Manager" width="100%">
</p>

# Packwire 📦⚡

Gestor y descargador moderno de paquetes y entornos de desarrollo para Windows, Linux y macOS.

---

## 🚀 Características Principales

- **Multiplataforma Nativo:** Soporte integral para **Windows** (shims `.cmd`/`.ps1`, registro `HKCU`, accesos directos y UAC), **Linux** (shims POSIX ejecutables, `~/.bashrc`, `.desktop` XDG y `pkexec`/`sudo`) y **macOS** (Safari WebKit, `.command`, `osascript` con privilegios de administrador y Homebrew/Xcode).
- **Identificación Canónica sin Colisiones:** Los paquetes usan identificadores semánticos estructurados (`<nombre>@<canal_o_versión>`, ej. `python@stable`, `python@3.13`, `git`, `docker`), evitando duplicados y colisiones de binarios en el sistema.
- **Dos Canales de Versión:**
  - `stable`: Resuelve dinámicamente la última versión estable oficial del software (consultando upstream oficial).
  - `fixed`: Pinned a una versión exacta inmutable (ej. 3.12.9, 3.13.2, 3.14.8).
- **Tres Modos de Instalación:**
  - `here`: Descarga e instalación 100% automatizada (headless) usando paquetes embebidos/portables o instaladores silenciosos, sin abrir el navegador.
  - `command`: Orquesta instalaciones oficiales por consola con soporte para elevación de privilegios de administrador (UAC en Windows, `osascript` en macOS, `pkexec`/`sudo` en Linux).
  - `site`: Redirige al sitio oficial de descarga para instalación asistida (ej. Visual Studio Code, Docker Desktop).
- **Arquitectura de Shims y PATH Limpio:**
  - En lugar de saturar el registro del sistema o archivos de perfil con múltiples carpetas en el `PATH`, Packwire mantiene una única carpeta central de Shims.
  - Permite que múltiples versiones convivan sin conflicto (`python.cmd`, `python3132.cmd`, `python3129.cmd` en Windows, o scripts ejecutables en Linux/macOS).
- **Arquitectura modular Rust:** Descarga, resolución, instalación, desinstalación, shims y actualización están separados en módulos nativos con estado persistente y trabajos en segundo plano.

---

## 💻 Uso desde la Terminal (CLI)

```bash
# Ver paquetes disponibles
packwire available

# Abrir la GUI nativa
packwirew.exe
packwire available | findstr /I "Linux"

# Ver información de un paquete resuelto
packwire info git
packwire info docker

# Instalar Python stable en modo headless
packwire install python --channel stable --mode here

# Instalar Git o Docker
packwire install git
packwire install docker

# Toolchains estables que siguen el upstream oficial
packwire install gcc                 # w64devkit x64 más reciente, con SHA-256
packwire install gcc-ucrt64          # GCC + UCRT64 de MSYS2, actualizado por pacman
packwire install clang               # LLVM/Clang oficial más reciente
packwire install msvc                # MSVC Build Tools estable vía winget
packwire install rufus               # Rufus portable más reciente
packwire install ventoy              # Ventoy para USB multiboot
packwire install pwsh                # PowerShell 7 x64 más reciente

# Instalar versión fija
packwire install python@3.12 --mode here

# Listar instalaciones actuales
packwire list

# Verificar o agregar shims al PATH del usuario
packwire path --add

# Iniciar la interfaz gráfica nativa sin consola
packwirew.exe
```

---

## 📦 Instalación y Descarga

> 📖 **Documentación completa:** Para instrucciones detalladas y solución de problemas, consulta la [Guía Completa de Instalación](docs/es/04-Distribucion/01-guia-instalacion.md) o [Full Installation Guide](docs/en/04-Distribution/01-installation-guide.md).

### 🌟 Método Recomendado: Binarios Oficiales Precompilados (Sin Python / Sin Código Fuente)

No necesitas tener Python instalado ni lidiar con el código fuente del proyecto. Puedes descargar los binarios oficiales compilados automáticamente desde:  
👉 **[GitHub Releases de Packwire](https://github.com/GrandKenzy/packwire/releases/latest)**

* **Windows:**
  - **Instalador con Asistente:** Descarga `Packwire-Setup.exe` y sigue el asistente gráfico (agrega al `PATH`, crea accesos directos y permite desinstalación limpia desde Windows).
  - **Portable:** Descarga `packwire-windows-x64.zip`, descomprime y usa `packwire.exe` directamente.
* **Linux:** Descarga `packwire-linux-x64.tar.gz`, extrae el binario y muévelo a `~/.local/bin/`.
* **macOS:** Descarga `packwire-macos-universal.tar.gz`, extrae el binario y muévelo a `/usr/local/bin/`.

---

### 🛠️ Para Desarrolladores (Desde el Código Fuente)

Si deseas colaborar o trabajar directamente sobre el código:

Instala Rust estable y compila los binarios nativos:

```powershell
cargo build --release --manifest-path packwire-rs/Cargo.toml --bins
```

Para generar el instalador de Windows, compila también `installer.iss` con Inno Setup.

### 3. Generación del Instalador Nativo de Windows (.exe Setup)
Packwire incluye un script para compilar un asistente de instalación nativo con **Inno Setup**:
1. Compila los binarios Rust:
   ```bash
   cargo build --release --manifest-path packwire-rs/Cargo.toml --bins
   ```
2. Compila el instalador con Inno Setup usando `installer.iss`:
   ```bash
   iscc installer.iss
   ```
   Esto generará `dist/Packwire-Setup.exe`, listo para distribuir sin necesidad de tener Python instalado.

---

## 🖥️ Interfaz Gráfica (GUI) con HTML5 y CSS3

Packwire incluye una interfaz moderna tipo Dashboard desarrollada íntegramente en **HTML5 + CSS3 + JavaScript**, con diseño oscuro inspirado en herramientas de desarrollo modernas.

- **Modo Ventana Nativa:** Se ejecuta desde Rust mediante `wry` y el motor web preinstalado del sistema operativo (Microsoft Edge WebView2 en Windows, Safari WebKit en macOS, WebKitGTK en Linux). Cero peso extra de Electron.
- **Servidor local:** La GUI inicia un servidor HTTP local interno para cargar el dashboard en WebView2/WebKit.

```bash
# Lanzar interfaz nativa sin consola
packwirew.exe
```

---

## 📚 Documentación / Documentation

Documentación completa y bilingüe disponible en Español e Inglés:
Full documentation is available in English and Spanish:

- 🇪🇸 **[Documentación en Español](docs/es/index.md)**
- 🇬🇧 **[English Documentation](docs/en/index.md)**

### Mapa de Secciones / Section Map
- **01. Arquitectura / Architecture:** [ES](docs/es/01-Arquitectura/index.md) \| [EN](docs/en/01-Architecture/index.md)
- **02. Módulos / Modules:** [ES](docs/es/02-Modulos/index.md) \| [EN](docs/en/02-Modules/index.md)
- **03. Interfaz / Interface:** [ES](docs/es/03-Interfaz/index.md) \| [EN](docs/en/03-Interface/index.md)
- **04. Distribución / Distribution:** [ES](docs/es/04-Distribucion/index.md) \| [EN](docs/en/04-Distribution/index.md)
- **05. Referencia / Reference:** [ES](docs/es/05-Referencia/index.md) \| [EN](docs/en/05-Reference/index.md)

---

## 📁 Estructura del Proyecto

```text
packwire/
├── docs/                         # Documentación bilingüe estructurada (Gram standard)
│   ├── es/                       # Documentación completa en Español
│   │   ├── 01-Arquitectura/
│   │   ├── 02-Modulos/
│   │   ├── 03-Interfaz/
│   │   ├── 04-Distribucion/
│   │   ├── 05-Referencia/
│   │   └── index.md
│   ├── en/                       # Full documentation in English
│   │   ├── 01-Architecture/
│   │   ├── 02-Modules/
│   │   ├── 03-Interface/
│   │   ├── 04-Distribution/
│   │   ├── 05-Reference/
│   │   └── index.md
│   └── index.md                  # Gateway bilingüe
├── assets/                       # Recursos distribuibles de la aplicación
│   ├── manifests/                # Catálogo de manifests
│   └── web/                      # Dashboard HTML/CSS/JavaScript
├── packwire-rs/                  # Implementación nativa Rust
│   ├── Cargo.toml
│   └── src/                      # CLI, GUI y núcleo de instalación
├── installer.iss                 # Script de Inno Setup para crear Packwire-Setup.exe
└── README.md
```
