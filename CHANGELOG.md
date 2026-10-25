# Changelog

Todos los cambios importantes de Packwire se documentan aquí.

## [0.1.4] - 2026-10-24

### Añadido

- Resolución dinámica de las últimas versiones estables oficiales para GCC/w64devkit, LLVM/Clang, Rufus, Ventoy y PowerShell 7.
- Verificación SHA-256 de los assets publicados por GitHub cuando el proveedor ofrece el digest.
- Alternativas MSYS2 UCRT64 y CLANG64, además de MSVC.
- Catálogo Linux con 18 distribuciones enlazadas a sus páginas oficiales para PCs antiguas, servidores, programación y uso personal.
- Icono predeterminado local con fallback seguro para iconos remotos de paquetes.
- Diagnóstico local ampliado de Windows: edición/build, equipo, CPU, RAM, GPU, almacenamiento, BIOS/UEFI, Secure Boot, TPM, virtualización, uptime y adaptadores activos.
- Plan de datos documentado para explicar utilidad, límites y privacidad del diagnóstico.

### Cambiado

- El apartado Sistema ahora tiene un único botón de desinstalación total y un botón separado para actualizar el diagnóstico.
- La interfaz muestra el filtro Linux y distingue mejor entre entradas `stable`, `fixed` y enlaces de catálogo.
- El instalador valida hashes esperados y protege la extracción de archivos comprimidos frente a rutas inseguras.
- La guía de GCC recomienda mantener Defender activo, verificar el origen y revisar el historial de protección antes de restaurar un falso positivo.

### Seguridad

- El diagnóstico no recopila números de serie, claves de producto, nombre de usuario, MAC, IP, SSID, historial, procesos ni contenido de discos.
- No se añaden exclusiones amplias de Windows Defender automáticamente.

## [0.1.3] - 2026-10-24

- Versión anterior publicada en GitHub Releases.
