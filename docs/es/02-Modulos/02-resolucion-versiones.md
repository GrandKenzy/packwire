# Resolución Dinámica de Versiones

El motor de resolución está implementado en `packwire-rs/src/resolver.rs`. Consulta fuentes oficiales para resolver canales `stable` y versiones `fixed`, y actualiza las URLs y hashes del manifest antes de instalar.

Los resolvers integrados cubren, entre otros, Python, Node.js y releases de GitHub. La resolución se ejecuta antes de la descarga y no modifica el catálogo local original.
