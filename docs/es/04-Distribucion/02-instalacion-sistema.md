# Integración y Auto-Instalación en el Sistema

El subsistema de auto-instalación provee las herramientas necesarias para configurar Packwire como un comando de primer orden en el sistema operativo del usuario (Windows, Linux y macOS). Se implementa en [`packwire/installer_self.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/installer_self.py) y se expone tanto en la CLI (`packwire setup`) como a través de los scripts de arranque rápido [`install.bat`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/install.bat), [`install.ps1`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/install.ps1) e [`install.sh`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/install.sh).

A diferencia de los instaladores tradicionales que requieren derechos administrativos obligatorios para registrar binarios, la integración de Packwire opera íntegramente dentro del espacio del usuario actual (`HKCU` y `%APPDATA%` en Windows; `~/.local/share/packwire` y archivos de configuración de shell en Linux y macOS), evitando solicitar permisos de elevación para la operativa estándar del gestor.

> [!TIP]
> Si buscas instrucciones prácticas de instalación paso a paso para usuarios finales, consulta la **[Guía Completa de Instalación](guia_instalacion.md)**. Este documento detalla la especificación interna y arquitectura de `installer_self.py`.

---

## ⚙️ Especificación Técnica

### Firmas y Estructuras

```python
def create_windows_shortcut(
    target_path: Path,
    shortcut_path: Path,
    arguments: str = "",
    icon_path: Optional[Path] = None,
    description: str = "Packwire - Gestor de Paquetes"
) -> bool: ...

def install_self(
    add_to_path: bool = True,
    create_start_menu: bool = True,
    create_desktop: bool = False
) -> Tuple[bool, str]: ...
```

### Parámetros de `install_self`

| Parámetro | Tipo | Requerido | Descripción |
| :--- | :--- | :--- | :--- |
| `add_to_path` | `bool` | No | Si es `True`, inyecta la carpeta de shims en la variable `PATH` del usuario (registro en Windows o `.bashrc`/`.zshrc` en POSIX). |
| `create_start_menu`| `bool` | No | Si es `True`, genera el acceso en el Menú Inicio (Windows) o la entrada `.desktop` en el menú de aplicaciones del sistema (Linux). |
| `create_desktop` | `bool` | No | Si es `True`, crea un acceso directo de la interfaz gráfica en el Escritorio del usuario. |

* **Retorno:** Tupla `Tuple[bool, str]` confirmando el estado de la integración y el desglose de acciones efectuadas.

---

## 🔍 Fases de la Integración Multiplataforma

### 1. Compilación de Shims Maestros
Genera los archivos de enlace global en el directorio central de shims (`%APPDATA%\packwire\shims` en Windows, `~/.local/share/packwire/shims` en Linux/macOS):

* **En Windows:**
  * `packwire.cmd`: Envoltorio para consola CMD estándar.
  * `packwire.ps1`: Envoltorio para PowerShell.
  Si Packwire opera desde código fuente o paquete editable, los shims invocan `python.exe -m packwire %*`. Si se ejecuta desde un binario standalone congelado, apuntan directamente a la ruta física de `packwire.exe`.

* **En Linux y macOS:**
  * `packwire`: Script POSIX sin extensión, dotado de permisos de ejecución `0755` (`rwxr-xr-x`):
    ```sh
    #!/bin/sh
    exec "/ruta/al/python" -m packwire "$@"
    ```

### 2. Registro en la Variable `PATH` del Usuario
Invoca `PatherVisitor.add_shims_to_user_path()`, adaptando la persistencia según la arquitectura anfitriona:
* **En Windows:** Inserta la carpeta de shims en `HKEY_CURRENT_USER\Environment\Path` y difunde el mensaje de sistema `WM_SETTINGCHANGE` mediante la API nativa de Win32 (`SendMessageTimeoutW`).
* **En Linux y macOS:** Examina los archivos de perfil del usuario (`~/.zshrc`, `~/.bashrc`, `~/.profile`) y agrega la directiva:
  ```bash
  export PATH="$HOME/.local/share/packwire/shims:$PATH"
  ```
  asegurando precedencia inmediata en nuevas sesiones de terminal.

### 3. Creación de Accesos Directos e Integración Gráfica
Para proveer acceso a la interfaz gráfica sin terminal de fondo:

* **En Windows (`pythonw.exe`):**
  * **Menú Inicio:** `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Packwire.lnk`
  * **Escritorio:** `%USERPROFILE%\Desktop\Packwire.lnk`
  * En entornos Python estándar enlaza hacia `pythonw.exe -m packwire ui`, impidiendo que se despliegue la ventana negra de consola detrás del dashboard visual. Se asocia con el icono binario `logo.ico`.

* **En Linux (Especificación XDG Desktop Entry):**
  * **Lanzador de Aplicaciones:** `~/.local/share/applications/packwire.desktop`
  * **Escritorio:** `~/Desktop/packwire.desktop` (con permisos de ejecución `0755`)
  * Estructura generada:
    ```ini
    [Desktop Entry]
    Type=Application
    Name=Packwire
    Comment=Packwire Package Manager
    Exec=/usr/bin/python3 -m packwire ui
    Icon=/ruta/a/packwire/core/gui/web/logo.png
    Terminal=false
    Categories=Development;System;
    ```

* **En macOS (Acceso Directo `.command`):**
  * **Escritorio:** `~/Desktop/Packwire.command` (con permisos `0755`)
  * Estructura generada:
    ```sh
    #!/bin/sh
    exec "/usr/local/bin/python3" -m packwire ui
    ```

---

## 📦 Scripts de Aprovisionamiento Rápido

### `install.bat` (Instalador de Un Clic para Windows)
Permite instalar Packwire simplemente haciendo doble clic desde el Explorador de Archivos de Windows:
1. Verifica si `python` está presente en el `PATH` del sistema.
2. Invoca `install.ps1` con la bandera de bypass de políticas de ejecución:
   ```cmd
   powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1" -DesktopShortcut %*
   ```
3. Si detecta que fue ejecutado por doble clic interactivo, realiza una pausa (`pause`) al culminar para permitir leer el reporte de instalación.

### `install.ps1` (Script Automatizado de PowerShell)
Automatiza la instalación de dependencias en modo editable (`pip install -e .`) e invoca `python -m packwire setup` con soporte para banderas de personalización:
* `-DesktopShortcut`: Crea el icono en el escritorio.
* `-NoStartMenu`: Omite el acceso en el menú de programas.
* `-NoPath`: Omite la modificación del `PATH`.

### `install.sh` (Script Automatizado POSIX para Linux y macOS)
Permite la instalación integral en sistemas Unix con un solo comando:
1. Detecta automáticamente la presencia de `python3` o `python` y el gestor de paquetes `pip`.
2. Actualiza `pip` e instala el proyecto en modo editable (`pip install -e .`).
3. Invoca `python3 -m packwire setup` propagando las opciones `--desktop`, `--no-start-menu` o `--no-path`.
4. Informa la ruta de los shims y confirma la activación inmediata de la CLI.

---

## ⚠️ Consideraciones Críticas y Casos de Borde

1. **Restricción de Scripting en PowerShell:** Muchos equipos Windows bloquean la ejecución de scripts `.ps1` mediante la directiva `Restricted`. Tanto `install.bat` como el creador de accesos directos ejecutan PowerShell con `-NoProfile -ExecutionPolicy Bypass`, garantizando que la instalación proceda sin interrupciones por directivas de seguridad locales.
2. **Generación Dinámica del Icono:** Si el archivo binario `logo.ico` no se encuentra en el repositorio, `packwire.packager.ensure_icon()` lo sintetiza dinámicamente a partir del archivo PNG embebido sin necesidad de librerías externas como Pillow.
3. **Persistencia en Shells POSIX:** En Linux y macOS, si el usuario utiliza shells no tradicionales (como `fish`), la entrada en `~/.bashrc` o `~/.zshrc` puede requerir declarar manualmente `set -gx PATH "$HOME/.local/share/packwire/shims" $PATH`.

---

## 💡 Ejemplo de Uso en Terminal

```bash
# Configuración estándar en Windows, Linux o macOS con acceso en el escritorio
packwire setup --desktop

# Configuración mínima solo para terminal (sin accesos directos de menú)
packwire setup --no-start-menu

# Ejecución del instalador universal en Linux / macOS
chmod +x install.sh
./install.sh --desktop
```
