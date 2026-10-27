# Subsistema de Visitantes (Visitor Pattern)

El subsistema de visitantes agrupa la lógica operativa y algorítmica de Packwire, separándola estrictamente de los modelos de datos pasivos (`Manifest` y `PackageState`). Reside en [`packwire/core/visitors/`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/visitors) y comprende cinco componentes especializados: `DownloaderVisitor`, `PatherVisitor`, `ManifestVisitor`, `UninstallerVisitor` y `UpdaterVisitor`.

Esta separación garantiza alta cohesión, facilita la realización de pruebas unitarias aisladas mediante mocks y permite sustituir o ampliar cualquier fase del pipeline sin afectar a los demás componentes.

---

## ⚙️ Especificación Técnica por Visitante

### 1. `DownloaderVisitor` ([`downloader.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/visitors/downloader.py))
Responsable de la transferencia de datos por red, almacenamiento en caché y validación de sumas criptográficas.

```python
class DownloaderVisitor:
    def __init__(self, cache_dir: Optional[Path] = None) -> None: ...

    def download(
        self,
        url: str,
        dest_filename: Optional[str] = None,
        expected_hash: Optional[str] = None,
        progress_callback: Optional[Callable[[int, int, float], None]] = None,
        force: bool = False
    ) -> Path: ...
```

* **Flujo Atómico:** Escribe sobre `<nombre>.part` en bloques de 64 KB calculando velocidad instantánea de transferencia en bytes por segundo (`speed_bps`).
* **Verificación de Integridad:** Si se provee `expected_hash`, calcula el SHA-256 del archivo completo y lo compara antes de renombrarlo; si falla, el archivo temporal es purgado de inmediato arrojando `ValueError`.

---

### 2. `PatherVisitor` ([`pather.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/visitors/pather.py))
Responsable de compilar los lanzadores intermediarios (shims) y registrar el directorio en las variables de entorno de Windows.

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

* **Gestión en Windows:** Escribe en `HKCU\Environment` e invoca `SendMessageTimeoutW` para notificar al shell (`WM_SETTINGCHANGE`) sin exigir reinicios de sesión.

---

### 3. `ManifestVisitor` ([`process_manifest.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/visitors/process_manifest.py))
Analizador sintáctico, validador de esquemas y motor de descubrimiento de manifiestos.

```python
class ManifestVisitor:
    def __init__(self, manifests_dir: Optional[Path] = None) -> None: ...

    def load_manifest(self, filepath: Path) -> Optional[Manifest]: ...
    def find_manifest(self, identifier: str) -> Optional[Manifest]: ...
    def list_available_manifests(self) -> List[Manifest]: ...
```

* **Tolerancia a Formatos:** Resuelve identificadores buscando coincidencias exactas por `id`, por alias (`aliases`) o por nombre de paquete (`name`), soportando comodines y sufijos de canal (ej. `python@stable`, `c`, `gcc`, `w64devkit`).
* **Soporte Frozen:** Detecta automáticamente si la aplicación se ejecuta desde un ejecutable empaquetado mediante `sys._MEIPASS`.

---

### 4. `UninstallerVisitor` ([`uninstaller.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/visitors/uninstaller.py))
Ejecutor de operaciones de remoción ordenada y limpieza profunda del sistema.

```python
class UninstallerVisitor:
    def __init__(self, pather: Optional[PatherVisitor] = None) -> None: ...

    def uninstall(self, pkg_id: str) -> Tuple[bool, str]: ...
    def uninstall_all(self, remove_path: bool = True) -> Tuple[bool, str]: ...
```

* **Remoción Individual (`uninstall`):** Localiza la entrada en `state.json`, elimina los shims registrados mediante `pather.remove_shims()`, purga la carpeta del paquete en `%APPDATA%\packwire\apps\` y da de baja el registro de forma atómica.
* **Desinstalación Total (`uninstall_all`):** Purga la totalidad de aplicaciones, vacía la carpeta de shims, vacía la memoria caché de instaladores, elimina `state.json` y opcionalmente (`remove_path=True`) remueve la entrada de shims de la variable `PATH` del usuario en el registro de Windows.

---

### 5. `UpdaterVisitor` ([`updater.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/visitors/updater.py))
Motor de detección de desviaciones y actualización automática de canales continuos.

```python
class UpdaterVisitor:
    def __init__(
        self,
        manifest_visitor: Optional[ManifestVisitor] = None,
        installer: Optional[Installer] = None,
        uninstaller: Optional[UninstallerVisitor] = None
    ) -> None: ...

    def check_updates(self) -> List[Tuple[PackageState, str]]: ...
    def update(self, pkg_id: Optional[str] = None) -> List[Tuple[str, bool, str]]: ...
```

* **Detección de Nuevas Versiones:** Itera sobre todos los paquetes registrados cuyo `type != "fixed"`, invoca la resolución upstream y compara si la versión disponible difiere de la instalada.
* **Actualización en Caliente:** Para cada paquete desactualizado, orquesta la instalación con bandera `force=True` sobreescribiendo limpiamente los binarios anteriores y regenerando los shims hacia la nueva versión.

---

## ⚠️ Consideraciones Críticas y Casos de Borde

1. **Bloqueo por Procesos Activos Durante Desinstalación:** Si un ejecutable del paquete está siendo utilizado por el usuario, el sistema de archivos de Windows arroja `PermissionError`. `UninstallerVisitor` captura la excepción e informa con precisión qué ruta se encuentra bloqueada, sugiriendo cerrar los procesos abiertos.
2. **Reversibilidad Garantizada:** Si la desinstalación falla a mitad del proceso, el estado no se corrompe gracias a que `state.json` solo se reescribe cuando los archivos físicos han sido satisfactoriamente eliminados.

---

## 💡 Ejemplo de Uso Integral

```python
from packwire.core.visitors.process_manifest import ManifestVisitor
from packwire.core.visitors.uninstaller import UninstallerVisitor
from packwire.core.visitors.updater import UpdaterVisitor

# 1. Búsqueda de un paquete en el catálogo
mv = ManifestVisitor()
m = mv.find_manifest("c")
print(f"Manifiesto encontrado: {m.id} ({m.name}) v{m.version}")

# 2. Comprobar actualizaciones de software instalado
uv = UpdaterVisitor(manifest_visitor=mv)
actualizaciones = uv.check_updates()
for pkg, nueva_ver in actualizaciones:
    print(f"Actualización disponible para {pkg.name}: {pkg.version} -> {nueva_ver}")

# 3. Aplicar actualizaciones pendientes
resultados = uv.update()
for id_pkg, ok, msg in resultados:
    print(f"Resultado {id_pkg}: {'OK' if ok else 'ERROR'} - {msg}")
```
