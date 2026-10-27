# Cola de Tareas y Concurrencia

El subsistema de cola de tareas gestiona la ejecución asíncrona de operaciones pesadas (descargas de red, descompresión de archivos de gran volumen y scripts de instalación de comandos) en hilos de fondo secundarios. Se encuentra implementado en [`packwire/core/task_queue.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/task_queue.py) mediante las clases `TaskJob` y `TaskQueue`, exponiendo la instancia singleton global `task_queue`.

Este módulo previene el bloqueo de la interfaz gráfica y permite al usuario encolar múltiples instalaciones consecutivas mientras supervisa la telemetría, el progreso porcentual y los registros de depuración en tiempo real.

---

## ⚙️ Especificación Técnica

### Estructura `TaskJob`

Representa una unidad de trabajo individual dentro del despachador:

```python
class TaskJob:
    def __init__(
        self,
        job_id: str,
        package_id: str,
        package_name: str,
        mode: str = "here",
        version: Optional[str] = None,
        force: bool = False
    ) -> None: ...

    def add_log(self, message: str) -> None: ...
    def set_status(self, status: str, status_text: Optional[str] = None) -> None: ...
    def set_result(self, success: bool, message: str) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...
```

#### Atributos de Estado de `TaskJob`

| Atributo | Tipo | Descripción |
| :--- | :--- | :--- |
| `id` | `str` | Identificador único del trabajo con formato `job-<hex8>`. |
| `package_id` | `str` | Identificador canónico del paquete destino (ej. `python@stable`). |
| `package_name`| `str` | Nombre legible para presentación visual en el dashboard. |
| `status` | `str` | Estado actual: `"queued"`, `"running"`, `"completed"` o `"failed"`. |
| `status_text` | `str` | Mensaje descriptivo breve de la sub-tarea en curso. |
| `progress` | `int` | Porcentaje de avance (0 a 100) o `-1` para operaciones indeterminadas. |
| `logs` | `List[str]` | Historial cronometrado de mensajes con formato `[HH:MM:SS] <mensaje>`. |
| `result` | `Optional[Dict]`| Diccionario final `{"success": bool, "message": str}` al culminar. |
| `created_at` | `float` | Marca de tiempo UNIX de creación. |
| `completed_at`| `Optional[float]`| Marca de tiempo UNIX de finalización de la tarea. |

---

### Estructura `TaskQueue`

Motor de despacho con cola bloqueante sincronizada (`queue.Queue`) y pool de hilos trabajadores:

```python
class TaskQueue:
    def __init__(self, num_workers: int = 1) -> None: ...
    def submit_install(
        self,
        package_id: str,
        package_name: Optional[str] = None,
        mode: str = "here",
        version: Optional[str] = None,
        force: bool = False
    ) -> str: ...
    def get_job(self, job_id: str) -> Optional[Dict[str, Any]]: ...
    def list_jobs(self) -> List[Dict[str, Any]]: ...
```

---

## 🔍 Ciclo de Vida del Hilo de Trabajo (`_worker_loop`)

El despacho opera bajo el siguiente patrón concurrente:

```
[Cliente (GUI / API)]
         │
         │ submit_install()
         ▼
[TaskJob creado en _jobs] ────► [queue.Queue.put(job)]
                                          │
                                          ▼
                         [Hilo Demonio PackwireWorker-0]
                                          │
                                 job = queue.get()
                                          │
                                          ▼
                                   job.set_status("running")
                                          │
                                          ▼
                       packwire.install(..., logger=job.add_log)
                                          │
                                 ┌────────┴────────┐
                                 ▼                 ▼
                             (Éxito)            (Error)
                                 │                 │
                         job.set_result(True) job.set_result(False)
                                 │                 │
                                 └────────┬────────┘
                                          ▼
                               queue.task_done()
```

1. **Recepción:** El hilo trabajador se mantiene en espera activa con un timeout de 1.0 segundo sobre `self._queue.get()`, comprobando la bandera de parada `_stop_event`.
2. **Ejecución y Telemetría:** Se asigna el estado `"running"`. Al invocar `packwire.install()`, se transfiere el método `job.add_log` como callback de registro. Conforme el extractor, descargador o procesador de comandos emiten mensajes, estos se almacenan protegidos por el cerrojo de exclusión mutua `self._lock` en la instancia del trabajo.
3. **Captura Robusta de Fallos:** Si el proceso interno lanza una excepción no controlada (`Exception`), el worker intercepta el error, establece el resultado en `success: False` con la traza correspondiente y garantiza la llamada a `queue.task_done()`.

---

## ⚠️ Consideraciones Críticas y Casos de Borde

1. **Serialización vs. Paralelismo:** Por defecto, `TaskQueue` se inicializa con `num_workers = 1`. Esta decisión arquitectónica previene que dos instalaciones simultáneas compitan por el cerrojo de escritura en `state.json` o manipulen la carpeta compartida de shims al mismo tiempo.
2. **Hilos Demonio (`daemon=True`):** Los trabajadores están marcados como hilos demonio, garantizando que el cierre de la ventana principal de la aplicación o la terminación del proceso principal finalice los subprocesos sin procesos zombi en segundo plano.
3. **Sincronización con el Frontend:** El panel web implementa un sondeo periódico (polling cada 1000ms) a `list_jobs()`, lo que permite reflejar el progreso y los registros en el visor sin requerir dependencias complejas como WebSockets.

---

## 💡 Ejemplo de Uso

```python
import time
from packwire.core.task_queue import task_queue

# Encolar la instalación de GCC y FFmpeg en segundo plano
id_gcc = task_queue.submit_install("c", package_name="C/C++ (GCC)", mode="here")
id_ff = task_queue.submit_install("ffmpeg", package_name="FFmpeg", mode="here")

print(f"Trabajos encolados: {id_gcc}, {id_ff}")

# Monitorear el progreso en consola hasta la culminación
while True:
    estado_gcc = task_queue.get_job(id_gcc)
    print(f"Estado de {estado_gcc['package_name']}: {estado_gcc['status']} - {estado_gcc['status_text']}")
    
    if estado_gcc["status"] in ("completed", "failed"):
        print(f"Resultado final: {estado_gcc['result']}")
        break
        
    time.sleep(1)
```
