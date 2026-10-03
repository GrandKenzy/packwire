@echo off
chcp 65001 >nul
setlocal

title Packwire Windows Installer

echo.
echo ==========================================================
echo              PACKWIRE WINDOWS INSTALLER                  
echo ==========================================================
echo.

where python >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [-] Error: Python no fue encontrado en el sistema.
    echo [!] Por favor instala Python 3.9 o superior y marca "Add python.exe to PATH".
    echo.
    goto :finish
)

echo [*] Ejecutando instalador en PowerShell...
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1" -DesktopShortcut %*
set SCRIPT_EXIT_CODE=%ERRORLEVEL%

if %SCRIPT_EXIT_CODE% neq 0 (
    echo.
    echo [-] La instalación finalizó con advertencias o errores (Código %SCRIPT_EXIT_CODE%).
)

:finish
echo.
REM Si fue abierto mediante doble clic en el explorador, pausar para permitir lectura
echo %cmdcmdline% | find /i "%~0" >nul
if %ERRORLEVEL% equ 0 (
    echo Presiona cualquier tecla para salir...
    pause >nul
)

exit /b %SCRIPT_EXIT_CODE%
