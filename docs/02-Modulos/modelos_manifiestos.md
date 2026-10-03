# Modelos de Datos y Especificación de Manifiestos

El subsistema de modelos define las estructuras de datos canónicas utilizadas en todo el ecosistema de Packwire. Reside en [`packwire/core/models.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/models.py) y está construido sobre dataclasses tipadas de Python (`dataclass`).

El propósito de estos contratos es proveer un esquema inmutable y predecible tanto para la ingestión de archivos JSON de configuración como para el almacenamiento del estado transaccional de las aplicaciones instaladas en disco.

---

## ⚙️ Especificación Técnica

### Firmas y Estructuras

```python
@dataclass
class HereConfig:
    url: Optional[str] = None
    format: str = "zip"
    binaries: List[str] = field(default_factory=lambda: ["python.exe"])
    addpath: bool = True
    silent_args: List[str] = field(default_factory=list)

@dataclass
class SiteConfig:
    url: str

@dataclass
class CommandConfig:
    script: Optional[str] = None
    windows: Optional[str] = None
    linux: Optional[str] = None
    darwin: Optional[str] = None
    shell: str = "powershell"
    elevated: bool = False
    binaries: List[str] = field(default_factory=list)

@dataclass
class InstallConfig:
    mode: str = "here"  # "here", "site", "command", "multi_os"
    clean: bool = True
    here: Optional[HereConfig] = None
    site: Optional[SiteConfig] = None
    command: Optional[CommandConfig] = None
    os: Optional[Dict[str, Any]] = None

@dataclass
class Manifest:
    id: str
    name: str
    type: str  # "stable" o "fixed"
    version: Optional[str] = None
    icon: str = "📦"
    category: str = "general"
    description: str = ""
    aliases: List[str] = field(default_factory=list)
    upstream: Optional[Dict[str, Any]] = None
    install: InstallConfig = field(default_factory=InstallConfig)
    manifest_hash: Optional[str] = None

    def calculate_hash(self) -> str: ...
    def to_dict(self) -> Dict[str, Any]: ...

@dataclass
class PackageState:
    id: str
    name: str
    version: str
    type: str
    mode: str
    install_path: Optional[str] = None
    binaries: List[str] = field(default_factory=list)
    shims: List[str] = field(default_factory=list)
    installed_at: str = ""
    manifest_hash: str = ""

    def to_dict(self) -> Dict[str, Any]: ...
```

---

## 🔍 Atributos del Manifiesto

| Campo | Tipo | Requerido | Descripción |
| :--- | :--- | :--- | :--- |
| `id` | `str` | Sí | Identificador semántico canónico único (ej. `python@stable`, `c-gcc`, `nodejs@stable`). |
| `name` | `str` | Sí | Nombre representativo del paquete. |
| `type` | `str` | Sí | Estrategia de versión: `"stable"` (dinámica por upstream) o `"fixed"` (anclada a versión fija). |
| `version` | `Optional[str]` | Condicional | Versión exacta (obligatoria en manifests de tipo `"fixed"`). |
| `icon` | `str` | No | Icono emoji o URL de imagen para el dashboard visual. |
| `category` | `str` | No | Agrupador funcional (`"programming languages"`, `"compilers"`, `"multimedia"`, etc.). |
| `aliases` | `List[str]` | No | Nombres cortos alternativos para invocación en CLI (ej. `["c", "gcc", "w64devkit"]`). |
| `install` | `InstallConfig` | Sí | Especificación de los parámetros de despliegue según el modo seleccionado. |
| `manifest_hash`| `Optional[str]` | Automático | Suma SHA-256 calculada sobre los campos normalizados del manifiesto. |

---

## 🔒 Algoritmo Determinista de Hash Canónico (`calculate_hash`)

Para garantizar que dos manifiestos con ligeras variaciones de formato o campos no esenciales (como espacios o descripción) no produzcan identidades inconsistentes, `Manifest.calculate_hash()` normaliza los campos determinantes antes de invocar la función de hash:

```python
canonical_data = {
    "name": self.name.lower(),
    "type": self.type,
    "version": self.version,
    "install": {
        "mode": self.install.mode,
        "clean": self.install.clean,
        "here_url": self.install.here.url if self.install.here else None,
        "site_url": self.install.site.url if self.install.site else None,
        "command_script": self.install.command.script if self.install.command else None,
        "binaries": (
            self.install.here.binaries if self.install.here
            else (self.install.command.binaries if self.install.command else [])
        ),
        "addpath": self.install.here.addpath if self.install.here else False,
    }
}
encoded = json.dumps(canonical_data, sort_keys=True).encode("utf-8")
return hashlib.sha256(encoded).hexdigest()
```

Esta función garantiza idempotencia matemática: cualquier modificación en la URL de descarga, en los ejecutables declarados o en el modo de instalación altera el hash de integridad resultante.

---

## ⚠️ Consideraciones Críticas y Casos de Borde

1. **Formatos Heterogéneos de Versión:** En manifests heredados, la versión puede estar definida como una lista de enteros (ej. `[3, 14]`). El método `Manifest.from_dict()` detecta automáticamente esta condición y la normaliza a cadena estándar `"3.14"`.
2. **Generación de ID Automático:** Si el JSON omite el campo `id`, el constructor sintetiza un identificador semántico determinista siguiendo la regla:
   - Tipo `stable`: `<nombre_slug>@stable`
   - Con versión: `<nombre_slug>@<version>`
   - Por defecto: `<nombre_slug>@default`
3. **Persistencia de `PackageState`:** Al registrar un paquete en `state.json`, se congela el `manifest_hash` original. Si un usuario actualiza el manifiesto en el catálogo, el actualizador detecta la desviación entre el hash instalado y el nuevo hash para desencadenar el proceso de actualización.

---

## 💡 Ejemplo de Declaración de Manifiesto en JSON

```json
{
  "id": "c-gcc",
  "name": "C/C++ (GCC)",
  "type": "fixed",
  "version": "1.23.0",
  "icon": "⚙️",
  "category": "compilers",
  "description": "Entorno completo y portátil de C/C++ para Windows con GCC 14, G++, GDB y GNU Make.",
  "aliases": ["c", "gcc", "g++", "w64devkit"],
  "install": {
    "mode": "here",
    "clean": true,
    "here": {
      "url": "https://github.com/skeeto/w64devkit/releases/download/v1.23.0/w64devkit-1.23.0.zip",
      "format": "zip",
      "binaries": ["gcc.exe", "g++.exe", "gdb.exe", "make.exe"],
      "addpath": true
    }
  }
}
```
