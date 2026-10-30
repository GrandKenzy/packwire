# Integración con el sistema

Packwire se distribuye como aplicación nativa. La integración del sistema se realiza desde el instalador de Inno Setup y desde el módulo Rust de PATH, sin Python ni scripts auxiliares.

## Windows

`Packwire-Setup.exe` instala:

- `packwire.exe`, la CLI nativa.
- `packwirew.exe`, la GUI nativa sin consola.
- `web/`, los recursos del dashboard.
- el catálogo sincronizado desde GitHub en `%APPDATA%\packwire\manifests`.
- accesos directos con `assets/web/front.ico` como icono.
- el directorio de Packwire en el PATH del usuario cuando se selecciona esa opción.

La instalación no requiere permisos administrativos: el destino predeterminado es `%LOCALAPPDATA%\Programs\Packwire`.

## PATH y shims

Los shims de paquetes gestionados se almacenan en:

```text
%APPDATA%\packwire\shims       # Windows
~/.local/share/packwire/shims   # Linux y macOS
```

La CLI permite comprobar o añadir el directorio de shims al PATH:

```text
packwire path
packwire path --add
```

La actualización del PATH de usuario en Windows usa el Registro (`HKCU`) y notifica el cambio al sistema mediante `WM_SETTINGCHANGE`.

## Compilación local

```powershell
cargo build --release --manifest-path packwire-rs/Cargo.toml --bins
```

Para construir el instalador, abre `installer.iss` con Inno Setup después de compilar los binarios.
