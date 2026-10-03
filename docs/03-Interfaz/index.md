# Interfaz de Usuario y Presentación

Esta sección documenta la arquitectura de la interfaz gráfica de usuario (GUI) de Packwire, implementada en [`packwire/core/gui/`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/gui). El subsistema combina estándares web abiertos (**HTML5, CSS3 modular y JavaScript Vanilla**) con el motor nativo del sistema operativo mediante la biblioteca `pywebview`.

Esta arquitectura proporciona una experiencia de escritorio moderna, fluida y con aceleración por hardware (Microsoft Edge WebView2 en Windows), evitando el alto consumo de memoria RAM y almacenamiento característico de entornos basados en Electron.

---

## Módulos y Especificaciones

* **[Arquitectura GUI](arquitectura_gui.md):** Motor de ventana nativa con `pywebview`, ciclo de vida del servidor web interno en `server.py` y modo fallback de navegador.
* **[Puente IPC y Comunicación](puente_ipc.md):** Especificación del puente de comunicación bidireccional `GuiBridge`, API RPC expuesta a JavaScript y persistencia de configuración en `configs.json`.
* **[Sistema de Temas y Estilos](temas_estilos.md):** Arquitectura modular de hojas de estilo (`global.css`, `default_*.css`, `dark_*.css`), conmutación dinámica en caliente y diseño responsive.

---

## Flujo de Trabajo

El flujo de interacción entre la vista web y el núcleo de Python se estructura de la siguiente manera:

```
[Usuario en Interfaz Web]
         │
         │ (Clic en "Instalar", "Tema", "Desinstalar")
         ▼
    [app.js]
         │
         │ pywebview.api.<metodo>()  /  fetch('/api/<endpoint>')
         ▼
   [GuiBridge] (bridge.py)
         │
         ├───────────────────────┬────────────────────────┐
         ▼                       ▼                        ▼
[TaskQueue / Installer]   [State / Pather]        [configs.json]
(Instalaciones async)     (Gestión de paquetes)   (Persistencia de tema)
         │                       │                        │
         └───────────────────────┼────────────────────────┘
                                 ▼
                     Respuesta JSON estructurada
                                 │
                                 ▼
                    [Actualización del DOM]
                    (Barra de progreso, logs,
                     conmutación de clases CSS)
```
