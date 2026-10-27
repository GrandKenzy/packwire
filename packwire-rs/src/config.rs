//! Rutas y configuración persistente de Packwire.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

/// Plataforma del sistema operativo en la que se está ejecutando Packwire.
pub fn system() -> &'static str {
    env::consts::OS
}

/// Arquitectura de la máquina usando la nomenclatura estándar de Rust.
pub fn machine() -> &'static str {
    env::consts::ARCH
}

/// Todas las rutas de datos de una instalación de Packwire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub packwire_root: PathBuf,
    pub apps_dir: PathBuf,
    pub shims_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub manifests_dir: PathBuf,
    pub state_file: PathBuf,
    pub config_file: PathBuf,
}

impl Paths {
    pub fn from_root(packwire_root: PathBuf) -> Self {
        Self {
            apps_dir: packwire_root.join("apps"),
            shims_dir: packwire_root.join("shims"),
            cache_dir: packwire_root.join("cache"),
            manifests_dir: packwire_root.join("manifests"),
            state_file: packwire_root.join("installed.json"),
            config_file: packwire_root.join("configs.json"),
            packwire_root,
        }
    }
}

/// Configuración de usuario por defecto.
pub fn default_config() -> Value {
    let mut config = Map::new();
    config.insert("theme".into(), Value::String("default".into()));
    config.insert("check_updates_on_start".into(), Value::Bool(true));
    Value::Object(config)
}

/// Devuelve la raíz de datos por defecto según el sistema operativo.
pub fn default_root() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    match system() {
        "windows" => env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".packwire")),
        "macos" => home
            .join("Library")
            .join("Application Support")
            .join("packwire"),
        _ => env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local").join("share"))
            .join("packwire"),
    }
}

/// Calcula las rutas de Packwire, aplicando `PACKWIRE_ROOT` si está definido.
pub fn paths() -> Paths {
    let root = env::var_os("PACKWIRE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(default_root);
    Paths::from_root(root)
}

/// Crea directorios y archivos base necesarios para la ejecución.
pub fn ensure_directories(paths: &Paths) -> io::Result<()> {
    for directory in [
        &paths.packwire_root,
        &paths.apps_dir,
        &paths.shims_dir,
        &paths.cache_dir,
        &paths.manifests_dir,
    ] {
        fs::create_dir_all(directory)?;
    }

    if !paths.state_file.exists() {
        write_text(&paths.state_file, "{}")?;
    }

    if !paths.config_file.exists() {
        save_json(&paths.config_file, &default_config())?;
    }

    Ok(())
}

/// Carga `configs.json`, fusionándolo con los valores por defecto.
///
/// Los errores de lectura o JSON inválido devuelven la configuración por
/// defecto sin abortar la aplicación.
pub fn load_config(paths: &Paths) -> Value {
    if let Err(_) = ensure_directories(paths) {
        return default_config();
    }

    let content = match fs::read_to_string(&paths.config_file) {
        Ok(content) => content,
        Err(_) => return default_config(),
    };

    let loaded = match serde_json::from_str::<Value>(&content) {
        Ok(Value::Object(object)) => object,
        _ => return default_config(),
    };

    let mut merged = match default_config() {
        Value::Object(object) => object,
        _ => unreachable!("default_config siempre devuelve un objeto JSON"),
    };
    merged.extend(loaded);
    Value::Object(merged)
}

/// Guarda la configuración de usuario con formato legible.
pub fn save_config(paths: &Paths, config: &Value) -> io::Result<()> {
    ensure_base_directory(&paths.packwire_root)?;
    save_json(&paths.config_file, config)
}

/// Obtiene una clave de configuración, o `default` si no existe.
pub fn get_config(paths: &Paths, key: &str, default: Option<Value>) -> Option<Value> {
    load_config(paths).get(key).cloned().or(default)
}

/// Actualiza y persiste una clave de configuración.
pub fn set_config(paths: &Paths, key: &str, value: Value) -> io::Result<()> {
    let mut config = load_config(paths);
    let object = config.as_object_mut().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "la configuración no es un objeto JSON",
        )
    })?;
    object.insert(key.to_owned(), value);
    save_config(paths, &config)
}

fn ensure_base_directory(root: &Path) -> io::Result<()> {
    fs::create_dir_all(root)
}

fn write_text(path: &Path, content: &str) -> io::Result<()> {
    fs::write(path, content)
}

fn save_json(path: &Path, value: &Value) -> io::Result<()> {
    let content = serde_json::to_string_pretty(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    write_text(path, &content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn temporary_paths() -> Paths {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("el reloj del sistema debe ser válido")
            .as_nanos();
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        Paths::from_root(
            env::temp_dir().join(format!("packwire-config-test-{}-{sequence}", suffix)),
        )
    }

    #[test]
    fn creates_base_files_and_defaults() {
        let paths = temporary_paths();
        ensure_directories(&paths).expect("debe crear la estructura base");

        assert!(paths.apps_dir.is_dir());
        assert!(paths.shims_dir.is_dir());
        assert_eq!(fs::read_to_string(&paths.state_file).unwrap(), "{}");
        assert_eq!(load_config(&paths)["theme"], "default");

        fs::remove_dir_all(paths.packwire_root).expect("debe limpiar el directorio temporal");
    }

    #[test]
    fn merges_loaded_values_with_defaults() {
        let paths = temporary_paths();
        ensure_directories(&paths).unwrap();
        fs::write(&paths.config_file, r#"{"theme":"dark"}"#).unwrap();

        let config = load_config(&paths);
        assert_eq!(config["theme"], "dark");
        assert_eq!(config["check_updates_on_start"], true);

        fs::remove_dir_all(paths.packwire_root).unwrap();
    }
}
