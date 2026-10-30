# 02. Módulos del Núcleo

> **Versión Actual:** `1.2.0`
> **Núcleo:** `packwire-rs/src/`

La sección de Módulos detalla los contratos de datos, el motor de resolución dinámica de versiones y los tres modos de instalación del núcleo Rust.

---

## Contenido de la Sección

| Guía | Resumen |
| :--- | :--- |
| **[01. Modelos y Manifiestos](./01-modelos-manifiestos.md)** | Especificación de contratos de datos (`Manifest`, `InstallConfig`, `PackageState`), validación de campos y cálculo del hash determinista SHA-256. |
| **[02. Resolución de Versiones](./02-resolucion-versiones.md)** | Conexión con upstream oficial (python.org, nodejs.org, releases de GitHub) y resolución de canales `stable` vs `fixed`. |
| **[03. Modos de Instalación](./03-instalador-modos.md)** | Mecanismos operativos de los tres modos de despliegue: extracción directa desatendida (`here`), ejecución oficial (`command`) y navegación externa (`site`). |

---

## Resumen Rápido

1. **Canales de Versión:** `stable` consulta los índices oficiales para resolver dinámicamente el release más reciente; `fixed` garantiza inmutabilidad fijando versiones específicas.
2. **Modos de Despliegue:** `here` para despliegues portables y headless; `command` para instaladores nativos con elevación; `site` para herramientas con asistentes interactivos complejos.
3. **Módulos especializados:** Descarga, shims, desinstalación y actualización están separados en módulos Rust con contratos tipados.
