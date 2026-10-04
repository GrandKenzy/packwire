# Catálogo de Diagnósticos y Errores

Este documento recopila de manera sistemática los códigos de error, condiciones de fallo y diagnósticos emitidos por los diferentes subsistemas de Packwire durante sus operaciones de instalación, actualización, desinstalación y ejecución gráfica.

Para cada condición se describe su nivel de severidad, el subsistema de origen, la causa técnica fundamental y el procedimiento de mitigación recomendado.

---

## 📋 Matriz Formal de Diagnósticos y Errores

| Código | Severidad | Subsistema | Causa Raíz | Mitigación Técnica |
| :--- | :--- | :--- | :--- | :--- |
| `ERR_PKG_NOT_FOUND` | Error | ManifestVisitor | No existe ningún manifiesto que coincida con el nombre, identificador o alias solicitado. | Verificar el identificador mediante `packwire available` o comprobar la sintaxis en el catálogo JSON. |
| `ERR_COLLISION` | Error | Collision Engine | Ya existe una versión instalada con el mismo nombre y ejecutables en conflicto. | Utilizar el parámetro `--force` para sobrescribir o especificar una versión fija diferente. |
| `ERR_DOWNLOAD_INCOMPLETE`| Error | DownloaderVisitor | La conexión de red se interrumpió antes de recibir la totalidad de bytes declarados en `Content-Length`. | Reintentar la operación. El fragmento parcial `.part` se descarta automáticamente para evitar corrupción. |
| `ERR_HASH_MISMATCH` | Crítico | DownloaderVisitor | La suma de verificación SHA-256 del binario descargado difiere del hash esperado en el manifiesto. | Comprobar si el archivo en el servidor remoto fue modificado por el upstream o si la descarga está corrompida. |
| `ERR_FILE_LOCKED` | Error | Installer / Pather | Un proceso activo de Windows (ej. terminal ejecutando `python.exe` o `gcc.exe`) mantiene bloqueado el archivo. | Cerrar todas las consolas, terminales o IDEs que tengan instancias del binario en ejecución y reintentar. |
| `ERR_UAC_CANCELLED` | Advertencia | Command Mode | El usuario canceló la ventana de elevación de privilegios de Control de Cuentas de Usuario (UAC - Win32 1223). | Aceptar el diálogo de elevación de Windows cuando se instalen herramientas de sistema como Chocolatey. |
| `ERR_PATH_REGISTRY` | Error | PatherVisitor | Fallo al abrir o escribir en la clave `HKCU\Environment` en el registro de Windows. | Verificar permisos del perfil de usuario en el registro o ejecutar `packwire path --add` en una terminal con permisos estándar. |
| `ERR_WEBVIEW_INIT` | Advertencia | GUI (Window) | El sistema carece de Microsoft Edge WebView2 Runtime o el entorno no soporta ventanas nativas. | Instalar WebView2 Runtime oficial o ejecutar Packwire en modo servidor web alternativo mediante `packwire ui --web`. |
| `ERR_UPSTREAM_TIMEOUT` | Advertencia | Resolver | El servidor upstream oficial (python.org, nodejs.org) no respondió dentro del timeout de 5 segundos. | El resolvedor conmuta a la versión de contingencia local precompilada; verificar la conexión a Internet o proxies corporativos. |
| `ERR_UNINSTALL_STATE` | Error | UninstallerVisitor | Se intentó desinstalar o reinstalar un paquete que no figura registrado en `state.json`. | Consultar los paquetes válidos instalados mediante `packwire list`. |
| `ERR_EXEC_POLICY` | Error | Scripts de Instalación | PowerShell bloquea la ejecución de `install.ps1` por política `Restricted`. | Ejecutar con `install.bat` o `powershell -ExecutionPolicy Bypass -File .\install.ps1`. |
| `ERR_PATH_STALE` | Advertencia | Shell / Terminal | La sesión de terminal actual no ha actualizado su variable `PATH` tras instalar. | Abrir una nueva terminal o ejecutar `source ~/.bashrc` / `source ~/.zshrc`. |

---

## 🔍 Protocolos Detallados de Mitigación

### 1. `ERR_COLLISION`: Gestión de Colisiones de Binarios
* **Condición:** Si el usuario ejecuta `packwire install python@3.14` teniendo previamente instalado `python@stable`, el motor de colisiones de `state.py` detecta que ambos paquetes compiten por el shim genérico `python.cmd`.
* **Solución:**
  * Si se desea sustituir la versión anterior como comando principal:
    ```bash
    packwire install python@3.14 --force
    ```
  * Si se desea que ambas convivan, Packwire compila automáticamente el shim versionado específico (`python314.cmd`), permitiendo invocar la versión exacta sin conflicto.

---

### 2. `ERR_FILE_LOCKED`: Error Win32 de Bloqueo de Archivos (Sharing Violation)
* **Condición:** En Windows, el sistema operativo prohíbe escribir o eliminar ejecutables (`.exe` o `.dll`) que estén siendo referenciados en memoria por un proceso en ejecución.
* **Solución:**
  1. Identificar si existe algún proceso huérfano en ejecución:
     ```powershell
     Get-Process python, gcc, node -ErrorAction SilentlyContinue
     ```
  2. Finalizar los procesos correspondientes y reintentar la operación de instalación o actualización.

---

### 3. `ERR_WEBVIEW_INIT`: Despliegue en Entornos Sin WebView2
* **Condición:** En versiones reducidas de Windows 10 LTSC o en máquinas virtuales mínimas de desarrollo, el componente WebView2 Runtime puede no encontrarse preinstalado.
* **Solución:**
  * Iniciar la interfaz en modo servidor web independiente:
    ```bash
    packwire ui --web
    ```
  * O descargar e instalar el instalador evergreen oficial de Microsoft WebView2 Runtime desde el sitio oficial de Microsoft Edge.
