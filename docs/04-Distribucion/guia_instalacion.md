# Guía Completa de Instalación

Packwire es compatible de forma nativa con **Windows (10/11)**, **Linux** y **macOS**. Esta guía describe de forma exhaustiva todos los métodos de instalación disponibles, priorizando la instalación mediante **binarios precompilados oficiales** para evitar la necesidad de instalar Python o manipular código fuente.

---

## ⚡ Resumen Rápido: ¿Qué método elegir?

```
┌────────────────────────────────────────────────────────────────────────┐
│  ¿Quieres usar Packwire directamente sin instalar Python ni clonar?   │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ SÍ (Recomendado)
                                    ▼
       👉 MÉTODO 1: Descargar Binarios Oficiales desde GitHub Releases
          • Windows : Packwire-Setup.exe (Instalador con asistente)
                      packwire-windows-x64.zip (Portable sin instalación)
          • Linux   : packwire-linux-x64.tar.gz (Binario ejecutable)
          • macOS   : packwire-macos-universal.tar.gz (Binario universal)

                                    │ NO (Soy Desarrollador)
                                    ▼
       👉 MÉTODOS 2, 3 y 4: Scripts de automatización o pip editable
          • Windows : install.bat / install.ps1
          • POSIX   : install.sh (Linux & macOS)
          • Python  : pip install -e . && packwire setup
```

---

## 🌟 Método 1 (Recomendado): Binarios Oficiales Precompilados

> [!TIP]
> **Sin dependencias ni código fuente:** Este método **NO requiere tener Python instalado** ni descargar el repositorio. Los binarios son compilados de forma determinista mediante GitHub Actions e incluyen todo lo necesario para funcionar de inmediato.

Todos los binarios se encuentran disponibles en:  
👉 **[GitHub Releases Oficiales de Packwire](https://github.com/GrandKenzy/packwire/releases/latest)**

---

### A. En Windows

Dispones de dos opciones según tu preferencia:

#### Opción 1: Asistente de Instalación Gráfico (`Packwire-Setup.exe`)
1. Descarga **`Packwire-Setup.exe`** (o `Packwire-Setup-vX.Y.Z.exe`) desde la página de Releases.
2. Haz doble clic en el instalador descargado.
3. Selecciona tu idioma preferido (Español o Inglés).
4. El asistente configurará automáticamente:
   - Instalación segura en el espacio de usuario (`%LOCALAPPDATA%\Programs\Packwire`).
   - **Acceso directo en el Escritorio** y en el **Menú Inicio**.
   - Integración directa en la variable de entorno **PATH** del usuario.
   - Registro en *Configuración > Aplicaciones instaladas* de Windows para desinstalación con un clic.
5. Abre cualquier consola (PowerShell, CMD o Terminal) y escribe `packwire --help` para comenzar.

#### Opción 2: Versión Portable (`packwire-windows-x64.zip`)
Si no deseas instalar nada en el sistema o prefieres llevar Packwire en una memoria USB:
1. Descarga **`packwire-windows-x64.zip`**.
2. Extrae el contenido en la carpeta que desees.
3. Dentro encontrarás el ejecutable autónomo `packwire.exe` (14.2 MB).
4. Puedes ejecutarlo directamente desde la terminal o hacer doble clic para inicializarlo:
   ```cmd
   .\packwire.exe setup --desktop
   ```

---

### B. En Linux

1. Descarga el archivo comprimido **`packwire-linux-x64.tar.gz`** desde GitHub Releases.
2. Abre una terminal en tu carpeta de descargas y ejecuta:
   ```bash
   # 1. Extraer el binario
   tar -xzf packwire-linux-x64.tar.gz

   # 2. Asignar permisos de ejecución
   chmod +x packwire

   # 3. Mover a una carpeta en tu PATH de usuario
   mkdir -p ~/.local/bin
   mv packwire ~/.local/bin/

   # 4. Inicializar shims y accesos directos
   packwire setup --desktop
   ```
3. Si `~/.local/bin` no está en tu PATH, agrégalo a tu `~/.bashrc`:
   ```bash
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

---

### C. En macOS

1. Descarga el paquete **`packwire-macos-universal.tar.gz`** desde GitHub Releases.
2. Abre la terminal y ejecuta:
   ```bash
   # 1. Extraer el binario
   tar -xzf packwire-macos-universal.tar.gz

   # 2. Asignar permisos
   chmod +x packwire

   # 3. Mover a la ruta de ejecutables del sistema o usuario
   sudo mv packwire /usr/local/bin/  # O mv packwire ~/.local/bin/

   # 4. Inicializar shims y accesos
   packwire setup --desktop
   ```

---

## 🛠️ Métodos para Desarrolladores (Desde el Código Fuente)

Si deseas colaborar en el desarrollo de Packwire, modificar manifiestos o extender el núcleo, utiliza los métodos basados en repositorio.

### Método 2: Scripts de Un Clic para Windows (`install.bat` / `install.ps1`)

Requiere tener Python 3.9+ instalado en el sistema.

* **Por doble clic:** Haz doble clic sobre [`install.bat`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/install.bat) en la raíz del proyecto.
* **Desde PowerShell:**
  ```powershell
  .\install.ps1 -DesktopShortcut
  ```
  *Banderas disponibles:* `-DesktopShortcut`, `-NoStartMenu`, `-NoPath`.

> [!NOTE]
> `install.bat` invoca automáticamente PowerShell con la directiva `-ExecutionPolicy Bypass`, evitando bloqueos de seguridad por políticas de ejecución locales de Windows.

---

### Método 3: Script Universal POSIX para Linux y macOS (`install.sh`)

Requiere `python3` (3.9+) y `pip`.

```bash
chmod +x install.sh
./install.sh --desktop
source ~/.bashrc   # o source ~/.zshrc en macOS
```

---

### Método 4: Instalación vía Pip en Entornos Virtuales

Para entornos aislados (`venv` o `conda`):

```bash
# Modo editable para desarrollo:
pip install -e .

# Inicializar shims del sistema y accesos directos:
packwire setup --desktop
```

---

## ✅ Verificación de la Instalación

Una vez instalado (por cualquiera de los métodos), abre una nueva ventana de terminal y comprueba su funcionamiento:

1. **Comprobar la versión y ayuda:**
   ```bash
   packwire --help
   ```

2. **Comprobar el estado del directorio de Shims y PATH:**
   ```bash
   packwire path
   ```
   Si la ruta de shims no está activa en tu sesión:
   ```bash
   packwire path --add
   ```

3. **Explorar el catálogo de paquetes:**
   ```bash
   packwire available
   ```

4. **Instalar un entorno de prueba:**
   ```bash
   packwire install python@stable --mode here
   ```

5. **Lanzar la interfaz gráfica:**
   ```bash
   # Ventana nativa acelerada por hardware:
   packwire ui

   # O en modo servidor web en tu navegador:
   packwire ui --web
   ```

---

## 🔄 Actualización de Packwire

* **Usuarios de Binarios Precompilados:** Simplemente descarga el nuevo instalador o binario desde [GitHub Releases](https://github.com/GrandKenzy/packwire/releases/latest) y ejecútalo; reemplazará la versión previa conservando todas tus herramientas instaladas.
* **Usuarios de Repositorio Git:**
  ```bash
  git pull origin main
  pip install -e .
  ```
* **Para actualizar los paquetes gestionados por Packwire:**
  ```bash
  packwire update
  ```

---

## 🗑️ Desinstalación Completa y Limpieza

### Desde Windows con Asistente
Si instalaste mediante `Packwire-Setup.exe`, dirígete a:
> **Configuración > Aplicaciones > Aplicaciones instaladas > Packwire > Desinstalar**

El desinstalador purgará automáticamente los binarios y revertirá el registro de variables de entorno.

### Mediante la Consola (Cualquier Plataforma)
Para desinstalar todos los paquetes gestionados, eliminar shims, cachés y retirar la ruta del `PATH`:
```bash
packwire remove --all
```

---

## 🛠️ Solución de Problemas Frecuentes

### 1. "El comando 'packwire' no se reconoce"
* **Causa:** La terminal actual se abrió antes de actualizar la variable de entorno `PATH`.
* **Solución:** Cierra todas las ventanas de terminal y vuelve a abrir una nueva ventana. En Linux/macOS ejecuta `source ~/.bashrc` o `source ~/.zshrc`.

### 2. Advertencia de SmartScreen en Windows al abrir el instalador
* **Causa:** En versiones recientes de software de código abierto recién publicado, Windows SmartScreen advierte sobre ejecutables que aún están construyendo reputación en la nube de Microsoft Defender.
* **Solución:** Haz clic en **"Más información"** y luego en **"Ejecutar de todas formas"**.

### 3. Faltan dependencias de ventana nativa en Linux
* **Causa:** Algunas distribuciones mínimas de Linux no incluyen WebKitGTK.
* **Solución:** Instala la librería del sistema:
  ```bash
  sudo apt-get install libwebkit2gtk-4.0-37  # Debian/Ubuntu
  ```
  O ejecuta Packwire directamente en el navegador:
  ```bash
  packwire ui --web
  ```
