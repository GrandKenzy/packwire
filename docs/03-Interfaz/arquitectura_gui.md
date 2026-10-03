# Arquitectura de la Interfaz Gráfica (GUI)

La arquitectura de la interfaz gráfica de Packwire implementa un patrón desacoplado entre el motor de renderizado y el núcleo de servicios. Se distribuye entre [`packwire/core/gui/window.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/gui/window.py) (lanzador de ventana nativa) y [`packwire/core/gui/server.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/gui/server.py) (servidor HTTP local y despachador de recursos estáticos).

El sistema cuenta con una estrategia de ejecución dual: puede desplegar una ventana nativa de escritorio acelerada por hardware o bien operar en modo servidor web local accesible desde cualquier navegador estándar.

---

## ⚙️ Especificación Técnica

### Firmas Principales

```python
# packwire/core/gui/window.py
def launch_gui(web_mode: bool = False, port: int = 5050) -> None: ...

# packwire/core/gui/server.py
def start_web_server(port: int = 5050, background: bool = False) -> int: ...
def get_web_dir() -> Path: ...
```

### Parámetros de `launch_gui`

| Parámetro | Tipo | Requerido | Descripción |
| :--- | :--- | :--- | :--- |
| `web_mode` | `bool` | No | Si es `True`, omite la ventana nativa y abre el panel en el navegador predeterminado. |
| `port` | `int` | No | Puerto TCP para el servidor HTTP local (por defecto `5050`). |

---

## 🔍 Modos de Ejecución

### 1. Modo Ventana Nativa (`pywebview`)

Es el modo predeterminado invocado mediante `packwire ui`:

```python
window = webview.create_window(
    title="Packwire - Modern Package Manager",
    url=f"http://127.0.0.1:{active_port}",
    width=1100,
    height=780,
    min_size=(900, 600),
    js_api=bridge,
    background_color="#0F172A"
)
webview.start(debug=False)
```

* **Motor en Windows:** Utiliza Microsoft Edge WebView2 (motor Chromium integrado en Windows 10 y Windows 11).
* **Consumo de Memoria:** Entre 40 MB y 70 MB de RAM (aproximadamente una quinta parte del consumo base de una aplicación Electron equivalente).
* **Inyección de API:** La propiedad `js_api` expone la instancia de `GuiBridge` directamente al objeto global `window.pywebview.api` dentro del contexto de JavaScript.

### 2. Modo Servidor Web Local (`server.py`)

Invocado cuando se activa la bandera `--web` o cuando el entorno del sistema carece de soporte de ventana nativa (por ejemplo, en sesiones remotas SSH o entornos virtuales):

1. **Selección de Motor HTTP:** Utiliza `bottle` si está disponible en el entorno; en caso contrario, conmuta limpiamente a una implementación multihilo basada en la biblioteca estándar `http.server.ThreadingHTTPServer`.
2. **Detección Dinámica de Puertos:** Si el puerto especificado (ej. `5050`) se encuentra ocupado por otro proceso, prueba incrementalmente puertos consecutivos (`5051`, `5052`, etc.) hasta encontrar un socket disponible.
3. **Despacho Estático y Modo Frozen:** La función `get_web_dir()` detecta si la aplicación se ejecuta dentro de un binario compilado por PyInstaller:
   ```python
   def get_web_dir() -> Path:
       if getattr(sys, "frozen", False) and hasattr(sys, "_MEIPASS"):
           return Path(sys._MEIPASS) / "packwire" / "core" / "gui" / "web"
       return Path(__file__).resolve().parent / "web"
   ```
4. **Lanzamiento:** Abre la URL local en el navegador predeterminado del sistema mediante `webbrowser.open()`.

---

## ⚠️ Consideraciones Críticas y Casos de Borde

1. **Ausencia de WebView2 Runtime en Windows Antiguos:** En instalaciones obsoletas de Windows 10 sin el componente WebView2 Runtime, `pywebview` puede arrojar una excepción de inicialización. `launch_gui()` captura el error e inicia automáticamente el servidor web local abriendo el navegador del sistema como mecanismo de contingencia transparente.
2. **Cierre Limpio y Liberación de Sockets:** El servidor HTTP local se inicia en un hilo demonio (`daemon=True`), garantizando que al cerrar la ventana principal de la interfaz o enviar una señal `SIGINT` (Ctrl+C), el puerto TCP sea liberado inmediatamente.

---

## 💡 Ejemplo de Invocación desde Python

```python
import packwire

# Lanzar ventana nativa en resolución 1100x780
packwire.launch_gui()

# O forzar ejecución en navegador web en un puerto específico
packwire.launch_gui(web_mode=True, port=8080)
```
