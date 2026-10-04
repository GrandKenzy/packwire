# Referencia de API, CLI y Diagnósticos

Esta sección reúne los manuales formales de especificación técnica, contratos de invocación y catálogos de soporte para desarrolladores y operadores de sistemas que interactúan con Packwire.

A través de estos documentos se especifican exhaustivamente la sintaxis de todos los comandos de terminal, los métodos y tipos de la API pública de Python, y la matriz de diagnóstico de errores para resolución de incidencias en producción.

---

## Módulos y Especificaciones

* **[Interfaz de Línea de Comandos (CLI)](cli.md):** Manual formal exhaustivo de comandos de consola, subcomandos, argumentos posicionales y banderas opcionales.
* **[API de Python](api_python.md):** Especificación de firmas, parámetros tipados y valores de retorno de las funciones públicas exportadas en el paquete raíz `packwire`.
* **[Catálogo de Diagnósticos y Errores](catalogo_errores.md):** Matriz sistemática de códigos de error, condiciones de fallo, causas fundamentales y protocolos de mitigación técnica.

---

## Índice de Consulta Rápida

| Interfaz | Ámbito de Uso | Punto de Entrada |
| :--- | :--- | :--- |
| **CLI** | Automatización, terminal interactiva y scripts batch/PowerShell. | `packwire <comando> [opciones]` o `python -m packwire` |
| **API Python** | Integración en proyectos, scripts de DevOps y extensiones. | `import packwire` |
| **Diagnósticos** | Análisis de incidencias, logs de instalación y depuración. | `%APPDATA%\packwire\state.json` (Windows) o `~/.local/share/packwire/state.json` (POSIX) |
