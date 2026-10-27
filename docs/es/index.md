# Documentación del Proyecto Packwire

> **Versión:** `0.1.4`  
> **Entorno:** Python 3.9+ (Windows, Linux, macOS)  
> **Licencia:** MIT  

Bienvenido a la documentación oficial en español de **Packwire**, un gestor y descargador moderno, elegante y autónomo de paquetes, compiladores y entornos de desarrollo para Windows, Linux y macOS.

Packwire resuelve de raíz la dispersión de versiones de herramientas de desarrollo mediante identificadores semánticos unificados (`<nombre>@<canal_o_versión>`), garantizando aislamiento total de binarios mediante una arquitectura centralizada de lanzadores proxy (*shims*) y una cola asíncrona de tareas en segundo plano.

---

## Índice de Documentación

### [01. Arquitectura del Sistema](./01-Arquitectura/index.md)
Fundamentos estructurales, directrices de diseño y patrones de ingeniería que rigen la operación interna de Packwire.
- **[01. Flujo de Ejecución](./01-Arquitectura/01-flujo-ejecucion.md)** — Ciclo de vida completo desde la resolución del manifiesto hasta la compilación de shims y registro de estado.
- **[02. Sistema de Shims y PATH](./01-Arquitectura/02-sistema-shims.md)** — Mecanismo proxying no invasivo en Windows (`.cmd`/`.ps1`), Linux y macOS (POSIX).
- **[03. Cola de Tareas y Concurrencia](./01-Arquitectura/03-cola-tareas.md)** — Orquestación no bloqueante con subprocesos demonio y logs cronometrados.

### [02. Módulos del Núcleo](./02-Modulos/index.md)
Especificación de modelos, resolución upstream, modos de instalación y la arquitectura de visitantes.
- **[01. Modelos y Manifiestos](./02-Modulos/01-modelos-manifiestos.md)** — Contratos de datos tipados, esquema de manifiestos y cálculo de hash determinista SHA-256.
- **[02. Resolución de Versiones](./02-Modulos/02-resolucion-versiones.md)** — Consulta dinámica a APIs oficiales y resolución de canales `stable` vs `fixed`.
- **[03. Modos de Instalación](./02-Modulos/03-instalador-modos.md)** — Comportamiento y flujo de los modos `here`, `command` y `site`.
- **[04. Subsistema de Visitantes](./02-Modulos/04-visitantes.md)** — Responsabilidades de `DownloaderVisitor`, `PatherVisitor`, `UninstallerVisitor` y `UpdaterVisitor`.

### [03. Interfaz de Usuario](./03-Interfaz/index.md)
Arquitectura del frontend SPA moderno acelerado por WebView nativo, puente IPC y tematización.
- **[01. Arquitectura GUI](./03-Interfaz/01-arquitectura-gui.md)** — Renderizado nativo con WebView2 / WebKit, sin el sobrecoste de memoria de Electron.
- **[02. Puente IPC y Comunicación](./03-Interfaz/02-puente-ipc.md)** — Especificación de llamadas RPC entre JavaScript y Python con soporte para jobs asíncronos.
- **[03. Sistema de Temas y Estilos](./03-Interfaz/03-temas-estilos.md)** — Hojas de estilo desacopladas y persistencia de temas visuales en `configs.json`.

### [04. Empaquetado y Distribución](./04-Distribucion/index.md)
Guías de despliegue para usuarios finales y administradores de sistemas en Windows, Linux y macOS.
- **[01. Guía Completa de Instalación](./04-Distribucion/01-guia-instalacion.md)** — Manual paso a paso multiplataforma para todas las modalidades de instalación.
- **[02. Integración en el Sistema](./04-Distribucion/02-instalacion-sistema.md)** — Registro en PATH, creación de accesos directos y shims de usuario.
- **[03. Empaquetador PyInstaller](./04-Distribucion/03-empaquetador-pyinstaller.md)** — Construcción de ejecutables autónomos `packwire.exe` y `packwirew.exe`.
- **[04. Instalador Inno Setup](./04-Distribucion/04-instalador-inno-setup.md)** — Compilación del asistente de instalación nativo para Windows (`Packwire-Setup.exe`).

### [05. Referencia y Diagnósticos](./05-Referencia/index.md)
Manuales formales de referencia de comandos, API de Python, diagnósticos de hardware y catálogo Linux.
- **[01. Interfaz de Línea de Comandos (CLI)](./05-Referencia/01-cli.md)** — Referencia exhaustiva de comandos, opciones y parámetros.
- **[02. API de Python](./05-Referencia/02-api-python.md)** — Firmas, tipos y ejemplos de uso de la biblioteca de Python.
- **[03. Catálogo de Diagnósticos y Errores](./05-Referencia/03-catalogo-errores.md)** — Matriz formal de errores, causas y soluciones paso a paso.
- **[04. Catálogo Linux](./05-Referencia/04-catalogo-linux.md)** — Guía de distribuciones Linux recomendadas por perfil de máquina y propósito.
- **[05. Plan de Datos del Sistema](./05-Referencia/05-plan-datos-sistema.md)** — Diagnósticos de hardware evaluados localmente y políticas de privacidad.
