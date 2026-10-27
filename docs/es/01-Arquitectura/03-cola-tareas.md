# Cola de Tareas y Concurrencia

`packwire-rs/src/task_queue.rs` mantiene una cola segura para operaciones de red, extracción e instalación. Cada trabajo tiene un identificador, estado, progreso y eventos de log consultables desde la GUI.

La cola evita bloquear el hilo de renderizado y coordina los hilos de trabajo mediante canales y estado sincronizado. Los errores se devuelven al trabajo sin dejar registros parciales de instalación.
