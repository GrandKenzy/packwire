# Modos de Instalación: Here, Command y Site

Packwire ofrece una arquitectura de instalación híbrida que permite adaptar la estrategia de aprovisionamiento a la naturaleza de cada software. Se implementa en [`packwire/core/installer.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/core/installer.py) a través de tres métodos especializados de la clase `Installer`: `install_here()`, `install_command()` e `install_site()`.

Esta versatilidad permite desde despliegues portables 100% aislados y desatendidos, hasta la ejecución de scripts oficiales que demandan privilegios de Administrador o la derivación asistida hacia instaladores gráficos oficiales.

---

## ⚙️ Comparativa de Modos

| Característica | Modo `here` | Modo `command` | Modo `site` |
| :--- | :--- | :--- | :--- |
| **Objetivo Principal** | Entornos portables, compiladores y utilidades CLI. | Gestores de paquetes y SDKs con instalador por script. | Suites pesadas y editores gráficos (ej. VS Code). |
| **Intervención del Usuario**| Cero (100% silencioso / headless). | Mínima (puede solicitar elevación UAC en Windows). | Manual guiada en el navegador web. |
| **Descarga Directa** | Sí (mediante `DownloaderVisitor`). | Delegada al script o comando ejecutado. | No (gestionada por el navegador del usuario). |
| **Destino en Disco** | `%APPDATA%\packwire\apps\<id>\` | Depende del instalador (ej. `C:\ProgramData\`). | N/A (Gestionado por el instalador oficial). |
| **Generación de Shims** | Sí (automática a partir de `binaries`). | Opcional (si se definen binarios en el manifiesto). | No. |
| **Reversibilidad** | 100% limpia (elimina carpeta y shims). | Depende del comando de desinstalación. | Manual por el usuario. |

---

## 🔍 Análisis Detallado por Modo

### 1. Modo `here` (Extracción Local Desatendida)

Es el modo predeterminado de Packwire. Diseñado para herramientas portables (como Python embed, GCC w64devkit, FFmpeg o Node.js zip):

```
[URL Remota] ──► [Downloader] ──► [Cache .zip] ──► [Extracción en apps/<id>/]
                                                             │
                                                             ▼
                                                    [Búsqueda de Binarios]
                                                             │
                                                             ▼
                                                    [Compilación de Shims]
```

1. **Descarga Atómica:** El archivo comprimido es descargado en `%APPDATA%\packwire\cache\`.
2. **Limpieza Previa:** Si `clean=True`, el directorio destino anterior es removido íntegramente para evitar residuos de versiones previas.
3. **Descompresión:** Se procesa automáticamente según el formato (`zip`, `tar.gz`, `tar.xz` o binario directo) usando los módulos estándar `zipfile` y `tarfile`.
4. **Normalización de Carpetas Anidadas:** Si el ZIP contiene una única carpeta raíz que envuelve los binarios (ej. `w64devkit/bin/gcc.exe`), `PatherVisitor` rastrea recursivamente la ubicación real de los ejecutables declarados.
5. **Inyección de Shims:** Se compilan los accesos `.cmd` y `.ps1` en `%APPDATA%\packwire\shims`.

### 2. Modo `command` (Ejecución de Scripts de Consola)

Diseñado para herramientas que cuentan con pipelines oficiales basados en consola o que requieren elevar privilegios en Windows (como Chocolatey, Rustup o MSYS2).

La configuración reside en la estructura `CommandConfig`:
```json
{
  "install": {
    "mode": "command",
    "command": {
      "elevated": true,
      "shell": "powershell",
      "windows": "Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))",
      "binaries": ["choco.exe"]
    }
  }
}
```

#### Manejo de Elevación de Privilegios (UAC)
Si `command.elevated` es `True`:
1. El instalador evalúa si el proceso actual ya cuenta con privilegios de Administrador mediante la API de Windows `ctypes.windll.shell32.IsUserAnAdmin()`.
2. Si no es administrador, ejecuta PowerShell invocando el verbo `RunAs` para disparar el diálogo nativo de Control de Cuentas de Usuario (UAC):
   ```powershell
   Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -Command <script>" -Wait
   ```
3. Si el usuario cancela la solicitud de elevación, el sistema captura el error Win32 `1223` (`ERROR_CANCELLED`) y aborta limpiamente informando que la operación fue revocada.

### 3. Modo `site` (Redirección Web Asistida)

Diseñado para aplicaciones de escritorio complejas (como Visual Studio Code) donde se recomienda que el usuario utilice el instalador GUI oficial de Microsoft.

1. El instalador no descarga binarios ni ejecuta procesos en segundo plano.
2. Invoca `webbrowser.open(site_url)`.
3. Registra en `state.json` la entrada con `mode: "site"` y `install_path: null`.
4. En el dashboard de Packwire, la tarjeta del software pasa a figurar como gestionada/instalada permitiendo registrar enlaces de acceso directo.

---

## ⚠️ Consideraciones Críticas y Casos de Borde

1. **Rutas Anidadas en Paquetes Comprimidos:** Muchos proyectos empaquetan su contenido dentro de subcarpetas arbitrarias (ej. `ffmpeg-7.1-essentials_build/bin/ffmpeg.exe`). El algoritmo de búsqueda de binarios en `install_here()` efectúa una búsqueda recursiva (`Path.rglob()`) para asegurar que el shim siempre apunte a la ruta física real.
2. **Reversibilidad en Modo Command:** Dado que un comando puede instalar software a nivel global del sistema, Packwire registra los binarios generados para remover los shims locales, pero la desinstalación física del software de terceros debe realizarse a través del panel de control de Windows si dicho paquete no incluye un script inverso.

---

## 💡 Ejemplo de Uso

```python
import packwire

# 1. Instalación autónoma en modo 'here'
ok, msg = packwire.install("c", mode="here")
print(f"Modo Here: {ok} -> {msg}")

# 2. Instalación por comandos oficiales con elevación
ok, msg = packwire.install("chocolatey", mode="command")
print(f"Modo Command: {ok} -> {msg}")

# 3. Instalación asistida mediante portal web oficial
ok, msg = packwire.install("vscode", mode="site")
print(f"Modo Site: {ok} -> {msg}")
```
