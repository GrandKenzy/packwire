//! Puente JSON y servidor local para la interfaz web de Packwire.
//!
//! El frontend existente se conserva sin reescritura: esta capa mantiene sus
//! métodos REST bajo /api y delega las instalaciones a TaskQueue.

use std::fs;
use std::io::{self, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use semver::Version;
use serde_json::{json, Map, Value};

use crate::config::{load_config, paths, save_config, system, Paths};
use crate::manifest::{sync_remote_catalog, ManifestLoader, ManifestSyncReport};
use crate::models::PackageState;
use crate::pather::Pather;
use crate::state::{get_installed, load_state, unregister_installed};
use crate::system_apps::list_system_apps;
use crate::task_queue::TaskQueue;

pub type GuiResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const APP_RELEASE_URL: &str = "https://api.github.com/repos/GrandKenzy/packwire/releases/latest";
const APP_UPDATE_CACHE_KEY: &str = "app_update_cache";
const APP_UPDATE_CACHE_TTL: u64 = 24 * 60 * 60;

#[derive(Clone)]
pub struct GuiBridge {
    pub paths: Paths,
    loader: ManifestLoader,
    task_queue: Arc<TaskQueue>,
    catalog_sync: ManifestSyncReport,
}

impl std::fmt::Debug for GuiBridge {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GuiBridge")
            .field("paths", &self.paths)
            .field("job_count", &self.task_queue.job_count())
            .finish_non_exhaustive()
    }
}

impl GuiBridge {
    pub fn new(paths: Paths) -> GuiResult<Self> {
        Self::new_with_catalog_sync(paths, ManifestSyncReport::cached())
    }

    fn new_with_catalog_sync(paths: Paths, catalog_sync: ManifestSyncReport) -> GuiResult<Self> {
        let loader = ManifestLoader::new(paths.manifests_dir.clone());
        let task_queue = Arc::new(TaskQueue::with_packwire(paths.clone(), 1)?);
        Ok(Self {
            paths,
            loader,
            task_queue,
            catalog_sync,
        })
    }

    /// Despacha una llamada del frontend y siempre devuelve JSON serializable.
    pub fn dispatch(&self, method: &str, args: &[Value]) -> Value {
        match self.dispatch_result(method, args) {
            Ok(value) => value,
            Err(error) => json!({"success": false, "message": error.to_string()}),
        }
    }

    fn dispatch_result(&self, method: &str, args: &[Value]) -> GuiResult<Value> {
        match method {
            "get_system_info" => Ok(self.get_system_info()),
            "list_available" => Ok(self.list_available()),
            "list_installed" => Ok(self.list_installed()),
            "list_system_apps" => Ok(list_system_apps()),
            "get_manifest" => {
                let package = required_string(args, 0, "package")?;
                Ok(self
                    .loader
                    .find_manifest(&package)
                    .map(|manifest| manifest.to_value())
                    .unwrap_or(Value::Null))
            }
            "queue_install" => {
                let package = required_string(args, 0, "package")?;
                let mode = optional_string(args, 1);
                let version = optional_string(args, 2);
                let force = optional_bool(args, 3).unwrap_or(false);
                self.queue_install(&package, mode.as_deref(), version.as_deref(), force)
            }
            "install" => {
                let package = required_string(args, 0, "package")?;
                let mode = optional_string(args, 1);
                let version = optional_string(args, 2);
                self.queue_install(&package, mode.as_deref(), version.as_deref(), false)
            }
            "get_job" => {
                let job_id = required_string(args, 0, "job_id")?;
                Ok(self
                    .task_queue
                    .get_job_snapshot(&job_id)
                    .map(|job| serde_json::to_value(job).expect("snapshot serializable"))
                    .unwrap_or(Value::Null))
            }
            "list_jobs" => Ok(json!(self.task_queue.list_job_snapshots())),
            "get_config" => {
                let key = optional_string(args, 0);
                let config = load_config(&self.paths);
                Ok(key
                    .and_then(|key| config.get(&key).cloned())
                    .unwrap_or(config))
            }
            "set_config" => {
                let key = required_string(args, 0, "key")?;
                let value = args.get(1).cloned().unwrap_or(Value::Null);
                save_config_value(&self.paths, &key, value)
            }
            "add_path" => self.add_path(),
            "add_defender_exclusion" => self.add_defender_exclusion(),
            "open_site" => {
                let url = required_string(args, 0, "url")?;
                open_url(&url)?;
                Ok(json!({"success": true}))
            }
            "open_guide" => {
                let guide = required_string(args, 0, "guide")?;
                self.open_guide(&guide)
            }
            "list_themes" => Ok(json!(self.list_themes())),
            "check_updates" => Ok(json!({})),
            "get_catalog_sync" => Ok(serde_json::to_value(&self.catalog_sync)?),
            "check_app_update" => self.check_app_update(),
            "uninstall" => {
                let package = required_string(args, 0, "package")?;
                self.uninstall(&package)
            }
            "uninstall_all" => self.uninstall_all(optional_bool(args, 0).unwrap_or(true)),
            "reinstall" | "update" => Ok(json!({
                "success": false,
                "message": format!("La operación '{method}' todavía no está migrada a Rust.")
            })),
            _ => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Método GUI no encontrado: {method}"),
            )
            .into()),
        }
    }

    fn get_system_info(&self) -> Value {
        let hardware = collect_windows_diagnostics();
        let is_windows_11 = hardware
            .get("os")
            .and_then(|value| value.get("build"))
            .and_then(value_as_u64)
            .is_some_and(|build| build >= 22_000);
        json!({
            "os": system(),
            "arch": crate::config::machine(),
            "shims_in_path": Pather::new(self.paths.shims_dir.clone())
                .map(|pather| pather.is_shims_in_path())
                .unwrap_or(false),
            "root_dir": self.paths.packwire_root,
            "shims_dir": self.paths.shims_dir,
            "apps_dir": self.paths.apps_dir,
            "version": env!("CARGO_PKG_VERSION"),
            "is_windows_11": is_windows_11,
            "is_windows": cfg!(windows),
            "hardware": hardware
        })
    }

    fn check_app_update(&self) -> GuiResult<Value> {
        let current_version = env!("CARGO_PKG_VERSION");
        let config = load_config(&self.paths);
        if config
            .get("check_updates_on_start")
            .and_then(Value::as_bool)
            == Some(false)
        {
            return Ok(json!({
                "success": true,
                "update_available": false,
                "disabled": true,
                "current_version": current_version
            }));
        }

        let now = unix_timestamp();
        if let Some(cache) = config.get(APP_UPDATE_CACHE_KEY).and_then(Value::as_object) {
            let cache_version = cache
                .get("current_version")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let checked_at = cache
                .get("checked_at")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            if cache_version == current_version
                && checked_at <= now
                && now.saturating_sub(checked_at) < APP_UPDATE_CACHE_TTL
            {
                let mut cached = Value::Object(cache.clone());
                cached["from_cache"] = Value::Bool(true);
                return Ok(cached);
            }
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .user_agent(format!("Packwire/{current_version} (Rust)"))
            .build()?;
        let response = client.get(APP_RELEASE_URL).send()?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(json!({
                "success": true,
                "update_available": false,
                "current_version": current_version,
                "message": "Todavía no hay una release pública de Packwire."
            }));
        }
        let release = response.error_for_status()?.json::<Value>()?;
        let tag = release
            .get("tag_name")
            .and_then(Value::as_str)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "release sin tag_name"))?;
        let latest_version = tag.trim_start_matches('v');
        let current = Version::parse(current_version)?;
        let latest = Version::parse(latest_version)?;
        let release_url = release
            .get("html_url")
            .and_then(Value::as_str)
            .unwrap_or("https://github.com/GrandKenzy/packwire/releases/latest");
        let release_name = release.get("name").and_then(Value::as_str).unwrap_or(tag);
        let result = json!({
            "success": true,
            "update_available": latest > current,
            "current_version": current_version,
            "latest_version": latest_version,
            "release_name": release_name,
            "release_url": release_url,
            "checked_at": now,
            "from_cache": false
        });
        save_config_value(&self.paths, APP_UPDATE_CACHE_KEY, result.clone())?;
        Ok(result)
    }

    fn list_available(&self) -> Value {
        let manifests = self
            .loader
            .list_available_manifests()
            .into_iter()
            .map(|manifest| manifest.to_value())
            .collect::<Vec<_>>();
        Value::Array(manifests)
    }

    fn list_installed(&self) -> Value {
        let state = load_state(&self.paths);
        let object: Map<String, Value> = state
            .into_iter()
            .map(|(id, package)| (id, package.to_value()))
            .collect();
        Value::Object(object)
    }

    fn uninstall(&self, package_id: &str) -> GuiResult<Value> {
        let Some(installed) = get_installed(&self.paths, package_id) else {
            return Ok(json!({
                "success": false,
                "message": format!("El paquete '{package_id}' no está registrado como instalado.")
            }));
        };

        if installed.mode == "site" {
            return Ok(json!({
                "success": false,
                "message": format!("'{}' fue instalado manualmente desde su sitio oficial y no puede desinstalarse desde Packwire.", installed.name)
            }));
        }

        if installed.mode == "command" {
            if let Err(error) = self.uninstall_external(&installed) {
                return Ok(json!({
                    "success": false,
                    "message": error.to_string()
                }));
            }
        }

        let pather = Pather::new(self.paths.shims_dir.clone())?;
        pather.remove_shims(&installed.shims)?;

        if installed.mode == "here" {
            if let Some(install_path) = installed.install_path.as_deref() {
                let target = PathBuf::from(install_path);
                if target.exists() {
                    if !is_safe_packwire_install_path(&self.paths.apps_dir, &target) {
                        return Ok(json!({
                            "success": false,
                            "message": format!("Se rechazó eliminar una ruta fuera de la carpeta de aplicaciones de Packwire: {}", target.display())
                        }));
                    }
                    fs::remove_dir_all(&target)?;
                }
            }
        }

        unregister_installed(&self.paths, package_id)?;
        Ok(json!({
            "success": true,
            "message": format!("Paquete '{}' (versión {}) desinstalado correctamente.", installed.name, installed.version)
        }))
    }

    fn uninstall_external(&self, installed: &PackageState) -> GuiResult<()> {
        let Some(manifest) = self.loader.find_manifest(&installed.id) else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "No se encontró el manifest de '{}'; no se modificó la instalación externa.",
                    installed.name
                ),
            )
            .into());
        };
        let source = manifest
            .upstream
            .as_ref()
            .and_then(|value| value.get("source"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let Some(winget_id) = manifest
            .upstream
            .as_ref()
            .and_then(|value| value.get("id"))
            .and_then(Value::as_str)
            .filter(|_| source == "winget")
        else {
            return Err(io::Error::other(format!(
                "'{}' se instaló mediante un comando externo sin desinstalador administrable por Packwire.",
                installed.name
            ))
            .into());
        };

        #[cfg(not(windows))]
        {
            let _ = winget_id;
            return Err(io::Error::other(
                "La desinstalación mediante WinGet solo está disponible en Windows.",
            )
            .into());
        }

        #[cfg(windows)]
        {
            let mut command = Command::new("winget.exe");
            command.args([
                "uninstall",
                "--id",
                winget_id,
                "-e",
                "--source",
                "winget",
                "--silent",
                "--accept-source-agreements",
            ]);
            hide_console_window(&mut command);
            let output = command.output()?;
            if !output.status.success() {
                let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
                return Err(io::Error::other(if detail.is_empty() {
                    format!("WinGet no pudo desinstalar '{}'.", installed.name)
                } else {
                    format!("WinGet no pudo desinstalar '{}': {detail}", installed.name)
                })
                .into());
            }
        }

        Ok(())
    }

    fn uninstall_all(&self, remove_path: bool) -> GuiResult<Value> {
        let package_ids = load_state(&self.paths).into_keys().collect::<Vec<_>>();
        let mut removed = 0usize;
        let mut warnings = Vec::new();

        for package_id in package_ids {
            match self.uninstall(&package_id) {
                Ok(result) if result["success"] == true => removed += 1,
                Ok(result) => warnings.push(
                    result["message"]
                        .as_str()
                        .unwrap_or("No se pudo desinstalar un paquete.")
                        .to_owned(),
                ),
                Err(error) => warnings.push(format!("{package_id}: {error}")),
            }
        }

        if self.paths.shims_dir.is_dir() {
            for entry in fs::read_dir(&self.paths.shims_dir)?.flatten() {
                if entry.path().is_file() {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
        if self.paths.cache_dir.is_dir() {
            fs::remove_dir_all(&self.paths.cache_dir)?;
            fs::create_dir_all(&self.paths.cache_dir)?;
        }
        if remove_path {
            let pather = Pather::new(self.paths.shims_dir.clone())?;
            let _ = pather.remove_shims_from_user_path();
        }

        let message = if warnings.is_empty() {
            format!(
                "Desinstalación completa de Packwire realizada. Paquetes eliminados: {removed}."
            )
        } else {
            format!(
                "Se eliminaron {removed} paquetes, pero quedaron avisos: {}",
                warnings.join(" | ")
            )
        };
        Ok(json!({
            "success": warnings.is_empty(),
            "message": message
        }))
    }

    fn queue_install(
        &self,
        package: &str,
        mode: Option<&str>,
        version: Option<&str>,
        force: bool,
    ) -> GuiResult<Value> {
        let manifest = self.loader.find_manifest(package).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("No se encontró ningún manifest para '{package}'."),
            )
        })?;
        let mode = mode
            .filter(|mode| !mode.is_empty())
            .unwrap_or(manifest.install.mode.as_str());
        let job_id = self.task_queue.submit_install(
            package,
            Some(manifest.name),
            mode,
            version.map(ToOwned::to_owned),
            force,
        )?;
        Ok(json!({"success": true, "job_id": job_id}))
    }

    fn add_path(&self) -> GuiResult<Value> {
        let pather = Pather::new(self.paths.shims_dir.clone())?;
        let success = pather.add_shims_to_user_path()?;
        Ok(json!({
            "success": success,
            "message": if success {
                "Directorio de shims añadido al PATH."
            } else {
                "No se pudo actualizar el PATH."
            }
        }))
    }

    fn add_defender_exclusion(&self) -> GuiResult<Value> {
        let pather = Pather::new(self.paths.shims_dir.clone())?;
        let (success, message) = pather.add_defender_exclusion(None)?;
        Ok(json!({"success": success, "message": message}))
    }

    fn open_guide(&self, guide_name: &str) -> GuiResult<Value> {
        if guide_name.contains("..")
            || guide_name.contains('/')
            || guide_name.contains('\\')
            || guide_name.is_empty()
        {
            return Ok(json!({"success": false, "message": "Nombre de guía no válido."}));
        }
        let guide_path = web_dir().join("guides").join(guide_name);
        if !guide_path.is_file() {
            return Ok(json!({
                "success": false,
                "message": format!("Guía no encontrada: {guide_name}")
            }));
        }
        open_url(&file_url(&guide_path))?;
        Ok(json!({"success": true, "path": guide_path}))
    }

    fn list_themes(&self) -> Vec<String> {
        let mut themes = Vec::new();
        if let Ok(entries) = fs::read_dir(web_dir().join("global")) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let Some(stem) = entry_path.file_stem().and_then(|value| value.to_str()) else {
                    continue;
                };
                if let Some((theme, _)) = stem.split_once('_') {
                    themes.push(theme.to_owned());
                }
            }
        }
        themes.sort();
        themes.dedup();
        if let Some(index) = themes.iter().position(|theme| theme == "default") {
            let default = themes.remove(index);
            themes.insert(0, default);
        }
        if themes.is_empty() {
            vec!["default".into(), "dark".into()]
        } else {
            themes
        }
    }
}

fn required_string(args: &[Value], index: usize, name: &str) -> GuiResult<String> {
    optional_string(args, index).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("falta el argumento '{name}'"),
        )
        .into()
    })
}

fn optional_string(args: &[Value], index: usize) -> Option<String> {
    args.get(index).and_then(|value| match value {
        Value::String(value) if !value.is_empty() => Some(value.clone()),
        _ => None,
    })
}

fn optional_bool(args: &[Value], index: usize) -> Option<bool> {
    args.get(index).and_then(Value::as_bool)
}

fn is_safe_packwire_install_path(apps_dir: &Path, target: &Path) -> bool {
    if !target.is_absolute() {
        return false;
    }

    let apps_root = fs::canonicalize(apps_dir).unwrap_or_else(|_| apps_dir.to_path_buf());
    let target_path = fs::canonicalize(target).unwrap_or_else(|_| target.to_path_buf());
    target_path != apps_root && target_path.starts_with(&apps_root)
}

fn hide_console_window(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    #[cfg(not(windows))]
    let _ = command;
}

fn value_as_u64(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|text| text.trim().parse().ok()))
}

fn collect_windows_diagnostics() -> Value {
    #[cfg(not(windows))]
    {
        return json!({});
    }

    #[cfg(windows)]
    {
        let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
$ProgressPreference = 'SilentlyContinue'
$os = Get-CimInstance Win32_OperatingSystem
$computer = Get-CimInstance Win32_ComputerSystem
$cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
$bios = Get-CimInstance Win32_BIOS | Select-Object -First 1
$displayVersion = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion').DisplayVersion
$secureBoot = $null
try { $secureBoot = [bool](Confirm-SecureBootUEFI) } catch { }
$tpm = $null
try { $tpm = Get-Tpm } catch { }
$uptime = $null
if ($os.LastBootUpTime) { $uptime = (Get-Date) - $os.LastBootUpTime }

$gpus = @(Get-CimInstance Win32_VideoController | Where-Object { $_.Name } | ForEach-Object {
    [pscustomobject]@{
        name = $_.Name
        driver = $_.DriverVersion
        memory_gb = if ($_.AdapterRAM) { [math]::Round(([double]$_.AdapterRAM / 1GB), 1) } else { $null }
    }
})
$volumes = @(Get-CimInstance Win32_LogicalDisk -Filter "DriveType=3" | Where-Object { $_.Size } | ForEach-Object {
    [pscustomobject]@{
        drive = $_.DeviceID
        filesystem = $_.FileSystem
        size_gb = [math]::Round(([double]$_.Size / 1GB), 1)
        free_gb = [math]::Round(([double]$_.FreeSpace / 1GB), 1)
    }
})
$network = @(Get-NetAdapter -Physical | Where-Object { $_.Status -eq 'Up' } | ForEach-Object {
    [pscustomobject]@{
        name = $_.Name
        status = $_.Status
        link_speed = $_.LinkSpeed
    }
})

[pscustomobject]@{
    os = [pscustomobject]@{
        caption = $os.Caption
        display_version = $displayVersion
        version = $os.Version
        build = $os.BuildNumber
        architecture = $os.OSArchitecture
    }
    computer = [pscustomobject]@{
        manufacturer = $computer.Manufacturer
        model = $computer.Model
        memory_gb = if ($computer.TotalPhysicalMemory) { [math]::Round(([double]$computer.TotalPhysicalMemory / 1GB), 1) } else { $null }
        memory_free_gb = if ($os.FreePhysicalMemory) { [math]::Round(([double]$os.FreePhysicalMemory / 1MB), 1) } else { $null }
        hypervisor_present = [bool]$computer.HypervisorPresent
    }
    cpu = [pscustomobject]@{
        name = $cpu.Name
        cores = $cpu.NumberOfCores
        logical_processors = $cpu.NumberOfLogicalProcessors
        max_clock_mhz = $cpu.MaxClockSpeed
        virtualization_firmware = $cpu.VirtualizationFirmwareEnabled
    }
    firmware = [pscustomobject]@{
        bios_vendor = $bios.Manufacturer
        bios_version = $bios.SMBIOSBIOSVersion
        bios_date = if ($bios.ReleaseDate) { $bios.ReleaseDate.ToString('yyyy-MM-dd') } else { $null }
        mode = if ($secureBoot -ne $null) { 'UEFI' } else { 'No confirmado' }
    }
    security = [pscustomobject]@{
        secure_boot = $secureBoot
        tpm_present = if ($tpm) { [bool]$tpm.TpmPresent } else { $null }
        tpm_ready = if ($tpm) { [bool]$tpm.TpmReady } else { $null }
        tpm_spec = if ($tpm) { $tpm.SpecVersion } else { $null }
    }
    uptime_hours = if ($uptime) { [math]::Round($uptime.TotalHours, 1) } else { $null }
    gpus = $gpus
    volumes = $volumes
    network = $network
} | ConvertTo-Json -Depth 6 -Compress
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
            return json!({});
        };
        if !output.status.success() || output.stdout.is_empty() {
            return json!({});
        }

        let Ok(mut data) = serde_json::from_slice::<Value>(&output.stdout) else {
            return json!({});
        };
        if let Some(object) = data.as_object_mut() {
            for key in ["gpus", "volumes", "network"] {
                let normalized = match object.remove(key) {
                    Some(Value::Array(items)) => items,
                    Some(Value::Object(item)) => vec![Value::Object(item)],
                    _ => Vec::new(),
                };
                object.insert(key.to_owned(), Value::Array(normalized));
            }
        }
        data
    }
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn save_config_value(paths: &Paths, key: &str, value: Value) -> GuiResult<Value> {
    let mut config = load_config(paths);
    let object = config.as_object_mut().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "la configuración no es un objeto",
        )
    })?;
    object.insert(key.to_owned(), value.clone());
    save_config(paths, &config)?;
    Ok(json!({"success": true, "key": key, "value": value}))
}

/// Servidor HTTP local que sirve el frontend existente y sus llamadas REST.
pub struct LocalServer {
    pub url: String,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for LocalServer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LocalServer")
            .field("url", &self.url)
            .finish_non_exhaustive()
    }
}

impl LocalServer {
    pub fn start(port: u16, bridge: GuiBridge) -> GuiResult<Self> {
        let listener = TcpListener::bind(("127.0.0.1", port))?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        let url = format!("http://127.0.0.1:{}", address.port());
        let stop = Arc::new(AtomicBool::new(false));
        let stop_for_thread = Arc::clone(&stop);
        let web_root = web_dir();
        let connections = Arc::new(Mutex::new(Vec::<JoinHandle<()>>::new()));
        let connections_for_thread = Arc::clone(&connections);

        let handle = thread::Builder::new()
            .name("PackwireHttpServer".into())
            .spawn(move || {
                while !stop_for_thread.load(Ordering::Acquire) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let bridge = bridge.clone();
                            let web_root = web_root.clone();
                            if let Ok(handle) = thread::Builder::new()
                                .name("PackwireHttpRequest".into())
                                .spawn(move || handle_connection(stream, bridge, web_root))
                            {
                                if let Ok(mut active) = connections_for_thread.lock() {
                                    active.push(handle);
                                }
                            }
                        }
                        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(20));
                        }
                        Err(_) => break,
                    }
                }
                if let Ok(mut active) = connections_for_thread.lock() {
                    for connection in active.drain(..) {
                        let _ = connection.join();
                    }
                }
            })?;

        Ok(Self {
            url,
            stop,
            handle: Some(handle),
        })
    }

    pub fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for LocalServer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Inicia la interfaz de Packwire.
///
/// En Windows, el modo normal abre el frontend dentro de una ventana nativa
/// Wry/TAO. `web_mode` conserva el fallback compatible con la interfaz web
/// anterior para desarrollo y diagnóstico.
pub fn launch_gui(web_mode: bool, port: u16) -> GuiResult<()> {
    let paths = paths();
    let catalog_sync = sync_remote_catalog(&paths.manifests_dir);
    let bridge = GuiBridge::new_with_catalog_sync(paths, catalog_sync)?;
    let server = LocalServer::start(port, bridge)?;
    if web_mode {
        return launch_browser_gui(server);
    }

    #[cfg(windows)]
    {
        return launch_native_gui(server.url.clone(), server);
    }

    #[cfg(not(windows))]
    launch_browser_gui(server)
}

fn launch_browser_gui(server: LocalServer) -> GuiResult<()> {
    println!("[*] Servidor local activo en: {}", server.url);
    println!("[*] Abriendo el frontend en el navegador predeterminado...");
    open_url(&server.url)?;
    println!("[*] Presiona Ctrl+C en esta terminal para detener el servidor.");

    let (sender, receiver) = mpsc::channel();
    let _ = ctrlc::set_handler(move || {
        let _ = sender.send(());
    });
    let _ = receiver.recv();
    drop(server);
    Ok(())
}

#[cfg(windows)]
fn launch_native_gui(url: String, _server: LocalServer) -> GuiResult<()> {
    use tao::dpi::LogicalSize;
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoop};
    use tao::window::WindowBuilder;
    use wry::WebViewBuilder;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Packwire")
        .with_inner_size(LogicalSize::new(1340.0, 840.0))
        .with_min_inner_size(LogicalSize::new(960.0, 640.0))
        .build(&event_loop)?;
    let _webview = WebViewBuilder::new().with_url(&url).build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}

fn handle_connection(mut stream: TcpStream, bridge: GuiBridge, web_root: PathBuf) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let Ok(request) = read_request(&mut stream) else {
        return;
    };
    let (status, content_type, body) = if request.path.starts_with("/api/") {
        let method = request.path["/api/".len()..]
            .split('?')
            .next()
            .unwrap_or_default();
        let args = if request.method == "POST" {
            serde_json::from_slice::<Value>(&request.body)
                .ok()
                .and_then(|payload| payload.get("args").cloned())
                .and_then(|args| args.as_array().cloned())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        (
            "200 OK",
            "application/json; charset=utf-8",
            serde_json::to_vec(&bridge.dispatch(method, &args)).unwrap_or_else(|_| {
                b"{\"success\":false,\"message\":\"respuesta invalida\"}".to_vec()
            }),
        )
    } else if request.method != "GET" {
        (
            "405 Method Not Allowed",
            "text/plain; charset=utf-8",
            b"Method Not Allowed".to_vec(),
        )
    } else {
        match static_file(&web_root, &request.path) {
            Ok((mime, body)) => ("200 OK", mime, body),
            Err(_) => (
                "404 Not Found",
                "text/plain; charset=utf-8",
                b"File Not Found".to_vec(),
            ),
        }
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.write_all(&body);
}

struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn read_request(stream: &mut TcpStream) -> io::Result<HttpRequest> {
    use std::io::Read;

    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 8192];
    let header_end;
    loop {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "petición HTTP incompleta",
            ));
        }
        bytes.extend_from_slice(&chunk[..read]);
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            header_end = index + 4;
            break;
        }
        if bytes.len() > 64 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "cabecera HTTP demasiado grande",
            ));
        }
    }

    let headers = std::str::from_utf8(&bytes[..header_end]).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("cabecera HTTP inválida: {error}"),
        )
    })?;
    let mut lines = headers.split("\r\n");
    let request_line = lines
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "línea HTTP ausente"))?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or_default().to_owned();
    let path = request_parts.next().unwrap_or("/").to_owned();
    let content_length = lines
        .find_map(|line| {
            line.strip_prefix("Content-Length:")
                .or_else(|| line.strip_prefix("content-length:"))
                .and_then(|value| value.trim().parse::<usize>().ok())
        })
        .unwrap_or(0);

    if content_length > 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cuerpo HTTP demasiado grande",
        ));
    }
    while bytes.len() < header_end + content_length {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "cuerpo HTTP incompleto",
            ));
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    Ok(HttpRequest {
        method,
        path,
        body: bytes[header_end..header_end + content_length].to_vec(),
    })
}

fn static_file(web_root: &Path, request_path: &str) -> io::Result<(&'static str, Vec<u8>)> {
    let relative = request_path
        .split('?')
        .next()
        .unwrap_or("/")
        .trim_start_matches('/');
    if relative.is_empty() {
        return static_file_path(web_root, Path::new("index.html"));
    }
    if relative.contains("..")
        || relative.contains('\\')
        || relative.to_ascii_lowercase().contains("%2e")
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "ruta estática no permitida",
        ));
    }
    static_file_path(web_root, Path::new(relative))
}

fn static_file_path(web_root: &Path, relative: &Path) -> io::Result<(&'static str, Vec<u8>)> {
    let path = web_root.join(relative);
    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "archivo estático no encontrado",
        ));
    }
    let mime = match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    };
    Ok((mime, fs::read(path)?))
}

fn web_dir() -> PathBuf {
    std::env::var_os("PACKWIRE_WEB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if let Ok(executable) = std::env::current_exe() {
                if let Some(parent) = executable.parent() {
                    let installed = parent.join("web");
                    if installed.is_dir() {
                        return installed;
                    }
                }
            }
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("assets")
                .join("web")
        })
}

fn file_url(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    format!(
        "file:///{}",
        path.replace(' ', "%20")
            .replace('#', "%23")
            .replace('?', "%3F")
    )
}

fn open_url(url: &str) -> GuiResult<()> {
    let status = if cfg!(windows) {
        Command::new("cmd.exe")
            .args(["/c", "start", "", url])
            .status()?
    } else if system() == "macos" {
        Command::new("open").arg(url).status()?
    } else {
        Command::new("xdg-open").arg(url).status()?
    };
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("no se pudo abrir la URL: {url}")).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_paths() -> Paths {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Paths::from_root(std::env::temp_dir().join(format!("packwire-gui-test-{suffix}")))
    }

    #[test]
    fn bridge_exposes_catalog_jobs_and_config_as_json() {
        let paths = test_paths();
        let bridge = GuiBridge::new(paths.clone()).unwrap();

        let available = bridge.dispatch("list_available", &[]);
        assert!(available.as_array().unwrap().len() >= 44);
        assert_eq!(bridge.dispatch("list_installed", &[]), json!({}));

        let info = bridge.dispatch("get_system_info", &[]);
        assert_eq!(
            info["root_dir"],
            paths.packwire_root.to_string_lossy().as_ref()
        );

        let saved = bridge.dispatch(
            "set_config",
            &[Value::String("theme".into()), Value::String("dark".into())],
        );
        assert_eq!(saved["success"], true);
        assert_eq!(
            bridge.dispatch("get_config", &[Value::String("theme".into())]),
            "dark"
        );

        let disabled = bridge.dispatch(
            "set_config",
            &[
                Value::String("check_updates_on_start".into()),
                Value::Bool(false),
            ],
        );
        assert_eq!(disabled["success"], true);
        let app_update = bridge.dispatch("check_app_update", &[]);
        assert_eq!(app_update["success"], true);
        assert_eq!(app_update["disabled"], true);

        let unsupported = bridge.dispatch("uninstall", &[Value::String("python".into())]);
        assert_eq!(unsupported["success"], false);
        fs::remove_dir_all(paths.packwire_root).unwrap();
    }

    #[test]
    fn static_files_reject_traversal_and_serve_frontend_assets() {
        let root = web_dir();
        let (_, index) = static_file(&root, "/").unwrap();
        assert!(String::from_utf8_lossy(&index).contains("Packwire"));
        assert!(static_file(&root, "/../Cargo.toml").is_err());
        assert!(static_file(&root, "/%2e%2e/Cargo.toml").is_err());
    }

    #[test]
    fn local_server_serves_json_api() {
        let paths = test_paths();
        let bridge = GuiBridge::new(paths.clone()).unwrap();
        let server = LocalServer::start(0, bridge).unwrap();
        let address = server.url.trim_start_matches("http://");
        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .write_all(b"GET /api/get_system_info HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("\"root_dir\""));
        fs::remove_dir_all(paths.packwire_root).unwrap();
    }
}
