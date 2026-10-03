# Referencia de la API de Python

El módulo raíz [`packwire`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/__init__.py) expone una interfaz de programación de aplicaciones (API) de alto nivel concebida para ser importada en scripts de automatización, entornos de integración continua o herramientas de aprovisionamiento de infraestructura como código en Windows.

Todas las funciones públicas incorporan anotaciones estrictas de tipo (`typing`) y gestionan internamente la orquestación de visitantes, validadores y persistencia en disco de manera segura.

---

## ⚙️ Funciones Principales

### `packwire.install`

Instala un paquete en el sistema según las directivas resueltas.

```python
def install(
    target: str,
    version: Optional[str] = None,
    channel: str = "stable",
    mode: Optional[str] = None,
    force: bool = False,
    logger: Optional[Callable[[str], None]] = None
) -> Tuple[bool, str]:
```

| Parámetro | Tipo | Por Defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `target` | `str` | *(Requerido)* | Nombre o identificador semántico del paquete (ej. `"python"`, `"python@3.14"`, `"c"`). |
| `version` | `Optional[str]` | `None` | Cadena de versión específica opcional (ej. `"3.13.2"`). |
| `channel` | `str` | `"stable"` | Canal de distribución: `"stable"` (dinámico upstream) o `"fixed"`. |
| `mode` | `Optional[str]` | `None` | Modo de despliegue: `"here"`, `"command"` o `"site"`. Si es `None`, adopta el predeterminado del manifiesto. |
| `force` | `bool` | `False` | Si es `True`, ignora comprobaciones de colisión y sobrescribe archivos preexistentes. |
| `logger` | `Optional[Callable[[str], None]]` | `None` | Función de callback que recibe mensajes de estado y progreso en tiempo real. |

* **Retorno:** `Tuple[bool, str]` donde el primer valor es `True` si la instalación culminó con éxito y el segundo es un mensaje explicativo.

---

### `packwire.uninstall`

Remueve un paquete registrado, eliminando sus ejecutables físicos y sus shims asociados.

```python
def uninstall(pkg_id: str) -> Tuple[bool, str]:
```

| Parámetro | Tipo | Descripción |
| :--- | :--- | :--- |
| `pkg_id` | `str` | Identificador canónico del paquete instalado (ej. `"python@stable"`, `"c-gcc"`). |

* **Retorno:** `Tuple[bool, str]`.

---

### `packwire.uninstall_all`

Ejecuta una purga completa y profunda de todo el ecosistema de Packwire en el equipo.

```python
def uninstall_all(remove_path: bool = True) -> Tuple[bool, str]:
```

| Parámetro | Tipo | Por Defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `remove_path` | `bool` | `True` | Si es `True`, remueve la carpeta de shims de la variable `PATH` del usuario en el registro. |

* **Retorno:** `Tuple[bool, str]`.

---

### `packwire.reinstall`

Fuerza la reinstalación limpia de un paquete que ya se encuentra en `state.json`.

```python
def reinstall(pkg_id: str) -> Tuple[bool, str]:
```

---

### `packwire.update`

Comprueba y aplica las actualizaciones disponibles para paquetes instalados bajo el canal `stable`.

```python
def update(pkg_id: Optional[str] = None) -> List[Tuple[str, bool, str]]:
```

* **Retorno:** Lista de tuplas `(id_paquete, exito_booleano, mensaje_detalle)`.

---

### `packwire.check_updates`

Inspecciona en segundo plano si los paquetes instalados tienen nuevas versiones liberadas upstream sin aplicar cambios en disco.

```python
def check_updates() -> Dict[str, Dict[str, Any]]:
```

* **Retorno:** Diccionario indexado por el ID del paquete con la estructura:
  ```python
  {
      "python@stable": {
          "has_update": True,
          "current_version": "3.13.1",
          "latest_version": "3.14.0",
          "name": "Python",
          "mode": "here",
          "type": "stable"
      }
  }
  ```

---

### `packwire.list_installed`

Obtiene una instantánea del estado de todos los paquetes registrados en el sistema.

```python
def list_installed() -> Dict[str, Dict[str, Any]]:
```

---

### `packwire.list_available`

Retorna la lista de todos los manifiestos presentes en el catálogo integrado de Packwire.

```python
def list_available() -> List[Dict[str, Any]]:
```

---

### `packwire.get_manifest`

Localiza y retorna los datos serializados del manifiesto correspondiente a un objetivo.

```python
def get_manifest(target: str) -> Optional[Dict[str, Any]]:
```

---

### `packwire.add_path`

Garantiza que la carpeta de shims de Packwire esté presente en la variable de entorno `PATH` del usuario en Windows.

```python
def add_path() -> bool:
```

---

### `packwire.launch_gui`

Despliega la interfaz gráfica moderna de usuario.

```python
def launch_gui(web_mode: bool = False, port: int = 5050) -> None:
```

---

## 🔒 Constantes y Rutas de Configuración Exportadas

```python
from packwire import (
    PACKWIRE_ROOT,  # Path al directorio raíz de Packwire (%APPDATA%/packwire)
    APPS_DIR,       # Path al almacenamiento de binarios (%APPDATA%/packwire/apps)
    SHIMS_DIR,      # Path al directorio de shims (%APPDATA%/packwire/shims)
    CACHE_DIR,      # Path al directorio de descargas temporales (%APPDATA%/packwire/cache)
    CONFIG_FILE     # Path al archivo de configuración general
)
```

---

## 💡 Ejemplo Completo de Automatización

```python
import packwire

# 1. Comprobar si los shims están en el PATH; si no, añadirlos
packwire.add_path()

# 2. Instalar suite de compilación C/C++ y servidor Node.js
paquetes = ["c", "nodejs"]

for pkg in paquetes:
    print(f"[*] Instalando {pkg}...")
    ok, detalle = packwire.install(pkg, channel="stable", mode="here")
    if ok:
        print(f"[+] Éxito: {detalle}")
    else:
        print(f"[-] Error al instalar {pkg}: {detalle}")

# 3. Reportar inventario instalado
print("\n--- INVENTARIO ACTUAL ---")
for id_pkg, info in packwire.list_installed().items():
    print(f"• {info['name']} v{info['version']} ({info['id']}) en {info['install_path']}")
```
