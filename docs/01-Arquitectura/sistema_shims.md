# Sistema de Shims y Gestión de PATH

El subsistema de shims es el componente central de integración entre los paquetes gestionados por Packwire y las terminales del sistema operativo (Windows, Linux y macOS). Se encuentra implementado en la clase `PatherVisitor` dentro de [`packwire/core/visitors/pather.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/visitors/pather.py) y se apoya en las constantes de ruta definidas en [`packwire/core/config.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/config.py).

Este diseño resuelve uno de los problemas históricos más graves en la administración de entornos de desarrollo: la degradación y saturación de la variable de entorno `PATH` por múltiples instaladores, colisiones de nombres binarios y desorden en el registro o perfiles de usuario.

---

## ⚙️ Especificación Técnica

### Firmas y Estructuras

```python
class PatherVisitor:
    def __init__(self, shims_dir: Optional[Path] = None) -> None: ...

    def create_shims(
        self,
        target_dir: Path,
        binaries: List[str],
        version_suffix: Optional[str] = None,
        is_default: bool = True
    ) -> List[str]: ...

    def remove_shims(self, shim_names: List[str]) -> None: ...

    def is_shims_in_path(self) -> bool: ...

    def add_shims_to_user_path(self) -> bool: ...

    def remove_shims_from_user_path(self) -> bool: ...
```

### Parámetros de `create_shims`

| Parámetro | Tipo | Requerido | Descripción |
| :--- | :--- | :--- | :--- |
| `target_dir` | `Path` | Sí | Directorio raíz donde reside la aplicación instalada. |
| `binaries` | `List[str]` | Sí | Lista de nombres de ejecutables a enlazar (ej. `["python.exe", "pip.exe"]` o `["git", "docker"]`). |
| `version_suffix` | `Optional[str]` | No | Sufijo numérico o alfanumérico para generar nombres alternativos con versión fija. |
| `is_default` | `bool` | No | Si es `True`, genera además el shim con el nombre base canónico (ej. `python.cmd` o `python`). |

* **Retorno:** `List[str]` con los nombres de todos los archivos shim efectivamente generados en disco.

---

## 🔍 Mecánica de Enlace y Proxying Multiplataforma

En lugar de agregar la carpeta de cada paquete instalado a la variable global `PATH`, Packwire inyecta una única ruta permanente en el perfil del usuario:

* **En Windows:** `%APPDATA%\packwire\shims`
* **En Linux y macOS:** `~/.local/share/packwire/shims`

Cuando un paquete como Python 3.14 es instalado, `PatherVisitor` no expone la ruta interna de extracción, sino que compila lanzadores proxy ligeros dentro de la carpeta de shims:

```
[Windows] %APPDATA%\packwire\shims\      [POSIX] ~/.local/share/packwire/shims/
├── python.cmd                           ├── python
├── python.ps1                           ├── python314
├── python314.cmd                        └── python-3.14
├── python314.ps1
├── python-3.14.cmd
└── python-3.14.ps1
```

### Contenido de los Shims Generados

#### 1. Lanzador para Consola de Comandos Windows (`.cmd`)
```batch
@echo off
"C:\Users\<Usuario>\AppData\Roaming\packwire\apps\python-3.14\python.exe" %*
```
El modificador `%*` propaga la totalidad de argumentos pasados en la terminal de forma íntegra hacia el binario destino.

#### 2. Lanzador para PowerShell (`.ps1`)
```powershell
& "C:\Users\<Usuario>\AppData\Roaming\packwire\apps\python-3.14\python.exe" $args
```
El operador de llamada `&` ejecuta el binario delegando el array de parámetros `$args`.

#### 3. Soporte Multiplataforma POSIX (Linux / macOS)
En sistemas Unix-like, se genera un script ejecutable sin extensión con directiva hashbang y permisos `0755` (`rwxr-xr-x`):
```sh
#!/bin/sh
exec "/home/usuario/.local/share/packwire/apps/python-3.14/bin/python" "$@"
```
`PatherVisitor` normaliza automáticamente los nombres eliminando extensiones `.exe` en Linux/macOS y aplicando bits de ejecución (`stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH`) tanto al binario destino como al shim generado.

---

## ⚙️ Registro y Persistencia en `PATH`

Para garantizar disponibilidad global sin requerir permisos de superusuario ni reiniciar la máquina, `PatherVisitor.add_shims_to_user_path()` implementa estrategias nativas por plataforma:

### 1. Entorno Windows: Registro y Difusión `WM_SETTINGCHANGE`
Accede mediante el módulo nativo `winreg` a:
```
HKEY_CURRENT_USER\Environment
```
Lee el valor `Path`, antepone `%APPDATA%\packwire\shims` al inicio de la cadena para asegurar precedencia y escribe el valor actualizado. Inmediatamente, notifica a los procesos del sistema mediante la Win32 API:

```python
HWND_BROADCAST = 0xFFFF
WM_SETTINGCHANGE = 0x001A
SMTO_ABORTIFHUNG = 0x0002
result = ctypes.c_long()

ctypes.windll.user32.SendMessageTimeoutW(
    HWND_BROADCAST,
    WM_SETTINGCHANGE,
    0,
    "Environment",
    SMTO_ABORTIFHUNG,
    5000,
    ctypes.byref(result)
)
```

### 2. Entorno POSIX (Linux y macOS): Perfiles de Shell
Examina de forma idempotente los archivos de inicialización del shell del usuario:
* `~/.zshrc` (macOS y distribuciones modernas con Zsh)
* `~/.bashrc` (distribuciones Linux estándar con Bash)
* `~/.profile` (sesiones POSIX genéricas)

Si la ruta no se encuentra presente, anexa la instrucción de exportación:
```bash
export PATH="/home/usuario/.local/share/packwire/shims:$PATH"
```
Al desinstalar (`remove_shims_from_user_path`), el sistema filtra y elimina de forma limpia dicha línea de los archivos de configuración sin alterar el resto del entorno.

---

## ⚠️ Consideraciones Críticas y Casos de Borde

1. **Políticas de Ejecución en PowerShell (`ExecutionPolicy`):** En entornos Windows con directiva `Restricted`, la invocación directa de `.ps1` puede verse bloqueada. `PatherVisitor` siempre genera en paralelo el archivo `.cmd`, el cual Windows ejecuta de forma prioritaria en terminales estándar sin restricciones de firma digital.
2. **Permisos de Ejecución en Archivos POSIX:** Al extraer archivos `.zip` o `.tar.gz` en Linux/macOS, los binarios pueden perder los atributos de ejecución. `PatherVisitor` analiza los ejecutables destino y aplica forzosamente `chmod +x` antes de vincular el shim.
3. **Resolución Binaria en Carpetas `bin/`:** Si el archivo ejecutable no reside directamente en la raíz del paquete sino dentro de un subdirectorio `bin/` (patrón común en GCC, Node.js y Git), el visitante realiza una búsqueda recursiva para localizar el binario real antes de generar el shim.
4. **Desinstalación y Limpieza:** Al invocar `remove_shims()`, el sistema elimina todos los alias derivados (`.cmd`, `.ps1`, variantes con versión y shims POSIX sin extensión) asegurando que no queden comandos huérfanos.

---

## 💡 Ejemplo de Uso

```python
from pathlib import Path
from packwire.core.visitors.pather import PatherVisitor

# Instanciar el gestor de shims
pather = PatherVisitor()

# Verificar y asegurar que los shims estén en el PATH del sistema
if not pather.is_shims_in_path():
    print("[*] Registrando shims en PATH del usuario...")
    pather.add_shims_to_user_path()

# Generar shims para una instalación de herramientas de compilación
bin_dir = Path("/opt/packwire/apps/c-gcc/bin")
shims = pather.create_shims(
    target_dir=bin_dir,
    binaries=["gcc", "g++", "make"],
    version_suffix="14.2.0",
    is_default=True
)

print(f"Shims compilados ({len(shims)}): {', '.join(shims)}")
```
