# 01. Arquitectura del Sistema

> **Versión Actual:** `0.1.4`
> **Plataformas:** Windows (Win32), Linux (POSIX), macOS (Darwin)

Esta sección expone los fundamentos estructurales, directrices de diseño y patrones de ingeniería que rigen la operación interna de Packwire. El sistema ha sido concebido bajo principios de modularidad estricta, determinismo en el estado del entorno de ejecución e idempotencia en las operaciones sobre el sistema de archivos anfitrión.

---

## Contenido de la Sección

| Guía | Resumen |
| :--- | :--- |
| **[01. Flujo de Ejecución](./01-flujo-ejecucion.md)** | Traza integral del ciclo de vida de un paquete desde su resolución e identificación hasta la compilación de shims y registro en la base de estado transaccional. |
| **[02. Sistema de Shims y PATH](./02-sistema-shims.md)** | Mecánica de proxying de binarios mediante scripts ligeros `.cmd` y `.ps1`, inyección no intrusiva en `HKCU\Environment` y difusión del mensaje `WM_SETTINGCHANGE`. |
| **[03. Cola de Tareas y Concurrencia](./03-cola-tareas.md)** | Arquitectura del subsistema de trabajadores asíncronos (`TaskQueue`), gestión de hilos de ejecución, buffers de registro cronometrados y despacho no bloqueante. |

---

## Resumen Rápido

1. **Aislamiento Centralizado:** En lugar de saturar el registro del sistema o archivos de perfil con múltiples carpetas en el `PATH`, Packwire mantiene una única carpeta central de lanzadores (`shims`).
2. **Determinismo:** Los paquetes son identificados mediante coordenadas canónicas semánticas (`<nombre>@<canal_o_versión>`), garantizando unicidad y evitando colisiones de ejecutables.
3. **Multiplataforma:** Soporte para Windows (PowerShell/UAC/Registry), Linux (`~/.bashrc`, XDG `.desktop`, scripts POSIX) y macOS (`~/.zshrc`, Safari WebKit, `osascript`).
