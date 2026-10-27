# 05. Referencia y Diagnósticos

> **Versión Actual:** `1.0.0`
> **Compatibilidad:** Rust nativo y WebView2/WebKit del sistema

Referencia técnica exhaustiva de la interfaz de comandos (CLI), el catálogo de diagnósticos y errores, las distribuciones Linux recomendadas y el plan de privacidad y telemetría de hardware.

---

## Contenido de la Sección

| Guía | Resumen |
| :--- | :--- |
| **[01. Interfaz de Línea de Comandos (CLI)](./01-cli.md)** | Manual formal de todos los comandos, subcomandos, argumentos posicionales y banderas de `packwire`. |
| **[03. Catálogo de Diagnósticos y Errores](./03-catalogo-errores.md)** | Códigos de error estandarizados, causas comunes y procedimientos de mitigación recomendados. |
| **[04. Catálogo Linux](./04-catalogo-linux.md)** | Catálogo de distribuciones oficiales Linux recomendadas según perfil de uso (servidores, programación, uso personal, PCs antiguas). |
| **[05. Plan de Datos del Sistema](./05-plan-datos-sistema.md)** | Descripción de la recolección de diagnósticos de hardware locales (CPU, GPU, RAM, UEFI) y garantías de privacidad. |

---

## Resumen Rápido

1. **CLI Completo:** Comandos para instalación (`install`), desinstalación (`remove`), reinstalación (`reinstall`), actualización (`update`), exploración (`available`, `list`, `info`), gestión de PATH (`path`), exclusión de Defender (`fix-defender`) y GUI (`ui`).
2. **Automatización:** Todas las operaciones están disponibles mediante la CLI nativa y códigos de salida estables.
3. **Privacidad:** La telemetría de hardware no recopila información personal, nombres de usuario ni credenciales; se almacena y evalúa únicamente de forma local.
