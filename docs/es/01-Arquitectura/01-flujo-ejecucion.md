# Flujo de Ejecución

El flujo principal vive en `packwire-rs/src/cli.rs`, `packwire-rs/src/task_queue.rs` y `packwire-rs/src/installer.rs`:

1. La CLI o la GUI recibe el identificador del paquete.
2. `manifest.rs` encuentra y valida el manifest.
3. `resolver.rs` obtiene la versión y los artefactos oficiales cuando es necesario.
4. `task_queue.rs` ejecuta la operación fuera del hilo de interfaz.
5. `installer.rs` descarga, valida, extrae y registra el paquete.
6. `pather.rs` crea o actualiza los shims.

El estado se escribe atómicamente en `%APPDATA%\packwire\installed.json` en Windows o en la ruta equivalente de la plataforma.
