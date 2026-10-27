# Modos de Instalación: Here, Command y Site

`packwire-rs/src/installer.rs` implementa los tres modos declarados por los manifests:

- `here`: descarga y extrae el paquete dentro del directorio administrado por Packwire.
- `command`: ejecuta el instalador o comando oficial definido por el manifest.
- `site`: abre el sitio oficial para herramientas que deben instalarse mediante su proveedor.

El instalador valida rutas, evita escapes de archivos comprimidos, registra el estado y genera los shims correspondientes.
