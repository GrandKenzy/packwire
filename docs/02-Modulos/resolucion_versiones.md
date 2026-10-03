# Resolución Dinámica de Versiones y Upstream

El motor de resolución de versiones es el subsistema encargado de descubrir, verificar y enlazar dinámicamente las últimas versiones oficiales publicadas por los proveedores de software (upstream). Se encuentra implementado en [`packwire/core/resolver.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/resolver.py).

Este mecanismo permite que paquetes declarados con canal `stable` permanezcan permanentemente al día sin necesidad de modificar manualmente los archivos de manifiesto cada vez que un lenguaje o herramienta libera una nueva revisión menor o de seguridad.

---

## ⚙️ Especificación Técnica

### Firmas y Estructuras

```python
class PythonResolver:
    @staticmethod
    def resolve_latest_stable() -> Tuple[str, str, str]: ...

    @staticmethod
    def resolve_fixed(version: str) -> Tuple[str, str, str]: ...

class NodeResolver:
    @staticmethod
    def resolve_latest_lts() -> Tuple[str, str, str]: ...

def resolve_manifest(manifest: Manifest) -> Manifest: ...
```

### Parámetros y Retorno de `resolve_manifest`

| Parámetro | Tipo | Requerido | Descripción |
| :--- | :--- | :--- | :--- |
| `manifest` | `Manifest` | Sí | Instancia original del manifiesto a enriquecer con datos de red. |

* **Retorno:** Instancia de `Manifest` con los campos `version`, `here.url`, `site.url` y `manifest_hash` actualizados conforme a la respuesta oficial del upstream.

---

## 🔍 Resolvedores Implementados

### 1. `PythonResolver`
Interactúa con el servicio REST público oficial de la Python Software Foundation:
* **Endpoint de consulta:** `https://www.python.org/api/v2/downloads/release/?is_published=true`
* **Criterio de filtrado:** Ordena las entregas por fecha de publicación descendente, descartando automáticamente versiones pre-release (`alpha`, `beta`, `rc`).
* **Construcción de URL Binaria:** Selecciona el paquete portable oficial embebido de 64 bits para Windows:
  ```
  https://www.python.org/ftp/python/{version}/python-{version}-embed-amd64.zip
  ```
* **Mecanismo de Resiliencia (Offline Fallback):** Si la red se encuentra inaccesible o la API no responde dentro del timeout establecido (5.0 segundos), el resolvedor conmuta automáticamente a una versión de respaldo estable precompilada (`3.13.2`), evitando que el instalador falle de forma catastrófica en entornos desconectados.

### 2. `NodeResolver`
Consulta el índice de versiones oficial de la Fundación OpenJS:
* **Endpoint de consulta:** `https://nodejs.org/dist/index.json`
* **Criterio de filtrado:** Filtra la colección de versiones buscando aquellas marcadas con la propiedad `lts != False` (versiones con Soporte Extendido a Largo Plazo).
* **Construcción de URL Binaria:** Ensambla la ruta al paquete ZIP autónomo para la arquitectura Windows x64:
  ```
  https://nodejs.org/dist/{version}/node-{version}-win-x64.zip
  ```

---

## ⚙️ Inyección en Manifiestos (`resolve_manifest`)

La función `resolve_manifest()` actúa como despachador de resolución analizando el identificador o nombre del paquete:

```python
def resolve_manifest(manifest: Manifest) -> Manifest:
    pkg_name = manifest.name.lower()
    pkg_id = manifest.id.lower()

    if "python" in pkg_name or "python" in pkg_id:
        if manifest.type == "stable":
            version, dl_url, site_url = PythonResolver.resolve_latest_stable()
            manifest.version = version
            if manifest.install.here:
                manifest.install.here.url = dl_url
            if manifest.install.site:
                manifest.install.site.url = site_url
        elif manifest.version:
            version, dl_url, site_url = PythonResolver.resolve_fixed(manifest.version)
            if manifest.install.here:
                manifest.install.here.url = dl_url

    elif "node" in pkg_name or "node" in pkg_id:
        if manifest.type == "stable":
            version, dl_url, site_url = NodeResolver.resolve_latest_lts()
            manifest.version = version
            if manifest.install.here:
                manifest.install.here.url = dl_url

    # Recalcular el hash criptográfico tras la resolución
    manifest.manifest_hash = manifest.calculate_hash()
    return manifest
```

---

## ⚠️ Consideraciones Críticas y Casos de Borde

1. **Cabecera `User-Agent` Obligatoria:** Ciertas APIs (incluyendo `python.org` y `github.com`) rechazan peticiones realizadas con el User-Agent estándar de `urllib` de Python. Todos los resolvedores configuran explícitamente:
   ```python
   headers = {"User-Agent": "Packwire/0.1.0 (Windows; Python)"}
   ```
2. **Determinismo Post-Resolución:** Al mutar la URL y versión dinámica, `resolve_manifest()` recalcula inmediatamente `manifest.calculate_hash()`, asegurando que la suma criptográfica registrada en `state.json` coincida con el paquete efectivamente descargado.
3. **Control de Timeouts:** Las solicitudes HTTP usan un timeout estricto de 5 segundos para evitar bloqueos prolongados en conexiones degradadas.

---

## 💡 Ejemplo de Uso

```python
from packwire.core.resolver import PythonResolver, NodeResolver

# Obtener dinámicamente la última versión de Python
py_ver, py_dl, py_site = PythonResolver.resolve_latest_stable()
print(f"Python Estable Detectado: v{py_ver}")
print(f"URL de Descarga: {py_dl}")

# Obtener la última versión LTS de Node.js
node_ver, node_dl, _ = NodeResolver.resolve_latest_lts()
print(f"Node.js LTS Detectado: {node_ver}")
print(f"URL de Descarga: {node_dl}")
```
