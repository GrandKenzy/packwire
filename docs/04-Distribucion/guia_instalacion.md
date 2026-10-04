# Guía Completa de Instalación

Packwire es compatible de forma nativa con **Windows (10/11)**, **Linux** y **macOS**. Esta guía describe de forma exhaustiva todos los métodos de instalación disponibles, desde instaladores automáticos de un solo clic hasta paquetes independientes para entornos sin Python preinstalado.

---

## 📋 Requisitos del Sistema

Antes de comenzar, asegúrate de que tu equipo cumple con los siguientes requisitos mínimos:

| Plataforma | Requisito Mínimo | Motor Gráfico (GUI) | Permisos Requeridos |
| :--- | :--- | :--- | :--- |
| **Windows** | Windows 10 (1809+) o Windows 11 | Microsoft Edge WebView2 (integrado) | Usuario estándar (`HKCU`) |
| **Linux** | Kernel 4.15+, glibc 2.27+ | WebKitGTK (`libwebkit2gtk-4.0` o `4.1`) | Usuario estándar (`~/.local/share`) |
| **macOS** | macOS 11.0 (Big Sur) o superior | Safari WebKit (nativo) | Usuario estándar (`~/.local/share`) |

> [!NOTE]
> **Permisos de Administrador no requeridos:** Packwire se instala y opera íntegramente en el espacio del usuario actual (`%APPDATA%` en Windows y `~/.local/share` en Linux/macOS). No requiere privilegios de Administrador ni `root` para su funcionamiento cotidiano, a menos que se instalen paquetes del sistema mediante el modo `command` (ej. Chocolatey o Docker oficial).

---

## 🚀 Métodos de Instalación

Selecciona el método más adecuado para tu entorno de trabajo:

```
                              ¿Tienes Python 3.9+ instalado?
                                    /               \
                                 SÍ                  NO
                                /                      \
          ¿Qué sistema utilizas?               [Windows Standalone / Inno Setup]
          /                    \                    Packwire-Setup.exe o packwire.exe
    [Windows]              [Linux / macOS]
   install.bat                install.sh
  o install.ps1              (POSIX bash)
```

---

### Método 1: Script de Un Clic para Windows (`install.bat` / `install.ps1`)

Este método es el más rápido y recomendado para usuarios y desarrolladores en Windows.

#### Opción A: Doble Clic (Sin Consola)
1. Localiza el archivo [`install.bat`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/install.bat) en la carpeta raíz del proyecto.
2. Haz **doble clic** sobre `install.bat`.
3. El script detectará tu instalación de Python, instalará las dependencias en modo editable (`pip install -e .`), generará los shims en `%APPDATA%\packwire\shims`, registrará la ruta en el `PATH` del usuario y creará el acceso directo en el Escritorio.
4. Al culminar, la ventana se pausará informando el resultado para que puedas revisarlo.

#### Opción B: Desde PowerShell
Abre una ventana de PowerShell y ejecuta:

```powershell
.\install.ps1 -DesktopShortcut
```

#### Parámetros soportados por `install.ps1`:
* `-DesktopShortcut`: Crea el acceso directo de la interfaz gráfica en tu Escritorio (`Packwire.lnk`).
* `-NoStartMenu`: Omite la creación del acceso en la carpeta de Programas del Menú Inicio.
* `-NoPath`: Omite la inyección automática de la carpeta de shims en la variable de entorno `PATH` del usuario.

> [!TIP]
> **Política de Ejecución de PowerShell:** Si tu equipo tiene restringida la ejecución de scripts (`Restricted`), `install.bat` invoca automáticamente PowerShell con la directiva `-ExecutionPolicy Bypass`, garantizando la instalación sin necesidad de alterar la directiva global de seguridad del sistema.

---

### Método 2: Script Universal para Linux y macOS (`install.sh`)

Para sistemas basados en Unix (distribuciones Linux como Ubuntu, Debian, Fedora, Arch y macOS), Packwire incluye el script automatizado [`install.sh`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/install.sh).

1. Abre tu terminal y sitúate en la raíz del proyecto.
2. Otorga permisos de ejecución al script:
   ```bash
   chmod +x install.sh
   ```
3. Ejecuta el instalador con las opciones deseadas:
   ```bash
   ./install.sh --desktop
   ```

#### Acciones automáticas efectuadas por `install.sh`:
* Detecta la presencia de `python3` (versión 3.9 o superior) y `pip`.
* Instala Packwire y sus dependencias (`pywebview`, `bottle`).
* Compila los shims POSIX ejecutables (`0755`) en `~/.local/share/packwire/shims`.
* Añade la directiva de exportación en tus archivos de perfil de shell (`~/.bashrc`, `~/.zshrc`, `~/.profile`).
* En Linux, crea el lanzador de escritorio `~/.local/share/applications/packwire.desktop` y en el Escritorio.
* En macOS, genera el acceso directo ejecutable `~/Desktop/Packwire.command`.

> [!IMPORTANT]
> **Activación inmediata en terminal abierta:** Tras finalizar la ejecución de `install.sh`, recarga tu archivo de configuración de terminal para que el comando `packwire` esté disponible de inmediato:
> ```bash
> source ~/.bashrc   # En Linux / Bash
> source ~/.zshrc    # En macOS / Zsh
> ```

---

### Método 3: Instalación Estándar vía Python (`pip`)

Si prefieres gestionar la instalación manualmente o dentro de un entorno virtual (`venv` / `conda`):

1. **Instalar el paquete con pip:**
   ```bash
   # Modo normal
   pip install .

   # O en modo editable para desarrollo continuo:
   pip install -e .
   ```

2. **Ejecutar la inicialización e integración del sistema:**
   Invoca el comando `setup` para generar los shims maestros, registrar las rutas de entorno y configurar los accesos de escritorio:
   ```bash
   packwire setup --desktop
   ```

   Si tu terminal aún no tiene la ruta de scripts de Python en el `PATH`, puedes invocarlo directamente a través del intérprete:
   ```bash
   python -m packwire setup --desktop
   ```

#### Banderas disponibles en `packwire setup`:
* `--desktop`: Genera el acceso directo en el Escritorio.
* `--no-start-menu`: Omite la creación de entradas en el Menú Inicio / Lanzador de aplicaciones.
* `--no-path`: Omite la modificación de la variable de entorno `PATH`.

---

### Método 4: Asistente Gráfico de Windows (`Packwire-Setup.exe` Inno Setup)

Para usuarios finales o entornos corporativos donde no se dispone de Python preinstalado:

#### Opción A: Descargar el Binario Oficial Precompilado
1. Ve a la sección de **Releases** en GitHub: `https://github.com/GrandKenzy/packwire/releases`.
2. Descarga el instalador más reciente: `Packwire-Setup.exe` (o `Packwire-Setup-vX.Y.Z.exe`).
3. Ejecuta el archivo descargado y sigue las instrucciones del asistente en pantalla:
   * El asistente permite seleccionar idioma (Español o Inglés).
   * Se instala en `{localappdata}\Programs\Packwire` sin requerir derechos de administrador.
   * Ofrece casillas de verificación para crear el icono en el Escritorio y registrar el binario en el `PATH` del usuario.
   * Registra automáticamente la entrada de desinstalación en *Configuración > Aplicaciones instaladas*.

#### Opción B: Compilarlo tú mismo desde el código fuente
1. Compila el ejecutable independiente:
   ```bash
   packwire build --exe
   ```
2. Compila el instalador con Inno Setup (`iscc`):
   ```cmd
   iscc installer.iss
   ```
3. Ejecuta el archivo generado en `dist/Packwire-Setup.exe`.

---

### Método 5: Ejecutable Autónomo Portátil (`packwire.exe`)

Si necesitas un ejecutable que puedas llevar en una memoria USB o ejecutar sin asistente de instalación:

1. Compila el ejecutable con:
   ```bash
   packwire build --exe
   ```
2. El binario resultante se encuentra en `dist/packwire.exe` (peso optimizado de ~14.2 MB).
3. Puedes copiar `packwire.exe` a cualquier ubicación o carpeta en tu `PATH` y ejecutarlo directamente:
   ```cmd
   packwire.exe --help
   packwire.exe ui
   ```

---

## ✅ Verificación de la Instalación

Una vez instalado, abre una nueva ventana de terminal y comprueba que Packwire responde correctamente:

1. **Comprobar la versión y ayuda general:**
   ```bash
   packwire --help
   ```

2. **Comprobar la configuración del directorio de Shims y PATH:**
   ```bash
   packwire path
   ```
   Si la ruta no estuviese registrada en tu sesión actual, puedes forzar su registro con:
   ```bash
   packwire path --add
   ```

3. **Consultar el catálogo de paquetes disponibles:**
   ```bash
   packwire available
   ```

4. **Probar una instalación desatendida:**
   ```bash
   packwire install python@stable --mode here
   ```

5. **Lanzar la interfaz gráfica:**
   ```bash
   # Ventana nativa acelerada por hardware
   packwire ui

   # O en modo servidor web en tu navegador:
   packwire ui --web
   ```

---

## 🔄 Actualización de Packwire

* **Si instalaste en modo editable (`pip install -e .`):**
  Solo necesitas actualizar tu repositorio git:
  ```bash
  git pull origin main
  ```
* **Si instalaste como paquete normal:**
  ```bash
  pip install --upgrade .
  packwire setup
  ```
* **Para actualizar los paquetes instalados por Packwire (canal `stable`):**
  ```bash
  packwire update
  ```

---

## 🗑️ Desinstalación Completa y Limpieza

Packwire incluye mecanismos de desinstalación limpia y reversible:

### Desinstalación mediante CLI
Para eliminar completamente todos los paquetes descargados, los shims generados, la caché temporal y retirar la ruta del `PATH` del usuario:

```bash
packwire remove --all
```

Si deseas desinstalar los paquetes pero conservar la ruta de shims en el `PATH`:
```bash
packwire remove --all --keep-path
```

### Desinstalación del paquete de Python
```bash
pip uninstall packwire -y
```

### Rutas de datos residuales (limpieza manual si fuera necesaria):
* **En Windows:** Eliminar la carpeta `%APPDATA%\packwire` (usualmente `C:\Users\<usuario>\AppData\Roaming\packwire`).
* **En Linux y macOS:** Eliminar el directorio `~/.local/share/packwire`.

---

## 🛠️ Solución de Problemas Frecuentes (Troubleshooting)

### 1. "El término 'packwire' no se reconoce como un cmdlet, función..."
* **Causa:** La variable de entorno `PATH` fue modificada pero la terminal actual aún no ha recargado las variables de entorno.
* **Solución:**
  * **En Windows:** Cierra y vuelve a abrir tu terminal (PowerShell o CMD). Las variables modificadas en `HKCU\Environment` se cargarán en las nuevas ventanas.
  * **En Linux / macOS:** Ejecuta `source ~/.bashrc` o `source ~/.zshrc`.
  * Si persiste, ejecuta `python -m packwire path --add`.

### 2. Error de política de ejecución de scripts en PowerShell (`PSSecurityException`)
* **Causa:** PowerShell restringe la ejecución de scripts no firmados por defecto (`Restricted`).
* **Solución:** Ejecuta el script especificando bypass puntual:
  ```powershell
  powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -DesktopShortcut
  ```
  O utiliza directamente [`install.bat`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/install.bat), el cual aplica este bypass automáticamente.

### 3. "Python no fue encontrado en el sistema"
* **Causa:** Python no está instalado o no se marcó la opción *"Add python.exe to PATH"* durante su instalación.
* **Solución:** Instala Python 3.9 o superior desde [python.org](https://www.python.org/) y asegúrate de marcar la casilla de verificación para agregarlo al `PATH`.

### 4. Error al iniciar la ventana gráfica (`ERR_WEBVIEW_INIT`)
* **Causa:** En sistemas Windows mínimos (Windows Server, LTSC) puede faltar el runtime de Microsoft Edge WebView2. En Linux puede faltar `libwebkit2gtk`.
* **Solución:**
  * En Windows: Descarga e instala Microsoft Edge WebView2 Runtime (evergreen).
  * En Linux (Ubuntu/Debian): `sudo apt-get install libwebkit2gtk-4.0-37` (o `libwebkit2gtk-4.1-0`).
  * **Alternativa inmediata:** Inicia Packwire en modo servidor web sin requerir ventana nativa:
    ```bash
    packwire ui --web
    ```
