# 02. Módulos del Núcleo

> **Versión Actual:** `0.1.4`
> **Paquete:** `packwire.core`

La sección de Módulos detalla los contratos de datos, el motor de resolución dinámica de versiones, los tres modos de instalación y los componentes de procesamiento basados en el patrón Visitor.

---

## Contenido de la Sección

| Guía | Resumen |
| :--- | :--- |
| **[01. Modelos y Manifiestos](./01-modelos-manifiestos.md)** | Especificación de contratos de datos (`Manifest`, `InstallConfig`, `PackageState`), validación de campos y cálculo del hash determinista SHA-256. |
| **[02. Resolución de Versiones](./02-resolucion-versiones.md)** | Conexión con upstream oficial (python.org, nodejs.org, releases de GitHub) y resolución de canales `stable` vs `fixed`. |
| **[03. Modos de Instalación](./03-instalador-modos.md)** | Mecanismos operativos de los tres modos de despliegue: extracción directa desatendida (`here`), ejecución oficial (`command`) y navegación externa (`site`). |
| **[04. Subsistema de Visitantes](./04-visitantes.md)** | Arquitectura y ciclo de vida de `DownloaderVisitor`, `PatherVisitor`, `UninstallerVisitor` y `UpdaterVisitor`. |

---

## Resumen Rápido

1. **Canales de Versión:** `stable` consulta los índices oficiales para resolver dinámicamente el release más reciente; `fixed` garantiza inmutabilidad fijando versiones específicas.
2. **Modos de Despliegue:** `here` para despliegues portables y headless; `command` para instaladores nativos con elevación; `site` para herramientas con asistentes interactivos complejos.
3. **Patrón Visitor:** Desacopla la lógica algorítmica (descarga, shims, desinstalación, actualización) de las estructuras de datos.
