# Arquitectura del Sistema

La sección de Arquitectura expone los fundamentos estructurales, directrices de diseño y patrones de ingeniería que rigen la operación interna de Packwire. El sistema ha sido concebido bajo principios de modularidad estricta, determinismo en el estado del entorno de ejecución e idempotencia en las operaciones sobre el sistema de archivos anfitrión (Windows, Linux y macOS).

A través de esta sección se analizan las interacciones entre los componentes del núcleo, la estrategia de desacoplamiento de dependencias y los mecanismos implementados para garantizar la integridad del sistema operativo anfitrión sin requerir privilegios de superusuario para las operaciones cotidianas.

---

## Módulos y Especificaciones

* **[Flujo de Ejecución](flujo_ejecucion.md):** Traza integral del ciclo de vida de un paquete desde su resolución e identificación hasta la compilación de shims y registro en la base de estado transaccional.
* **[Sistema de Shims y PATH](sistema_shims.md):** Mecánica de proxying de binarios mediante scripts ligeros `.cmd` y `.ps1`, inyección no intrusiva en `HKCU\Environment` y difusión del mensaje `WM_SETTINGCHANGE`.
* **[Cola de Tareas y Concurrencia](cola_tareas.md):** Arquitectura del subsistema de trabajadores asíncronos (`TaskQueue`), gestión de hilos de ejecución, buffers de registro cronometrados y despacho no bloqueante.

---

## Flujo de Trabajo

El flujo global de procesamiento de una orden de instalación dentro de Packwire sigue una secuencia rigurosamente orquestada:

```
[Usuario / CLI / GUI]
         │
         ▼
[packwire.install()] ───────────► [ManifestVisitor]
         │                               │
         │ (Carga y validación)          ▼
         ├───────────────────────► [Resolver] (Resolución dinámica upstream)
         │                               │
         │ (Manifiesto validado)         ▼
         ▼
  [Installer] ───────────────────► [Collision Engine] (state.py)
         │                               │ (Valida colisiones)
         ├───────────────────────────────┘
         ▼
  [Modo de Despliegue]
         │
         ├───► [here]    ────► [DownloaderVisitor] ──► Extracción atómica ──┐
         ├───► [command] ────► [Subprocess / UAC]  ──► Script / CLI        ─┤
         └───► [site]    ────► [WebBrowser]        ──► Navegación externa  ─┤
                                                                            │
         ┌──────────────────────────────────────────────────────────────────┘
         ▼
  [PatherVisitor] ───────────────► Generación de Shims (.cmd / .ps1)
         │                        en %APPDATA%/packwire/shims
         ▼
  [Registro de Estado] ──────────► Escritura atómica en state.json
         │
         ▼
  [Fin de Operación] (Notificación a TaskQueue / Salida de Consola)
```
