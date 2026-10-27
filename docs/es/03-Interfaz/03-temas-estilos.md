# Sistema Modular de Estilos y Temas Visuales

El subsistema de estilos de Packwire organiza la capa visual mediante una arquitectura modular en CSS3 sin preprocesadores pesados. Reside en [`packwire/core/gui/web/`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/gui/web) y se estructura a través del directorio [`global/`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/gui/web/global), separando estrictamente la estructura espacial de las paletas cromáticas.

Esta separación permite conmutar en caliente entre temas (Claro y Oscuro) en tiempo real, manteniendo la persistencia en `configs.json` para que la elección visual del usuario se conserve intacta entre ejecuciones.

---

## ⚙️ Estructura de Archivos de Estilo

```
packwire/core/gui/web/
├── global.css                 <-- Entrypoint global y variables raíz
├── styles.css                 <-- Reglas generales del dashboard y animaciones
└── global/
    ├── structure.css          <-- Estructura permanente (grid, flex, scrollbars)
    ├── default_background.css <-- Paleta tema claro (fondo y tipografía)
    ├── default_cards.css      <-- Tarjetas tema claro y bordes
    ├── default_buttons.css    <-- Botones e interacciones tema claro
    ├── default_layout.css     <-- Márgenes y rejillas tema claro
    ├── dark_background.css    <-- Paleta tema oscuro (#0F172A)
    ├── dark_cards.css         <-- Tarjetas tema oscuro (#1E293B) y resplandores
    └── dark_buttons.css       <-- Botones e interacciones tema oscuro
```

---

## 🔍 Módulos de Estilo

### 1. Estructura Invariante (`structure.css`)
Define la disposición espacial que permanece constante con independencia del tema seleccionado:
* **Rejilla del Dashboard:** Grid adaptativo con columnas `repeat(auto-fill, minmax(280px, 1fr))` para presentación de las tarjetas de software.
* **Barra Lateral de Navegación:** Contenedor flexible fijo con ancho de 240px, enlaces de sección y botones de estado.
* **Cajón de Telemetría (Drawer de Logs):** Panel inferior expandible donde se proyectan los registros cronometrados de la cola de tareas.
* **Scrollbar Moderno:** Reglas universales para estilizar las barras de desplazamiento en motores WebKit/Blink (Edge WebView2), eliminando las barras anchas clásicas de Windows:
  ```css
  ::-webkit-scrollbar {
      width: 6px;
      height: 6px;
  }
  ::-webkit-scrollbar-track {
      background: transparent;
  }
  ::-webkit-scrollbar-thumb {
      background: rgba(148, 163, 184, 0.3);
      border-radius: 9999px;
  }
  ::-webkit-scrollbar-thumb:hover {
      background: rgba(148, 163, 184, 0.6);
  }
  ```

### 2. Paleta Tema Claro (`default_*.css`)
* Fondo general: `#F8FAFC` (Slate 50).
* Tarjetas: Fondo `#FFFFFF` con borde sutil `#E2E8F0` y elevación suave por sombra `box-shadow: 0 1px 3px rgba(0,0,0,0.1)`.
* Tipografía: Tonos oscuros de alto contraste `#0F172A` (Slate 900).

### 3. Paleta Tema Oscuro (`dark_*.css`)
* Fondo general: `#0F172A` (Slate 900 profundo).
* Tarjetas: Fondo `#1E293B` (Slate 800) con bordes `#334155` y efecto de resplandor sutil al enfocar o pasar el cursor (`:hover`).
* Tipografía: Tonos claros `#F1F5F9` (Slate 100) y acentos cian/esmeralda para insignias de estado.

---

## ⚙️ Conmutación Dinámica y Persistencia

El cambio de tema se procesa en el cliente mediante JavaScript y se sincroniza con el backend:

```javascript
// packwire/core/gui/web/app.js
async function cambiarTema(nuevoTema) {
    document.body.classList.remove('theme-default', 'theme-dark');
    document.body.classList.add(`theme-${nuevoTema}`);
    
    // Persistir preferencia en configs.json a través de GuiBridge
    if (window.pywebview && window.pywebview.api) {
        await window.pywebview.api.set_config('theme', nuevoTema);
    } else {
        await fetch('/api/config', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ key: 'theme', value: nuevoTema })
        });
    }
}
```

Al inicializar la interfaz, `app.js` consulta `get_config('theme')`. Si existe una preferencia guardada, aplica la clase correspondiente al elemento `<body>` antes del primer ciclo de renderizado, previniendo el parpadeo de contenido no estilizado (FOUC).

---

## ⚠️ Consideraciones Críticas y Casos de Borde

1. **Resolución en Modo Compilado (`_MEIPASS`):** Al distribuir Packwire como ejecutable único (`packwire.exe`), los archivos CSS son extraídos en la carpeta temporal de PyInstaller. `bridge.py` y `server.py` resuelven la ruta absoluta mediante `get_web_dir()` y `get_global_css_dir()`, asegurando que todas las hojas de estilo modulares se sirvan correctamente sin errores 404.
2. **Iconografía Sobria y Libre de Emojis en Botones:** En conformidad con el diseño moderno, los botones de acción utilizan etiquetas de texto claras (`Instalar`, `Reinstalar`, `Desinstalar`, `Actualizar`) con iconos tipográficos técnicos o SVG limpios, eliminando el uso de emojis informales en los controles de interacción.

---

## 💡 Ejemplo de Declaración CSS en Tema Oscuro

```css
/* packwire/core/gui/web/global/dark_cards.css */
body.theme-dark .package-card {
    background: #1E293B;
    border: 1px solid #334155;
    border-radius: 12px;
    padding: 1.25rem;
    transition: transform 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
}

body.theme-dark .package-card:hover {
    transform: translateY(-2px);
    border-color: #38BDF8;
    box-shadow: 0 10px 25px -5px rgba(56, 189, 248, 0.15);
}
```
