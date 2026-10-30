# Changelog

Todos los cambios importantes de Packwire se documentan aquí.

## [1.2.0] - 2026-10-29

### Añadido

- Catálogo remoto de manifests sincronizado desde `assets/manifests/` del repositorio oficial de Packwire en GitHub.
- Caché local del catálogo para mantener el funcionamiento offline después de una sincronización correcta.
- Actualización automática del catálogo al abrir la GUI o iniciar la CLI.
- API local `get_catalog_sync` para informar a la interfaz si el catálogo se actualizó.

### Cambiado

- Los manifests ya no se empaquetan dentro del instalador ni de la versión portable.
- Los manifests remotos prevalecen sobre cualquier catálogo de desarrollo local y se eliminan del caché cuando dejan de existir en GitHub.

## [1.1.0] - 2026-10-29

### Añadido

- Manifest de NASM para ensamblado x86/x86-64 mediante WinGet.
- Manifest de GNU Binutils para exponer GAS (`as.exe`) en MSYS2 UCRT64.
- Manifest de QEMU para emulación y virtualización desde MSYS2 UCRT64.
- Alias de búsqueda para NASM, GAS, Binutils y QEMU.

## [1.0.0] - 2026-10-27

### Estable

- Primera versión estable de Packwire con CLI y GUI nativas en Rust.
- Distribución oficial para Windows mediante binarios portables e instalador Inno Setup.
- Catálogo ampliado de herramientas, lenguajes, runtimes, sistemas operativos y aplicaciones.
- Eliminada por completo la implementación Python y su cadena de empaquetado heredada.

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
