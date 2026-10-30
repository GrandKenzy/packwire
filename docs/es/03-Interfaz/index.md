# 03. Interfaz de Usuario

> **Versión Actual:** `1.2.0`
> **Motor:** Microsoft Edge WebView2 (Windows), WebKitGTK (Linux), Safari WebKit (macOS)

Esta sección documenta la arquitectura del frontend webview moderno de Packwire, su puente bidireccional IPC, el servidor HTTP de contingencia y el sistema dinámico de temas visuales.

---

## Contenido de la Sección

| Guía | Resumen |
| :--- | :--- |
| **[01. Arquitectura GUI](./01-arquitectura-gui.md)** | Integración nativa de WebView2/WebKit mediante Rust, servidor HTTP local, ciclo de vida de la ventana y ausencia de consola (`packwirew.exe`). |
| **[02. Puente IPC y Comunicación](./02-puente-ipc.md)** | Especificación de la interfaz JSON entre JavaScript y Rust, sincronización de estado y emisión de telemetría. |
| **[03. Sistema de Temas y Estilos](./03-temas-estilos.md)** | Arquitectura modular de estilos CSS (`structure.css`, backgrounds, cards, buttons) y persistencia de preferencias en `configs.json`. |

---

## Resumen Rápido

1. **Sin sobrecoste de Electron:** Utiliza el motor Webview nativo del sistema operativo, reduciendo drásticamente el consumo de memoria RAM y el tamaño del binario.
2. **Ventana sin consola:** Se ejecuta mediante el binario separado `packwirew.exe`.
3. **Tematización Dinámica:** Hojas de estilo modulares que permiten alternar temas (Claro / Oscuro) sin recargar la aplicación.
