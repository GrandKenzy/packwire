# Modelos de Datos y Manifiestos

Los contratos tipados de Packwire están implementados en `packwire-rs/src/models.rs`. El cargador y el parser tolerante de JSON están en `packwire-rs/src/manifest.rs`.

Los modelos cubren `Manifest`, `InstallConfig`, `PackageState` y la información de los paquetes instalados. El hash canónico SHA-256 permite detectar cambios en los manifiestos y mantener compatibilidad con el estado persistente existente.

Los archivos de catálogo se distribuyen en `assets/manifests/`.
