; =====================================================================
; Packwire Inno Setup Script
; Genera el instalador gráfico nativo de Windows (Packwire-Setup.exe)
; Requiere Inno Setup 6+: https://jrsoftware.org/isdl.php
; =====================================================================

#define MyAppName "Packwire"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Packwire Team"
#define MyAppURL "https://github.com/GrandKenzy/packwire"
#define MyAppExeName "packwire.exe"
#define MyAppGuiExeName "packwirew.exe"
#ifndef BuildDir
#define BuildDir "packwire-rs\target\release"
#endif

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
SetupIconFile=assets\web\front.ico
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
Source: "{#BuildDir}\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#BuildDir}\{#MyAppGuiExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "assets\web\*"; DestDir: "{app}\web"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "assets\manifests\*.json"; DestDir: "{app}\manifests"; Flags: ignoreversion
Source: "assets\web\front.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppGuiExeName}"; IconFilename: "{app}\front.ico"; Comment: "Packwire"
Name: "{group}\Packwire CLI"; Filename: "{cmd}"; Parameters: "/k ""{app}\{#MyAppExeName}"" --help"; IconFilename: "{app}\front.ico"; Comment: "Packwire CLI"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppGuiExeName}"; IconFilename: "{app}\front.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppGuiExeName}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall skipifsilent

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
  Paths, PaddedPaths: string;
  AppDir: string;
  P: Integer;
begin
  AppDir := ExpandConstant('{app}');
  if RegQueryStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', Paths) then
  begin
    PaddedPaths := ';' + Paths + ';';
    P := Pos(';' + Uppercase(AppDir) + ';', Uppercase(PaddedPaths));
    if P > 0 then
    begin
      Delete(PaddedPaths, P, Length(AppDir) + 1);
      if (Length(PaddedPaths) > 0) and (PaddedPaths[1] = ';') then
        Delete(PaddedPaths, 1, 1);
      if (Length(PaddedPaths) > 0) and (PaddedPaths[Length(PaddedPaths)] = ';') then
        Delete(PaddedPaths, Length(PaddedPaths), 1);
      RegWriteStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', PaddedPaths);
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
