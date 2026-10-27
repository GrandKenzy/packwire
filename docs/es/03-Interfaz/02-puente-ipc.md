# Puente IPC y Comunicación Bidireccional (GuiBridge)

El puente de comunicación entre procesos (IPC) es el componente responsable de mediar entre el entorno de ejecución web (JavaScript en el DOM) y el motor de Packwire en Python. Se implementa en [`packwire/core/gui/bridge.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/gui/bridge.py) mediante la clase `GuiBridge`.

`GuiBridge` expone una interfaz RPC (Remote Procedure Call) unificada que es accesible de forma dual: directamente a través de promesas nativas de JavaScript (`window.pywebview.api.<metodo>()`) en modo ventana, o mediante llamadas HTTP REST asíncronas (`fetch('/api/<endpoint>')`) cuando opera bajo el servidor web.

---

## ⚙️ Especificación Técnica

### Catálogo de Métodos de `GuiBridge`

```python
class GuiBridge:
    def get_available(self) -> List[Dict[str, Any]]: ...
    def get_installed(self) -> Dict[str, Dict[str, Any]]: ...
    def install(
        self,
        package_id: str,
        mode: Optional[str] = None,
        version: Optional[str] = None,
        force: bool = False
    ) -> Dict[str, Any]: ...
    def uninstall(self, package_id: str) -> Dict[str, Any]: ...
    def uninstall_all(self, remove_path: bool = True) -> Dict[str, Any]: ...
    def reinstall(self, package_id: str) -> Dict[str, Any]: ...
    def update(self, package_id: Optional[str] = None) -> List[Dict[str, Any]]: ...
    def check_updates(self) -> Dict[str, Dict[str, Any]]: ...
    def get_manifest(self, target: str) -> Optional[Dict[str, Any]]: ...
    def get_jobs(self) -> List[Dict[str, Any]]: ...
    def get_job(self, job_id: str) -> Optional[Dict[str, Any]]: ...
    def get_config(self, key: Optional[str] = None) -> Any: ...
    def set_config(self, key: str, value: Any) -> Dict[str, Any]: ...
    def is_shims_in_path(self) -> bool: ...
    def add_shims_to_path(self) -> bool: ...
    def get_global_css(self) -> str: ...
```

---

## 🔍 Matriz de Contratos RPC

| Método | Argumentos Clave | Tipo Retorno | Descripción Operativa |
| :--- | :--- | :--- | :--- |
| `get_available` | Ninguno | `List[Dict]` | Retorna todos los manifiestos disponibles en el catálogo. |
| `get_installed` | Ninguno | `Dict[str, Dict]` | Retorna los paquetes instalados registrados en `state.json`. |
| `install` | `package_id`, `mode`, `version`, `force` | `Dict[str, Any]` | Encola la instalación en `TaskQueue` y retorna `{"success": True, "job_id": str}`. |
| `uninstall` | `package_id` | `Dict[str, Any]` | Desinstala un paquete y remueve sus shims. |
| `uninstall_all`| `remove_path: bool` | `Dict[str, Any]` | Purgado absoluto de aplicaciones, shims, caché y PATH. |
| `reinstall` | `package_id` | `Dict[str, Any]` | Reinstala limpiamente forzando sobrescritura. |
| `check_updates`| Ninguno | `Dict[str, Dict]` | Chequea en segundo plano nuevas versiones upstream. |
| `get_jobs` | Ninguno | `List[Dict]` | Consulta la telemetría y logs de la cola de tareas. |
| `get_config` | `key: Optional[str]` | `Any` | Obtiene opciones de usuario desde `configs.json`. |
| `set_config` | `key: str, value: Any` | `Dict[str, Any]` | Persiste una clave/valor en `configs.json`. |

---

## ⚙️ Persistencia de Configuración (`configs.json`)

Para evitar que las elecciones del usuario (como el tema visual seleccionado) se restablezcan al reiniciar la aplicación, `GuiBridge` gestiona el archivo persistente:

```
%APPDATA%\packwire\configs.json
```

La lectura y escritura se efectúan de forma atómica y tolerante a fallos:

```python
def set_config(self, key: str, value: Any) -> Dict[str, Any]:
    from packwire.core.config import set_config
    try:
        set_config(key, value)
        return {"success": True, "key": key, "value": value}
    except Exception as e:
        return {"success": False, "error": str(e)}
```

---

## ⚠️ Consideraciones Críticas y Casos de Borde

1. **Serialización JSON Estricta:** Todas las respuestas emitidas por `GuiBridge` deben ser tipos primitivos serializables (`dict`, `list`, `str`, `int`, `float`, `bool`). Cualquier objeto de dominio (`Manifest` o `PackageState`) se convierte a diccionario mediante su método `.to_dict()` antes de ser transmitido.
2. **Despacho Asíncrono no Bloqueante:** Al invocar `bridge.install()`, el método no ejecuta la instalación en el hilo de la interfaz; en su lugar, la envía a `TaskQueue.submit_install()` y retorna inmediatamente el `job_id`. Esto garantiza que la interfaz web permanezca receptiva a 60 FPS sin sufrir congelamientos durante transferencias masivas de datos.

---

## 💡 Ejemplo de Invocación desde JavaScript (`app.js`)

```javascript
// Iniciar instalación asíncrona mediante el puente IPC
async function instalarPaquete(pkgId, modo) {
    try {
        let res;
        if (window.pywebview && window.pywebview.api) {
            // Invocación directa pywebview IPC
            res = await window.pywebview.api.install(pkgId, modo);
        } else {
            // Fallback HTTP REST
            const r = await fetch('/api/install', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ package: pkgId, mode: modo })
            });
            res = await r.json();
        }

        if (res.success) {
            console.log(`Trabajo iniciado con éxito. ID: ${res.job_id}`);
            iniciarMonitoreoTarea(res.job_id);
        } else {
            alert(`Error: ${res.message}`);
        }
    } catch (err) {
        console.error("Fallo de comunicación IPC:", err);
    }
}
```
