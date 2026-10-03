<#
.SYNOPSIS
    Packwire Windows Installer Script
.DESCRIPTION
    Instala y configura Packwire en Windows:
    - Verifica el entorno de Python
    - Instala Packwire y sus dependencias
    - Genera los lanzadores shims en %APPDATA%\packwire\shims
    - Registra el directorio de shims en la variable PATH del usuario
    - Crea los accesos directos en el Menú Inicio y Escritorio
#>
[CmdletBinding()]
param(
    [switch]$DesktopShortcut,
    [switch]$NoStartMenu,
    [switch]$NoPath
)

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "             PACKWIRE WINDOWS INSTALLER                  " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

# 1. Verify Python
$pythonCmd = Get-Command python -ErrorAction SilentlyContinue
if (-not $pythonCmd) {
    Write-Host "[-] Python no fue detectado en el PATH." -ForegroundColor Red
    Write-Host "[!] Por favor instala Python 3.9+ o añádelo al PATH antes de continuar." -ForegroundColor Yellow
    exit 1
}

$pythonVersion = & python --version 2>&1
Write-Host "[+] Entorno Python: $pythonVersion ($($pythonCmd.Source))" -ForegroundColor Green

# 2. Check source location
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$isRepo = Test-Path "$scriptDir\packwire\__init__.py"

if ($isRepo) {
    Write-Host "[*] Instalando dependencias de Packwire en modo editable..." -ForegroundColor Cyan
    & python -m pip install -e $scriptDir --no-warn-script-location
    
    Write-Host "[*] Configurando shims, variables de entorno y accesos directos..." -ForegroundColor Cyan
    $setupArgs = @("-m", "packwire", "setup")
    if ($DesktopShortcut) { $setupArgs += "--desktop" }
    if ($NoStartMenu) { $setupArgs += "--no-start-menu" }
    if ($NoPath) { $setupArgs += "--no-path" }
    
    & python @setupArgs
} else {
    Write-Host "[*] Instalando Packwire..." -ForegroundColor Cyan
    & python -m pip install packwire --no-warn-script-location
    & python -m packwire setup
}

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Green
Write-Host "  ¡PACKWIRE HA SIDO INSTALADO Y CONFIGURADO CON ÉXITO!    " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
Write-Host ""
Write-Host "Para empezar a usar Packwire:" -ForegroundColor White
Write-Host "  • Terminal : packwire --help" -ForegroundColor Yellow
Write-Host "  • Catálogo : packwire available" -ForegroundColor Yellow
Write-Host "  • Gráfico  : packwire ui (o desde el icono en Menú Inicio / Escritorio)" -ForegroundColor Yellow
Write-Host ""
