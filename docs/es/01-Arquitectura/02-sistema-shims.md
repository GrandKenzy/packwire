# Sistema de Shims y PATH

El sistema de shims está implementado en `packwire-rs/src/pather.rs`. Cada paquete gestionado puede exponer lanzadores `.cmd` y `.ps1` en Windows o scripts POSIX en Linux y macOS.

Los shims se almacenan en `%APPDATA%\packwire\shims` en Windows y en `~/.local/share/packwire/shims` en sistemas POSIX. `packwire path` comprueba el estado y `packwire path --add` registra la ruta en el entorno del usuario.

En Windows, el backend usa el Registro de usuario y notifica `WM_SETTINGCHANGE` para que las nuevas terminales reciban el cambio.
