# Manual de la Interfaz de Línea de Comandos (CLI)

La interfaz de línea de comandos de Packwire constituye el punto de acceso primordial para entornos automatizados, scripts de despliegue continuo (CI/CD) y administradores de sistemas. Se implementa en [`packwire/cli.py`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/packwire/cli.py) y puede ser invocada de manera equivalente como:

```bash
packwire <comando> [opciones]
# o mediante el módulo de Python:
python -m packwire <comando> [opciones]
```

---

## ⚙️ Estructura Global y Subcomandos

```
packwire [-h] {install,remove,reinstall,update,list,available,info,path,ui,setup,build} ...
```

### Códigos de Salida del Proceso (`Exit Codes`)

* `0`: Operación completada con éxito.
* `1`: Fallo durante la ejecución (error de sintaxis, paquete inexistente, discrepancia de suma de verificación o error de permisos).

---

## 🔍 Catálogo Exhaustivo de Comandos

### 1. `install`
Instala un paquete en el sistema resolviendo dependencias y compilando sus shims correspondientes.

```bash
packwire install <package> [--version <ver>] [--channel {stable,fixed}] [--mode {here,site}] [--force]
```

| Argumento / Bandera | Tipo | Requerido | Descripción |
| :--- | :--- | :--- | :--- |
| `package` | Posicional | Sí | Nombre, alias o identificador semántico del paquete (ej. `python`, `python@stable`, `python@3.14`, `c`, `ffmpeg`, `chocolatey`). |
| `--version`, `-v` | Opción | No | Fuerza una versión exacta específica (ej. `-v 3.12.8`). |
| `--channel` | Opción | No | Canal de distribución: `stable` (predeterminado) o `fixed`. |
| `--mode`, `-m` | Opción | No | Modo de despliegue: `here` (automático desatendido) o `site` (sitio oficial). |
| `--force`, `-f` | Bandera | No | Ignora colisiones preexistentes y fuerza la sobrescritura física de binarios y shims. |

```bash
# Ejemplos:
packwire install python
packwire install python@3.13 --mode here
packwire install c --force
```

---

### 2. `remove` (Alias: `uninstall`)
Desinstala un paquete individual o purga la totalidad del entorno Packwire.

```bash
packwire remove [<package>] [--all] [--keep-path]
```

| Argumento / Bandera | Tipo | Requerido | Descripción |
| :--- | :--- | :--- | :--- |
| `package` | Posicional | Condicional | Identificador del paquete a remover (ej. `python@stable`). Obligatorio salvo si se usa `--all`. |
| `--all` | Bandera | No | Purga absoluta: remueve todas las aplicaciones, shims, registros de estado, caché y variable PATH. |
| `--keep-path` | Bandera | No | Utilizada junto con `--all` para evitar eliminar la entrada de shims del registro `PATH`. |

```bash
# Ejemplos:
packwire remove python@stable
packwire remove --all
```

---

### 3. `reinstall`
Ejecuta una reinstalación limpia forzada de un paquete previamente registrado.

```bash
packwire reinstall <package>
```

```bash
# Ejemplo:
packwire reinstall c-gcc
```

---

### 4. `update`
Comprueba y aplica actualizaciones para paquetes configurados bajo el canal dinámico `stable`.

```bash
packwire update [<package>]
```

| Argumento / Bandera | Tipo | Requerido | Descripción |
| :--- | :--- | :--- | :--- |
| `package` | Posicional | No | Identificador del paquete a actualizar. Si se omite, analiza y actualiza todos los paquetes instalados. |

```bash
# Ejemplos:
packwire update
packwire update python@stable
```

---

### 5. `list`
Genera una tabla formateada en consola con todos los paquetes registrados en `state.json`.

```bash
packwire list
```

**Formato de salida:**
```text
ID                   NOMBRE       VERSIÓN    TIPO     MODO     RUTA
--------------------------------------------------------------------------------
python@stable        Python       3.14.0     stable   here     C:\...\apps\python@stable
c-gcc                C/C++ (GCC)  1.23.0     fixed    here     C:\...\apps\c-gcc
```

---

### 6. `available`
Imprime la lista de todos los manifiestos disponibles en el catálogo interno de Packwire.

```bash
packwire available
```

---

### 7. `info`
Inspecciona y proyecta el manifiesto normalizado de un paquete en formato JSON estructurado.

```bash
packwire info <package>
```

```bash
# Ejemplo:
packwire info ffmpeg
```

---

### 8. `path`
Verifica o configura el directorio de shims dentro de la variable de entorno `PATH` del usuario.

```bash
packwire path [--add]
```

| Bandera | Descripción |
| :--- | :--- |
| *(sin banderas)* | Muestra la ruta física del directorio de shims y comprueba si ya está presente en el `PATH`. |
| `--add` | Inyecta la ruta en `HKCU\Environment\Path` y difunde el evento de sistema `WM_SETTINGCHANGE`. |

---

### 9. `ui` (Alias: `gui`)
Inicia el panel de control gráfico moderno de Packwire.

```bash
packwire ui [--web] [--port <puerto>]
```

| Bandera | Tipo | Valor por Defecto | Descripción |
| :--- | :--- | :--- | :--- |
| `--web` | Bandera | `False` | Fuerza la ejecución en servidor HTTP local abriendo la interfaz en el navegador predeterminado. |
| `--port` | Opción | `5050` | Puerto TCP de enlace para el servidor web. |

---

### 10. `setup` (Alias: `self-install`)
Instala e integra Packwire en el sistema operativo Windows (shims maestros, `PATH` y accesos directos).

```bash
packwire setup [--desktop] [--no-start-menu] [--no-path]
```

| Bandera | Descripción |
| :--- | :--- |
| `--desktop` | Genera un acceso directo de la interfaz gráfica en el Escritorio del usuario actual. |
| `--no-start-menu` | Omite la creación del icono en el Menú Inicio de Windows. |
| `--no-path` | Omite la inyección del directorio de shims en la variable `PATH`. |

---

### 11. `build` (Alias: `pack`)
Compila y genera los artefactos de distribución del proyecto.

```bash
packwire build [--exe] [--wheel] [--all]
```

| Bandera | Descripción |
| :--- | :--- |
| `--exe` | Compila el binario ejecutable independiente `dist/packwire.exe` con PyInstaller. |
| `--wheel` | Construye el paquete de distribución universal Python (`.whl` y `.tar.gz`) en `dist/`. |
| `--all` | Compila secuencialmente tanto el ejecutable nativo como los paquetes wheel. |
