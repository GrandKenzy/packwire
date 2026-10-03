# Módulos del Núcleo

Esta sección documenta la especificación funcional y el diseño de clases que integran el paquete [`packwire.core`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core). En esta capa residen las estructuras de datos fundamentales, los motores de resolución de versiones, la lógica de despliegue y los patrones de comportamiento que procesan las aplicaciones en el sistema.

El diseño modular aísla las responsabilidades de cada componente, facilitando la incorporación de nuevos lenguajes y herramientas al catálogo sin alterar el pipeline general de ejecución.

---

## Módulos y Especificaciones

* **[Modelos y Manifiestos](modelos_manifiestos.md):** Especificación de contratos de datos (`Manifest`, `InstallConfig`, `PackageState`), cálculo determinista de hash criptográfico SHA-256 e invariantes de serialización.
* **[Resolución de Versiones](resolucion_versiones.md):** Motores de descubrimiento upstream (`PythonResolver`, `NodeResolver`) y resolución en caliente para canales dinámicos `stable` y versiones ancladas `fixed`.
* **[Modos de Instalación](instalador_modos.md):** Comportamiento detallado de los tres modos de despliegue soportados: extracción atómica local (`here`), ejecución de scripts con elevación de privilegios (`command`) y enlace asistido a la web oficial (`site`).
* **[Subsistema de Visitantes](visitantes.md):** Operaciones desacopladas mediante el patrón Visitor: `DownloaderVisitor`, `PatherVisitor`, `ManifestVisitor`, `UninstallerVisitor` y `UpdaterVisitor`.

---

## Flujo de Trabajo

La interacción entre los módulos del núcleo se organiza según el siguiente esquema de dependencias:

```
[packwire/core/manifests/*.json]
               │
               ▼
      [ManifestVisitor] ────────► [models.Manifest]
               │                         │
               ▼                         ▼
      [resolver.py] ────────────► [calculate_hash()]
               │                         │
               └───────────┬─────────────┘
                           ▼
                 [installer.Installer]
                           │
      ┌────────────────────┼────────────────────┐
      ▼                    ▼                    ▼
[install_here]      [install_command]     [install_site]
      │                    │                    │
[Downloader]        [Process / UAC]       [WebBrowser]
      │                    │                    │
      └────────────────────┼────────────────────┘
                           ▼
                     [PatherVisitor]
                           │
                           ▼
                  [state.register()] ──► [state.json]
```
