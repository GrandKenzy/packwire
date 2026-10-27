//! Persistencia del estado instalado y detección de colisiones.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

use crate::config::{ensure_directories, Paths};
use crate::models::PackageState;

static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Resultado de comprobar duplicados y colisiones de binarios.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionCheck {
    pub is_duplicate: bool,
    pub duplicate_message: Option<String>,
    pub collisions: Vec<String>,
}

/// Carga el registro de paquetes instalados.
///
/// Un archivo inexistente, inválido o incompatible produce un estado vacío
/// para que Packwire pueda recuperarse sin bloquear el arranque.
pub fn load_state(paths: &Paths) -> HashMap<String, PackageState> {
    if ensure_directories(paths).is_err() || !paths.state_file.exists() {
        return HashMap::new();
    }

    let content = match fs::read_to_string(&paths.state_file) {
        Ok(content) => content,
        Err(_) => return HashMap::new(),
    };
    let raw: Map<String, Value> = match serde_json::from_str(&content) {
        Ok(raw) => raw,
        Err(_) => return HashMap::new(),
    };

    raw.into_iter()
        .filter_map(|(pkg_id, value)| {
            PackageState::from_value(&value)
                .ok()
                .map(|package| (pkg_id, package))
        })
        .collect()
}

/// Guarda el registro usando un archivo temporal y reemplazo controlado.
pub fn save_state(paths: &Paths, state: &HashMap<String, PackageState>) -> io::Result<()> {
    ensure_directories(paths)?;

    let data: Map<String, Value> = state
        .iter()
        .map(|(pkg_id, package)| (pkg_id.clone(), package.to_value()))
        .collect();
    let content = serde_json::to_vec_pretty(&data)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    atomic_write(&paths.state_file, &content)
}

/// Comprueba si el paquete ya está registrado o si sus binarios chocan con
/// shims/binarios de paquetes existentes.
pub fn check_collision(paths: &Paths, pkg_id: &str, binaries: &[String]) -> CollisionCheck {
    let state = load_state(paths);

    if let Some(existing) = state.get(pkg_id) {
        return CollisionCheck {
            is_duplicate: true,
            duplicate_message: Some(format!(
                "El paquete '{pkg_id}' ya está instalado (versión {}).",
                existing.version
            )),
            collisions: Vec::new(),
        };
    }

    let mut collisions = Vec::new();
    for (existing_id, existing_package) in &state {
        let existing_shims: Vec<String> = existing_package
            .shims
            .iter()
            .map(|shim| file_stem(shim).to_lowercase())
            .collect();

        for binary in binaries {
            let base_binary = file_stem(binary).to_lowercase();
            if existing_shims.iter().any(|shim| shim == &base_binary)
                || existing_package.binaries.iter().any(|item| item == binary)
            {
                collisions.push(format!("{binary} (ya provisto por '{existing_id}')"));
            }
        }
    }

    CollisionCheck {
        is_duplicate: false,
        duplicate_message: None,
        collisions,
    }
}

/// Registra o actualiza un paquete instalado.
pub fn register_installed(paths: &Paths, mut package: PackageState) -> io::Result<()> {
    if package.installed_at.is_empty() {
        package.installed_at = installation_timestamp();
    }

    let mut state = load_state(paths);
    state.insert(package.id.clone(), package);
    save_state(paths, &state)
}

/// Elimina un paquete del registro y devuelve su estado anterior.
pub fn unregister_installed(paths: &Paths, pkg_id: &str) -> io::Result<Option<PackageState>> {
    let mut state = load_state(paths);
    let removed = state.remove(pkg_id);
    if removed.is_some() {
        save_state(paths, &state)?;
    }
    Ok(removed)
}

/// Obtiene el estado de un paquete instalado.
pub fn get_installed(paths: &Paths, pkg_id: &str) -> Option<PackageState> {
    load_state(paths).remove(pkg_id)
}

fn file_stem(value: &str) -> &str {
    std::path::Path::new(value)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(value)
}

fn installation_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}

fn atomic_write(path: &PathBuf, content: &[u8]) -> io::Result<()> {
    let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("state.json");
    let temp_path = path.with_file_name(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        sequence
    ));

    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;
        file.write_all(content)?;
        file.flush()?;
        file.sync_all()?;
        replace_file(&temp_path, path)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn replace_file(temp_path: &PathBuf, destination: &PathBuf) -> io::Result<()> {
    match fs::rename(temp_path, destination) {
        Ok(()) => Ok(()),
        Err(rename_error) if cfg!(target_os = "windows") && destination.exists() => {
            // Windows does not replace an existing file with rename(). Remove
            // only the known state target, then complete the replacement.
            fs::remove_file(destination).map_err(|_| rename_error)?;
            fs::rename(temp_path, destination)
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Paths;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_paths() -> Paths {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Paths::from_root(env::temp_dir().join(format!("packwire-state-test-{suffix}")))
    }

    fn package(id: &str, binaries: &[&str], shims: &[&str]) -> PackageState {
        PackageState {
            id: id.into(),
            name: id.into(),
            version: "1.0.0".into(),
            package_type: "fixed".into(),
            mode: "here".into(),
            install_path: Some("C:/packwire/apps/test".into()),
            binaries: binaries.iter().map(|value| (*value).into()).collect(),
            shims: shims.iter().map(|value| (*value).into()).collect(),
            installed_at: String::new(),
            manifest_hash: "hash".into(),
        }
    }

    #[test]
    fn persists_and_loads_state() {
        let paths = temporary_paths();
        let stored = package("tool@1", &["tool.exe"], &["tool.cmd"]);
        register_installed(&paths, stored.clone()).unwrap();

        let loaded = load_state(&paths);
        assert_eq!(loaded.get("tool@1").unwrap().binaries, stored.binaries);
        assert!(!loaded.get("tool@1").unwrap().installed_at.is_empty());

        fs::remove_dir_all(paths.packwire_root).unwrap();
    }

    #[test]
    fn detects_duplicates_and_binary_collisions() {
        let paths = temporary_paths();
        register_installed(&paths, package("tool@1", &["tool.exe"], &["tool.cmd"])).unwrap();

        let duplicate = check_collision(&paths, "tool@1", &["tool.exe".into()]);
        assert!(duplicate.is_duplicate);
        assert!(duplicate.duplicate_message.is_some());

        let collision = check_collision(&paths, "other@1", &["tool.exe".into()]);
        assert_eq!(
            collision.collisions,
            vec!["tool.exe (ya provisto por 'tool@1')"]
        );

        fs::remove_dir_all(paths.packwire_root).unwrap();
    }

    #[test]
    fn unregisters_package() {
        let paths = temporary_paths();
        register_installed(&paths, package("tool@1", &[], &[])).unwrap();
        let removed = unregister_installed(&paths, "tool@1").unwrap();
        assert_eq!(removed.unwrap().id, "tool@1");
        assert!(get_installed(&paths, "tool@1").is_none());

        fs::remove_dir_all(paths.packwire_root).unwrap();
    }
}
