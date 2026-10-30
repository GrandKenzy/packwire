# Flujo de Ejecución

El flujo principal vive en `packwire-rs/src/cli.rs`, `packwire-rs/src/task_queue.rs`, `packwire-rs/src/installer.rs` y `packwire-rs/src/manifest.rs`:

1. Al iniciar, `sync_remote_catalog` consulta la carpeta `assets/manifests/` del repositorio de Packwire en GitHub.
2. Los manifests nuevos o modificados se validan y se guardan en la caché local; si no hay red, se conserva la caché anterior.
3. La CLI o la GUI recibe el identificador del paquete.
4. `manifest.rs` encuentra y valida el manifest.
5. `resolver.rs` obtiene la versión y los artefactos oficiales cuando es necesario.
6. `task_queue.rs` ejecuta la operación fuera del hilo de interfaz.
7. `installer.rs` descarga, valida, extrae y registra el paquete.
8. `pather.rs` crea o actualiza los shims.

El estado se escribe atómicamente en `%APPDATA%\packwire\installed.json` y el catálogo sincronizado en `%APPDATA%\packwire\manifests` en Windows, o en las rutas equivalentes de la plataforma.
