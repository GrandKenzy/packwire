# Integración y Auto-Instalación en el Sistema

El subsistema de auto-instalación provee las herramientas necesarias para configurar Packwire como un comando de primer orden en el sistema operativo Windows. Se implementa en [`packwire/installer_self.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/installer_self.py) y se expone tanto en la CLI (`packwire setup`) como a través de los scripts de arranque rápido [`install.bat`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/install.bat) y [`install.ps1`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/install.ps1).

A diferencia de los instaladores tradicionales que requieren derechos administrativos obligatorios, la integración de Packwire opera íntegramente dentro del espacio del usuario actual (`HKCU` y `%APPDATA%`), evitando solicitar permisos de elevación UAC para la operativa ordinaria.

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
| `add_to_path` | `bool` | No | Si es `True`, inyecta la carpeta de shims en la variable `PATH` de usuario en el registro. |
| `create_start_menu`| `bool` | No | Si es `True`, genera el acceso directo en el Menú Inicio del usuario. |
| `create_desktop` | `bool` | No | Si es `True`, crea un acceso directo de la interfaz gráfica en el Escritorio. |

* **Retorno:** Tupla `Tuple[bool, str]` confirmando el estado de la integración y el desglose de acciones efectuadas.

---

## 🔍 Fases de la Integración

### 1. Compilación de Shims Maestros
Genera los archivos de enlace global en `%APPDATA%\packwire\shims`:
* `packwire.cmd`: Envoltorio para CMD.
* `packwire.ps1`: Envoltorio para PowerShell.

Si Packwire se ejecuta desde código fuente o paquete pip, los shims invocan `python.exe -m packwire %*`. Si se ejecuta desde un binario standalone compilado, apuntan directamente a la ruta física de `packwire.exe`.

### 2. Registro en la Variable `PATH` del Usuario
Invoca `PatherVisitor.add_shims_to_user_path()`, insertando la carpeta de shims en `HKEY_CURRENT_USER\Environment\Path` y difundiendo el mensaje de sistema `WM_SETTINGCHANGE` para que cualquier nueva terminal reconozca el comando `packwire` de inmediato.

### 3. Creación de Accesos Directos Silenciosos (`pythonw.exe`)
Para los accesos directos de la interfaz gráfica en el **Menú Inicio** (`%APPDATA%\Microsoft\Windows\Start Menu\Programs\Packwire.lnk`) y en el **Escritorio**, el sistema analiza el entorno:
* En entornos Python no congelados, el acceso directo enlaza a:
  ```
  Target:    C:\...\Python314\pythonw.exe
  Arguments: -m packwire ui
  ```
  El uso de `pythonw.exe` es crítico: ejecuta la interfaz gráfica sin desplegar la clásica ventana negra de consola detrás del dashboard.
* Se adjunta el icono oficial extraído o compilado en `logo.ico`.

---

## 📦 Scripts de Aprovisionamiento Rápido

### `install.bat` (Instalador de Un Clic)
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

---

## ⚠️ Consideraciones Críticas y Casos de Borde

1. **Restricción de Scripting en PowerShell:** Muchos equipos Windows bloquean la ejecución de scripts `.ps1` mediante la directiva `Restricted`. Tanto `install.bat` como el creador de accesos directos ejecutan PowerShell con `-NoProfile -ExecutionPolicy Bypass`, garantizando que la instalación proceda sin interrupciones por directivas de seguridad locales.
2. **Generación Automática del Icono ICO:** Si el archivo binario `logo.ico` no se encuentra en el repositorio, `packwire.packager.ensure_icon()` lo sintetiza dinámicamente a partir del archivo PNG embebido sin necesidad de librerías externas como Pillow.

---

## 💡 Ejemplo de Uso en Terminal

```bash
# Configuración estándar con acceso en el escritorio
packwire setup --desktop

# Configuración mínima solo para terminal (sin accesos directos)
packwire setup --no-start-menu
```
