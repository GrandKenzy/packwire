# Arquitectura de la Interfaz Gráfica

La GUI de Packwire está implementada en Rust dentro de `packwire-rs/src/gui.rs`. El frontend es una SPA HTML/CSS/JavaScript ubicada en `assets/web/` y se muestra mediante `wry`/WebView2 en Windows y el motor WebKit disponible en Linux o macOS.

## Ejecutables

- `packwire.exe`: CLI con consola.
- `packwirew.exe`: GUI sin consola.

El backend inicia un servidor HTTP local en `127.0.0.1:5050`, sirve los recursos de `assets/web/` y expone las operaciones de Packwire como endpoints JSON. La GUI nativa carga esa dirección dentro del WebView del sistema.

Si el puerto está ocupado, el backend selecciona otro puerto disponible. La variable `PACKWIRE_WEB_DIR` permite indicar una carpeta de recursos alternativa durante desarrollo o empaquetado.

## Recursos

```text
assets/web/
├── index.html
├── app.js
├── styles.css
├── global/
└── guides/
```

La aplicación no usa Electron, Python, PyInstaller ni un servidor web externo.
