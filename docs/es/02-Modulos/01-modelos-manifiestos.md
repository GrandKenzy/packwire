# Modelos de Datos y Manifiestos

Los contratos tipados de Packwire están implementados en `packwire-rs/src/models.rs`. El cargador y el parser tolerante de JSON están en `packwire-rs/src/manifest.rs`.

Los modelos cubren `Manifest`, `InstallConfig`, `PackageState` y la información de los paquetes instalados. El hash canónico SHA-256 permite detectar cambios en los manifiestos y mantener compatibilidad con el estado persistente existente.

Los archivos fuente del catálogo viven en `assets/manifests/` dentro del repositorio de Packwire. Las versiones distribuidas de Packwire sincronizan esa carpeta desde GitHub al iniciar y guardan la copia validada en `%APPDATA%\packwire\manifests`. El caché local permite seguir usando el catálogo sin conexión.
