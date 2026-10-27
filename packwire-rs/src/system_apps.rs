//! Inventario de aplicaciones instaladas fuera del registro de Packwire.
//!
//! En Windows se consultan las claves estándar de desinstalación de Win32 y
//! los paquetes AppX registrados para el usuario/equipo. Este inventario es
//! informativo: Packwire no ejecuta automáticamente los desinstaladores del
//! sistema porque sus argumentos y permisos varían entre proveedores.

use serde_json::{json, Value};

#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
};
#[cfg(windows)]
use winreg::RegKey;

#[cfg(windows)]
const UNINSTALL_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall";

/// Devuelve el inventario que debe mostrar la zona "Aplicaciones del sistema".
pub fn list_system_apps() -> Value {
    #[cfg(windows)]
    {
        let mut apps = registry_apps();
        apps.extend(appx_apps());
        apps.sort_by(|left, right| {
            left["name"]
                .as_str()
                .unwrap_or_default()
                .to_lowercase()
                .cmp(&right["name"].as_str().unwrap_or_default().to_lowercase())
        });
        return json!({
            "success": true,
            "supported": true,
            "source": "Windows Registry + AppX",
            "apps": apps
        });
    }

    #[cfg(not(windows))]
    {
        json!({
            "success": true,
            "supported": false,
            "source": "unsupported",
            "apps": [],
            "message": "El inventario de aplicaciones del sistema está disponible actualmente en Windows."
        })
    }
}

#[cfg(windows)]
fn registry_apps() -> Vec<Value> {
    let mut apps = Vec::new();
    let locations = [
        ("usuario", HKEY_CURRENT_USER, 0),
        ("equipo-64", HKEY_LOCAL_MACHINE, KEY_WOW64_64KEY),
        ("equipo-32", HKEY_LOCAL_MACHINE, KEY_WOW64_32KEY),
    ];

    for (scope, hive, view) in locations {
        let root = RegKey::predef(hive);
        let Ok(uninstall) = root.open_subkey_with_flags(UNINSTALL_PATH, KEY_READ | view) else {
            continue;
        };

        for key_name in uninstall.enum_keys().flatten() {
            let Ok(entry) = uninstall.open_subkey_with_flags(&key_name, KEY_READ) else {
                continue;
            };
            if entry
                .get_value::<u32, _>("SystemComponent")
                .unwrap_or_default()
                == 1
                || entry.get_value::<u32, _>("NoDisplay").unwrap_or_default() == 1
            {
                continue;
            }

            let name = entry
                .get_value::<String, _>("DisplayName")
                .unwrap_or_default();
            if name.trim().is_empty() {
                continue;
            }

            apps.push(json!({
                "id": format!("registry:{scope}:{key_name}"),
                "name": name,
                "version": entry.get_value::<String, _>("DisplayVersion").unwrap_or_default(),
                "publisher": entry.get_value::<String, _>("Publisher").unwrap_or_default(),
                "install_location": entry.get_value::<String, _>("InstallLocation").unwrap_or_default(),
                "source": "Windows Registry",
                "kind": "win32",
                "managed_by_packwire": false,
                "can_uninstall": false
            }));
        }
    }

    apps
}

#[cfg(windows)]
fn appx_apps() -> Vec<Value> {
    let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
$ProgressPreference = 'SilentlyContinue'
$items = @(Get-AppxPackage -AllUsers | Where-Object { $_.Name -and $_.Version } | Select-Object Name, Version, Publisher, InstallLocation, PackageFullName)
if ($items.Count -eq 0) {
  $items = @(Get-AppxPackage | Where-Object { $_.Name -and $_.Version } | Select-Object Name, Version, Publisher, InstallLocation, PackageFullName)
}
$items | ConvertTo-Json -Compress
"#;

    let mut command = Command::new("powershell.exe");
    command.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ]);
    hide_console_window(&mut command);
    let Ok(output) = command.output() else {
        return Vec::new();
    };
    if !output.status.success() || output.stdout.is_empty() {
        return Vec::new();
    }

    let Ok(value) = serde_json::from_slice::<Value>(&output.stdout) else {
        return Vec::new();
    };
    match value {
        Value::Array(items) => items.into_iter().filter_map(appx_value).collect(),
        Value::Object(_) => appx_value(value).into_iter().collect(),
        _ => Vec::new(),
    }
}

#[cfg(windows)]
fn appx_value(value: Value) -> Option<Value> {
    let name = value.get("Name")?.as_str()?.trim().to_owned();
    if name.is_empty() {
        return None;
    }
    Some(json!({
        "id": format!("appx:{}", value.get("PackageFullName").and_then(Value::as_str).unwrap_or(&name)),
        "name": name,
        "version": value.get("Version").and_then(Value::as_str).unwrap_or_default(),
        "publisher": value.get("Publisher").and_then(Value::as_str).unwrap_or_default(),
        "install_location": value.get("InstallLocation").and_then(Value::as_str).unwrap_or_default(),
        "source": "Windows AppX",
        "kind": "appx",
        "managed_by_packwire": false,
        "can_uninstall": false
    }))
}

#[cfg(windows)]
fn hide_console_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    command.creation_flags(CREATE_NO_WINDOW);
}
