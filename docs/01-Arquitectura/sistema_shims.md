# Sistema de Shims y Gestión de PATH

El subsistema de shims es el componente central de integración entre los paquetes gestionados por Packwire y la consola de comandos de Windows. Se encuentra implementado en la clase `PatherVisitor` dentro de [`packwire/core/visitors/pather.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/visitors/pather.py) y se apoya en las constantes de ruta definidas en [`packwire/core/config.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/config.py).

Este diseño resuelve uno de los problemas históricos más graves en la administración de entornos de desarrollo en Windows: la degradación y corrupción de la variable de entorno `PATH` por saturación de entradas de diferentes instaladores, colisiones de nombres binarios y el límite de longitud en el registro de Windows.

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
| `binaries` | `List[str]` | Sí | Lista de nombres de ejecutables a enlazar (ej. `["python.exe", "pip.exe"]`). |
| `version_suffix` | `Optional[str]` | No | Sufijo numérico o alfanumérico para generar nombres alternativos con versión fija. |
| `is_default` | `bool` | No | Si es `True`, genera además el shim con el nombre base canónico (ej. `python.cmd`). |

* **Retorno:** `List[str]` con los nombres de todos los archivos shim efectivamente generados en disco.

---

## 🔍 Mecánica de Enlace y Proxying

En lugar de agregar la carpeta de cada paquete instalado a la variable global `PATH`, Packwire inyecta una única ruta permanente en el perfil del usuario:

```
%APPDATA%\packwire\shims
```

Cuando un paquete como Python 3.14 es instalado, `PatherVisitor` no expone `%APPDATA%\packwire\apps\python-3.14\`, sino que compila lanzadores proxy ligeros dentro de la carpeta de shims:

```
%APPDATA%\packwire\shims\
├── python.cmd
├── python.ps1
├── python314.cmd
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

#### 3. Soporte Multiplataforma (POSIX / Linux / macOS)
En sistemas Unix-like, se genera un script ejecutable sin extensión con directiva hashbang y permisos `0755` (`rwxr-xr-x`):
```sh
#!/bin/sh
exec "/home/usuario/.local/share/packwire/apps/python-3.14/bin/python" "$@"
```

---

## ⚙️ Inyección en Registro y Difusión Windows

Para registrar la carpeta de shims sin requerir permisos de Administrador ni reiniciar el equipo, `PatherVisitor.add_shims_to_user_path()` ejecuta un protocolo en dos fases:

### 1. Manipulación del Registro de Usuario
Accede mediante el módulo nativo `winreg` a la clave:
```
HKEY_CURRENT_USER\Environment
```
Lee el valor de tipo `REG_EXPAND_SZ` o `REG_SZ` denominado `Path`, normaliza la lista de rutas separadas por punto y coma (`;`), antepone `%APPDATA%\packwire\shims` al inicio de la cadena para asegurar máxima precedencia y escribe el valor actualizado.

### 2. Difusión de Mensaje del Sistema (`WM_SETTINGCHANGE`)
Para que los procesos nuevos y el shell de Windows Explorer reconozcan la modificación sin reiniciar la sesión, se realiza una invocación a la API nativa de Win32 mediante `ctypes`:

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

---

## ⚠️ Consideraciones Críticas y Casos de Borde

1. **Políticas de Ejecución en PowerShell (`ExecutionPolicy`):** En Windows donde la política sea `Restricted`, la invocación de `packwire.ps1` puede verse rechazada. Por este motivo, `PatherVisitor` siempre genera en paralelo el archivo `.cmd`, el cual Windows ejecuta de forma prioritaria en terminales estándar de comandos sin restricciones de firma digital.
2. **Duplicación de Rutas en el Registro:** El método analiza la cadena existente con comparaciones de rutas absolutas normalizadas en minúsculas para prevenir entradas repetidas.
3. **Desinstalación y Limpieza:** Al invocar `remove_shims()`, el sistema elimina todos los alias derivados (`.cmd`, `.ps1` y variantes con versión) asegurando que no queden comandos huérfanos que apunten a ejecutables eliminados.

---

## 💡 Ejemplo de Uso

```python
from pathlib import Path
from packwire.core.visitors.pather import PatherVisitor

# Instanciar el gestor de shims
pather = PatherVisitor()

# Verificar estado en el entorno actual
if not pather.is_shims_in_path():
    print("[*] Registrando shims en PATH del usuario...")
    pather.add_shims_to_user_path()

# Generar shims para una instalación manual de GCC
bin_dir = Path("C:/herramientas/w64devkit/bin")
shims = pather.create_shims(
    target_dir=bin_dir,
    binaries=["gcc.exe", "g++.exe", "make.exe"],
    version_suffix="14.2.0",
    is_default=True
)

print(f"Shims compilados ({len(shims)}): {', '.join(shims)}")
```
