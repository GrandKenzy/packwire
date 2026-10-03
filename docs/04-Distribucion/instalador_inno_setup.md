# Instalador Nativo de Windows (Inno Setup)

Para la distribución formal en entornos empresariales, institucionales o para usuarios finales que esperan un asistente clásico de instalación de Windows con interfaz tipo asistente paso a paso, Packwire incluye la especificación formal para **Inno Setup** en [`installer.iss`](file:///c:/Users/Kentucky/Desktop/PROYECTOS%20_%20PYTHON/packwire/installer.iss).

Este instalador genera el ejecutable `dist/Packwire-Setup.exe`, registrando formalmente la aplicación en el catálogo de Windows (*Configuración > Aplicaciones instaladas* o *Panel de control > Programas y características*) e implementando rutinas seguras de desinstalación y modificación del entorno del sistema.

---

## ⚙️ Especificación Técnica de `installer.iss`

### Metadatos y Configuración Global

```ini
[Setup]
AppId={{E58C3729-28BC-4375-9C4B-0BE9FD162121}
AppName=Packwire
AppVersion=1.0.0
AppPublisher=Packwire Team
AppPublisherURL=https://github.com/packwire/packwire
DefaultDirName={localappdata}\Programs\Packwire
DefaultGroupName=Packwire
OutputDir=dist
OutputBaseFilename=Packwire-Setup
SetupIconFile=packwire\core\gui\web\logo.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ChangesEnvironment=yes
```

#### Parámetros Clave de Seguridad
* `PrivilegesRequired=lowest`: Instala la aplicación en el espacio del usuario local (`{localappdata}\Programs\Packwire`) por defecto. Esto permite que cualquier usuario en computadoras corporativas o con permisos restringidos instale y use Packwire sin requerir autorización de un Administrador de TI.
* `PrivilegesRequiredOverridesAllowed=dialog`: Si un usuario con privilegios de administrador ejecuta el instalador, este le permite elegir si desea instalar la aplicación para todos los usuarios del equipo o solo para el usuario en sesión.

---

## 🔍 Tareas Configurables (`[Tasks]`)

El asistente ofrece al usuario las siguientes opciones mediante casillas de verificación:

| Tarea | Descripción | Estado Inicial | Acción Técnica |
| :--- | :--- | :--- | :--- |
| `desktopicon` | Crear un acceso directo en el Escritorio | Marcada | Genera el acceso directo `{autodesktop}\Packwire.lnk` apuntando a `packwire.exe ui`. |
| `addtopath` | Añadir Packwire al `PATH` del usuario | Desmarcada | Invoca el script Pascal interno para concatenar la ruta en `HKCU\Environment\Path`. |

---

## ⚙️ Código Pascal Embebido (`[Code]`)

Para evitar dependencias de binarios externos durante la instalación y remoción del `PATH`, `installer.iss` incluye rutinas nativas en lenguaje Pascal de Inno Setup:

```pascal
const EnvironmentKey = 'Environment';

function AddPathToUserEnvironment(): Boolean;
var
  Paths: string;
  AppDir: string;
begin
  Result := True;
  AppDir := ExpandConstant('{app}');
  if RegQueryStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', Paths) then
  begin
    if Pos(';' + Uppercase(AppDir) + ';', ';' + Uppercase(Paths) + ';') = 0 then
    begin
      Paths := Paths + ';' + AppDir;
      RegWriteStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', Paths);
    end;
  end;
end;
```

Durante la desinstalación (`usPostUninstall`), el procedimiento complementario `RemovePathFromUserEnvironment()` rastrea y elimina la cadena de `{app}`, asegurando que el registro de Windows quede 100% limpio.

---

## 📦 Rutinas de Post-Instalación y Desinstalación

### 1. Inicialización Inmediata (`[Run]`)
Al finalizar la copia de archivos, el instalador ejecuta de forma silenciosa el comando interno:
```ini
Filename: "{app}\packwire.exe"; Parameters: "setup --no-start-menu"; Flags: runhidden
```
Esto inicializa la carpeta de shims `%APPDATA%\packwire\shims`, garantizando que el entorno esté completamente listo antes de que el usuario abra la aplicación.

### 2. Desinstalación Profunda (`[UninstallRun]`)
Al invocar `unins000.exe`, el asistente desinstalador ejecuta:
```ini
Filename: "{app}\packwire.exe"; Parameters: "uninstall --all"; Flags: runhidden
```
Esto asegura que todos los paquetes que el usuario haya descargado a lo largo del tiempo, junto con sus shims y cachés temporales, sean purgados íntegramente del disco duro antes de eliminar el ejecutable principal.

---

## ⚠️ Requisitos para la Compilación

1. **Herramienta Requerida:** [Inno Setup 6.0 o superior](https://jrsoftware.org/isdl.php).
2. **Pre-requisito:** Debe haberse ejecutado previamente `packwire build --exe` para que el archivo `dist/packwire.exe` esté disponible.

### Comando de Compilación

```bash
# Compilar el asistente desde consola
iscc installer.iss
```

El proceso generará `dist/Packwire-Setup.exe` comprimido con el algoritmo LZMA2 en ultra-compresión.
