; =====================================================================
; Packwire Inno Setup Script
; Genera el instalador gráfico nativo de Windows (Packwire-Setup.exe)
; Requiere Inno Setup 6+: https://jrsoftware.org/isdl.php
; =====================================================================

#define MyAppName "Packwire"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Packwire Team"
#define MyAppURL "https://github.com/packwire/packwire"
#define MyAppExeName "packwire.exe"

[Setup]
AppId={{E58C3729-28BC-4375-9C4B-0BE9FD162121}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={localappdata}\Programs\Packwire
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=dist
OutputBaseFilename=Packwire-Setup
SetupIconFile=packwire\core\gui\web\logo.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ChangesEnvironment=yes
DisableProgramGroupPage=auto

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "addtopath"; Description: "Añadir Packwire a la variable de entorno PATH del usuario"; GroupDescription: "Configuración del Sistema:"; Flags: unchecked

[Files]
Source: "dist\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "packwire\core\gui\web\logo.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Parameters: "ui"; IconFilename: "{app}\logo.ico"; Comment: "Packwire Package Manager"
Name: "{group}\Packwire CLI (Consola)"; Filename: "{cmd}"; Parameters: "/k ""{app}\{#MyAppExeName}"" --help"; IconFilename: "{app}\logo.ico"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Parameters: "ui"; IconFilename: "{app}\logo.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Parameters: "setup --no-start-menu"; StatusMsg: "Configurando shims y entorno de Packwire..."; Flags: runhidden
Filename: "{app}\{#MyAppExeName}"; Parameters: "ui"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{app}\{#MyAppExeName}"; Parameters: "uninstall --all"; RunOnceId: "PackwireCleanAll"; Flags: runhidden

[Code]
// Gestión de PATH en Inno Setup
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
  end
  else
  begin
    RegWriteStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', AppDir);
  end;
end;

procedure RemovePathFromUserEnvironment();
var
  Paths: string;
  AppDir: string;
  P: Integer;
begin
  AppDir := ExpandConstant('{app}');
  if RegQueryStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', Paths) then
  begin
    P := Pos(';' + Uppercase(AppDir) + ';', ';' + Uppercase(Paths) + ';');
    if P > 0 then
    begin
      Delete(Paths, P, Length(AppDir) + 1);
      RegWriteStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', Paths);
    end;
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and IsTaskSelected('addtopath') then
  begin
    AddPathToUserEnvironment();
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    RemovePathFromUserEnvironment();
  end;
end;
