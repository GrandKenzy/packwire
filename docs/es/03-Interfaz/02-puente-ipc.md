# Comunicación entre la GUI y el backend

El puente de Packwire está implementado en `packwire-rs/src/gui.rs`. La interfaz web y el núcleo Rust intercambian objetos JSON mediante endpoints HTTP locales; la GUI nativa carga el mismo servidor dentro de `packwirew.exe`.

## Operaciones principales

| Operación | Función |
|---|---|
| `get_available` | Lista los manifests del catálogo. |
| `get_installed` | Lee el estado de paquetes instalados. |
| `install` | Encola una instalación. |
| `uninstall` | Desinstala un paquete gestionado. |
| `get_jobs` / `get_job` | Consulta el progreso de tareas. |
| `get_config` / `set_config` | Lee o persiste la configuración. |
| `get_system_info` | Devuelve diagnósticos locales del sistema. |
| `list_system_apps` | Lista aplicaciones detectadas por Windows. |

Las respuestas se serializan con `serde_json`. Las operaciones de red, extracción e instalación se ejecutan fuera del hilo de renderizado mediante `task_queue.rs`, por lo que la ventana permanece receptiva.
