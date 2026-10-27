# Flujo de Ejecución y Ciclo de Vida

El flujo de ejecución de Packwire define el ciclo de vida por el cual una petición de instalación, actualización o reinstalación es procesada por el motor. Este subsistema reside en [`packwire/core/installer.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/installer.py) bajo la coordinación de la clase `Installer` y la función pública `packwire.install()` expuesta en [`packwire/__init__.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/__init__.py).

El diseño garantiza invariantes transaccionales estrictos: si una fase intermedia falla (por ejemplo, una descarga incompleta o un binario corrupto), el sistema aborta la operación sin registrar un estado inconsistente en `state.json` ni generar enlaces de acceso residuales en el directorio de shims.

---

## ⚙️ Especificación Técnica

### Firmas y Estructuras Principales

```python
class Installer:
    def __init__(
        self,
        downloader: Optional[DownloaderVisitor] = None,
        pather: Optional[PatherVisitor] = None,
        manifest_visitor: Optional[ManifestVisitor] = None
    ) -> None: ...

    def install(
        self,
        manifest: Manifest,
        mode_override: Optional[str] = None,
        force: bool = False,
        logger: Optional[Callable[[str], None]] = None
    ) -> Tuple[bool, str]: ...

    def install_here(
        self,
        manifest: Manifest,
        force: bool = False,
        logger: Optional[Callable[[str], None]] = None
    ) -> Tuple[bool, str]: ...

    def install_command(
        self,
        manifest: Manifest,
        logger: Optional[Callable[[str], None]] = None
    ) -> Tuple[bool, str]: ...

    def install_site(
        self,
        manifest: Manifest,
        logger: Optional[Callable[[str], None]] = None
    ) -> Tuple[bool, str]: ...
```

### Parámetros y Retorno del Método `install`

| Parámetro | Tipo | Requerido | Descripción |
| :--- | :--- | :--- | :--- |
| `manifest` | `Manifest` | Sí | Estructura inmutable con los metadatos y directivas de despliegue del paquete. |
| `mode_override` | `Optional[str]` | No | Permite forzar el modo (`"here"`, `"command"`, `"site"`), ignorando el predeterminado del manifiesto. |
| `force` | `bool` | No | Si es `True`, ignora comprobaciones de colisión y sobrescribe archivos preexistentes. |
| `logger` | `Optional[Callable[[str], None]]` | No | Delegado de telemetría invocado para registrar mensajes de progreso en tiempo real. |

* **Retorno:** Tupla `Tuple[bool, str]` donde el primer elemento representa el éxito booleano de la operación y el segundo contiene una descripción técnica del resultado o el motivo del rechazo.

---

## 🔍 Fases Detalladas del Ciclo de Vida

```
  ┌────────────────────────────────────────────────────────┐
  │ 1. Normalización de Identificador y Selección de Canal │
  └───────────────────────────┬────────────────────────────┘
                              ▼
  ┌────────────────────────────────────────────────────────┐
  │ 2. Resolución Dinámica de Versión y Upstream           │
  └───────────────────────────┬────────────────────────────┘
                              ▼
  ┌────────────────────────────────────────────────────────┐
  │ 3. Detección de Colisiones en state.json               │
  └───────────────────────────┬────────────────────────────┘
                              ▼
  ┌────────────────────────────────────────────────────────┐
  │ 4. Adquisición y Verificación de Integridad            │
  └───────────────────────────┬────────────────────────────┘
                              ▼
  ┌────────────────────────────────────────────────────────┐
  │ 5. Despliegue de Binarios e Inyección de Shims         │
  └───────────────────────────┬────────────────────────────┘
                              ▼
  ┌────────────────────────────────────────────────────────┐
  │ 6. Registro Transaccional en Base de Estado            │
  └────────────────────────────────────────────────────────┘
```

### 1. Normalización del Identificador
El usuario puede solicitar objetivos flexibles como `python`, `python@stable` o `python@3.14`. La función `install()` descompone el término:
- Si incluye `@`, consulta directamente el catálogo de manifiestos mediante `ManifestVisitor.find_manifest()`.
- Si no incluye versión explícita y se especifica canal `stable`, compone `f"{target}@stable"` y ejecuta la resolución dinámica.

### 2. Resolución Dinámica
Cuando el manifiesto es de tipo `stable`, se delega a `packwire.core.resolver.resolve_manifest()`. El resolvedor consulta la API remota (ej. `python.org/api/v2/downloads/release/`), obtiene la última versión liberada, inyecta la URL de descarga del paquete zip portable o instalador y actualiza la propiedad `version` y el hash canónico del manifiesto.

### 3. Detección de Colisiones
Se invoca `packwire.core.state.check_collision(manifest)`. Si ya existe una versión diferente instalada con el mismo nombre y el manifiesto no provee sufijos de versión o no se activó la bandera `force=True`, se cancela el proceso retornando un mensaje descriptivo para evitar que un binario sobrescriba al otro de forma no deseada.

### 4. Adquisición y Verificación de Integridad
En el modo `here`, se invoca `DownloaderVisitor.download()`:
- Se genera un archivo temporal `.part` en `%APPDATA%/packwire/cache`.
- Si se definió una suma SHA-256 esperada en el manifiesto, se calcula y compara el hash del archivo descargado; ante cualquier discrepancia, el archivo corrupto es eliminado inmediatamente.
- Una vez completada la transferencia al 100%, se efectúa un renombramiento atómico (`replace`) del archivo final.

### 5. Despliegue e Inyección de Shims
Los archivos son extraídos en `%APPDATA%/packwire/apps/<nombre_paquete>/`. Posteriormente, `PatherVisitor.create_shims()` rastrea recursivamente los ejecutables declarados en la propiedad `binaries` del manifiesto (ej. `python.exe`, `gcc.exe`, `node.exe`) y genera los envoltorios `.cmd` y `.ps1` correspondientes en `%APPDATA%/packwire/shims`.

### 6. Registro Transaccional
Se construye una instancia de `PackageState` que encapsula la versión instalada, los shims creados, la ruta física en disco y el hash canónico del manifiesto. Esta estructura se guarda de forma atómica en `state.json` mediante `register_installed()`.

---

## ⚠️ Consideraciones Críticas y Casos de Borde

1. **Interrupción Abrupta de Descargas:** Las descargas parciales nunca permanecen con su extensión definitiva. El uso del sufijo `.part` previene que una ejecución interrumpida sea interpretada por futuras invocaciones como un archivo válido en caché.
2. **Archivos Bloqueados en Windows (Error 32 - `ERROR_SHARING_VIOLATION`):** Si un proceso en ejecución (como una instancia de Python o GCC en terminal) mantiene abierto un ejecutable dentro de `%APPDATA%/packwire/apps/`, la reinstalación detectará la imposibilidad de sobrescribir el archivo y emitirá un error controlado solicitando cerrar las consolas activas.
3. **Paquetes sin Binarios Locales (Modo `site`):** En este caso no se realiza descarga ni se generan shims; el sistema registra la redirección web en el estado para auditoría y abre el navegador por defecto del sistema mediante el módulo estándar `webbrowser`.

---

## 💡 Ejemplo de Uso Programático

```python
import packwire
from packwire.core.models import Manifest, InstallConfig, HereConfig

# Definición manual de un manifiesto personalizado para una herramienta interna
custom_tool = Manifest(
    id="mytool@1.0.0",
    name="MyTool",
    type="fixed",
    version="1.0.0",
    description="Herramienta de desarrollo propia",
    install=InstallConfig(
        mode="here",
        clean=True,
        here=HereConfig(
            url="https://internal.corp/downloads/mytool-1.0.0.zip",
            format="zip",
            binaries=["mytool.exe"],
            addpath=True
        )
    )
)

# Ejecución de instalación capturando telemetría
def auditor(mensaje: str):
    print(f"[AUDITORIA] {mensaje}")

exito, resultado = packwire._installer.install(custom_tool, logger=auditor)
print(f"Estado: {exito} | Detalle: {resultado}")
```
