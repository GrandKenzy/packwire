//! Carga, limpieza y descubrimiento de manifests.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::Manifest;

pub type ManifestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const MANIFESTS_API_URL: &str =
    "https://api.github.com/repos/GrandKenzy/packwire/contents/assets/manifests?ref=main";
const MANIFESTS_RAW_URL: &str =
    "https://raw.githubusercontent.com/GrandKenzy/packwire/main/assets/manifests";
const SYNC_METADATA_FILE: &str = ".packwire-catalog-sync";
const USER_AGENT: &str = "Packwire/1.2.0 (catalog-sync)";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestSyncReport {
    pub success: bool,
    pub updated: bool,
    pub downloaded: usize,
    pub removed: usize,
    pub checked_at: u64,
    pub from_cache: bool,
    pub message: String,
}

impl ManifestSyncReport {
    fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            updated: false,
            downloaded: 0,
            removed: 0,
            checked_at: unix_timestamp(),
            from_cache: true,
            message: message.into(),
        }
    }

    pub fn cached() -> Self {
        Self {
            success: true,
            updated: false,
            downloaded: 0,
            removed: 0,
            checked_at: unix_timestamp(),
            from_cache: true,
            message: "Se usará el catálogo local disponible.".into(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct SyncMetadata {
    source: String,
    branch: String,
    entries: BTreeMap<String, String>,
    checked_at: u64,
}

#[derive(Debug, Deserialize)]
struct GitHubContentEntry {
    name: String,
    path: String,
    sha: String,
    #[serde(rename = "type")]
    entry_type: String,
}

#[derive(Debug, Clone)]
pub struct ManifestLoader {
    pub builtin_dir: PathBuf,
    pub manifests_dir: PathBuf,
}

impl ManifestLoader {
    pub fn new(manifests_dir: PathBuf) -> Self {
        let source_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("assets")
            .join("manifests");
        let installed_dir = std::env::current_exe()
            .ok()
            .and_then(|executable| executable.parent().map(|parent| parent.join("manifests")));
        let builtin_dir = installed_dir
            .filter(|directory| directory.is_dir())
            .unwrap_or(source_dir);
        Self {
            builtin_dir,
            manifests_dir,
        }
    }

    pub fn with_dirs(builtin_dir: PathBuf, manifests_dir: PathBuf) -> Self {
        Self {
            builtin_dir,
            manifests_dir,
        }
    }

    pub fn load_from_file(&self, file_path: &Path) -> ManifestResult<Manifest> {
        load_manifest_from_file(file_path)
    }

    pub fn find_manifest(&self, identifier: &str) -> Option<Manifest> {
        if !is_simple_identifier(identifier) {
            return None;
        }

        let normalized = identifier.to_lowercase().replace(['@', ':'], "-");
        let mut names = vec![
            format!("{normalized}.json"),
            format!("{identifier}.json"),
            format!("{}.json", normalized.replace("python-", "python-fixed-")),
            format!("{}.json", normalized.replace("python-", "python-stable-")),
            "manifest.json".into(),
        ];
        if !identifier.contains('@') {
            names.push(format!("{normalized}-stable.json"));
        }

        for directory in self.search_dirs() {
            if !directory.is_dir() {
                continue;
            }
            for name in &names {
                let candidate = directory.join(name);
                if candidate.is_file() {
                    if let Ok(manifest) = self.load_from_file(&candidate) {
                        return Some(manifest);
                    }
                }
            }

            let entries = fs::read_dir(&directory).ok()?;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) != Some("json") {
                    continue;
                }
                let Ok(manifest) = self.load_from_file(&path) else {
                    continue;
                };
                if manifest_matches(&manifest, identifier) {
                    return Some(manifest);
                }
            }

            let entries = fs::read_dir(&directory).ok()?;
            for entry in entries.flatten() {
                let subdirectory = entry.path();
                if !subdirectory.is_dir() {
                    continue;
                }
                let candidate = subdirectory.join("manifest.json");
                if candidate.is_file() {
                    if let Ok(manifest) = self.load_from_file(&candidate) {
                        if manifest_matches(&manifest, identifier) {
                            return Some(manifest);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn list_available_manifests(&self) -> Vec<Manifest> {
        let mut manifests = Vec::new();
        let mut seen_ids = HashSet::new();

        for directory in self.search_dirs() {
            let Ok(entries) = fs::read_dir(directory) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) != Some("json") {
                    continue;
                }
                let Ok(manifest) = self.load_from_file(&path) else {
                    continue;
                };
                if seen_ids.insert(manifest.id.clone()) {
                    manifests.push(manifest);
                }
            }
        }
        manifests
    }

    fn search_dirs(&self) -> [PathBuf; 2] {
        [self.manifests_dir.clone(), self.builtin_dir.clone()]
    }
}

/// Sincroniza el catálogo remoto de Packwire en el caché local.
///
/// El listado se obtiene desde la API de contenidos de GitHub y los JSON se
/// descargan desde raw.githubusercontent.com. La caché se conserva completa
/// cuando la red falla, de modo que Packwire sigue funcionando offline después
/// de una primera sincronización correcta.
pub fn sync_remote_catalog(manifests_dir: &Path) -> ManifestSyncReport {
    let checked_at = unix_timestamp();
    if let Err(error) = fs::create_dir_all(manifests_dir) {
        return ManifestSyncReport::failure(format!(
            "no se pudo preparar la caché del catálogo: {error}"
        ));
    }

    let metadata_path = manifests_dir.join(SYNC_METADATA_FILE);
    let previous = read_sync_metadata(&metadata_path);
    let client = match Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent(USER_AGENT)
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            return ManifestSyncReport::failure(format!(
                "no se pudo preparar la conexión con el catálogo: {error}"
            ));
        }
    };

    let remote_entries = match client
        .get(MANIFESTS_API_URL)
        .header("Accept", "application/vnd.github+json")
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .and_then(|response| response.json::<Vec<GitHubContentEntry>>())
    {
        Ok(entries) => entries
            .into_iter()
            .filter(|entry| {
                entry.entry_type == "file"
                    && entry.name.ends_with(".json")
                    && entry.path == format!("assets/manifests/{}", entry.name)
                    && is_simple_identifier(&entry.name)
            })
            .map(|entry| (entry.name, entry.sha))
            .collect::<BTreeMap<_, _>>(),
        Err(error) => {
            return ManifestSyncReport::failure(format!(
                "no se pudo consultar el catálogo remoto; se conserva la caché local: {error}"
            ));
        }
    };

    if remote_entries.is_empty() {
        return ManifestSyncReport::failure(
            "GitHub devolvió una carpeta de manifests vacía; se conserva la caché local.",
        );
    }

    let mut downloaded = 0;
    let mut errors = Vec::new();
    for (name, sha) in &remote_entries {
        let target = manifests_dir.join(name);
        let cached_is_current = previous.entries.get(name) == Some(sha)
            && target.is_file()
            && load_manifest_from_file(&target).is_ok();
        if cached_is_current {
            continue;
        }

        let url = format!("{MANIFESTS_RAW_URL}/{name}");
        let result: Result<String, String> = match client
            .get(url)
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
        {
            Ok(response) => response
                .text()
                .map_err(|error| error.to_string())
                .and_then(|raw| {
                    load_manifest_from_text(&raw, &target)
                        .map(|_| raw)
                        .map_err(|error| error.to_string())
                }),
            Err(error) => Err(error.to_string()),
        };

        match result {
            Ok(raw) => match write_atomic(&target, raw.as_bytes()) {
                Ok(()) => downloaded += 1,
                Err(error) => errors.push(format!("{name}: no se pudo guardar: {error}")),
            },
            Err(error) => errors.push(format!("{name}: no se pudo descargar o validar: {error}")),
        }
    }

    if !errors.is_empty() {
        return ManifestSyncReport {
            success: false,
            updated: downloaded > 0,
            downloaded,
            removed: 0,
            checked_at,
            from_cache: false,
            message: format!(
                "La sincronización quedó incompleta; se conserva el catálogo anterior. {}",
                errors.join("; ")
            ),
        };
    }

    let mut removed = 0;
    if let Ok(entries) = fs::read_dir(manifests_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.file_name().and_then(|value| value.to_str()) == Some(SYNC_METADATA_FILE) {
                continue;
            }
            let is_json = path.extension().and_then(|value| value.to_str()) == Some("json");
            let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if is_json && !remote_entries.contains_key(name) && path.is_file() {
                if fs::remove_file(&path).is_ok() {
                    removed += 1;
                }
            }
        }
    }

    let metadata = SyncMetadata {
        source: MANIFESTS_API_URL.into(),
        branch: "main".into(),
        entries: remote_entries,
        checked_at,
    };
    if let Ok(content) = serde_json::to_vec_pretty(&metadata) {
        let _ = write_atomic(&metadata_path, &content);
    }

    ManifestSyncReport {
        success: true,
        updated: downloaded > 0 || removed > 0,
        downloaded,
        removed,
        checked_at,
        from_cache: false,
        message: if downloaded > 0 || removed > 0 {
            "Catálogo remoto actualizado correctamente.".into()
        } else {
            "El catálogo remoto ya estaba actualizado.".into()
        },
    }
}

fn load_manifest_from_file(file_path: &Path) -> ManifestResult<Manifest> {
    if !file_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("no se encontró el manifest: {}", file_path.display()),
        )
        .into());
    }

    let raw = fs::read_to_string(file_path)?;
    load_manifest_from_text(&raw, file_path)
}

fn load_manifest_from_text(raw: &str, file_path: &Path) -> ManifestResult<Manifest> {
    let cleaned = strip_json_comments_and_trailing_commas(raw);
    let value: Value = serde_json::from_str(&cleaned).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "el manifest {} no contiene JSON válido: {error}",
                file_path.display()
            ),
        )
    })?;
    Ok(Manifest::from_value(&value).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("manifest inválido {}: {error}", file_path.display()),
        )
    })?)
}

fn read_sync_metadata(path: &Path) -> SyncMetadata {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn write_atomic(path: &Path, content: &[u8]) -> io::Result<()> {
    let temporary = path.with_extension("packwire-tmp");
    fs::write(&temporary, content)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temporary, path)
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn manifest_matches(manifest: &Manifest, identifier: &str) -> bool {
    let identifier = identifier.to_lowercase();
    let base_identifier = identifier.split('@').next().unwrap_or(&identifier);
    let names: Vec<String> = std::iter::once(manifest.name.to_lowercase())
        .chain(manifest.aliases.iter().map(|alias| alias.to_lowercase()))
        .collect();

    manifest.id.to_lowercase() == identifier
        || names
            .iter()
            .any(|name| name == &identifier || name == base_identifier)
        || names
            .iter()
            .any(|name| normalize_name(name) == normalize_name(&identifier))
}

fn normalize_name(value: &str) -> String {
    value.replace(['-', '_'], "")
}

fn is_simple_identifier(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && Path::new(value).file_name().and_then(|name| name.to_str()) == Some(value)
}

pub fn strip_json_comments_and_trailing_commas(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;

    while index < chars.len() {
        let current = chars[index];
        if in_string {
            output.push(current);
            if escaped {
                escaped = false;
            } else if current == '\\' {
                escaped = true;
            } else if current == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if current == '"' {
            in_string = true;
            output.push(current);
            index += 1;
            continue;
        }
        if current == '/' && chars.get(index + 1) == Some(&'/') {
            index += 2;
            while index < chars.len() && chars[index] != '\n' {
                index += 1;
            }
            continue;
        }
        if current == '/' && chars.get(index + 1) == Some(&'*') {
            index += 2;
            while index + 1 < chars.len() && !(chars[index] == '*' && chars[index + 1] == '/') {
                index += 1;
            }
            index = (index + 2).min(chars.len());
            continue;
        }
        if current == ',' {
            let mut lookahead = index + 1;
            loop {
                while lookahead < chars.len() && chars[lookahead].is_whitespace() {
                    lookahead += 1;
                }
                if chars.get(lookahead) == Some(&'/') && chars.get(lookahead + 1) == Some(&'/') {
                    lookahead += 2;
                    while lookahead < chars.len() && chars[lookahead] != '\n' {
                        lookahead += 1;
                    }
                    continue;
                }
                if chars.get(lookahead) == Some(&'/') && chars.get(lookahead + 1) == Some(&'*') {
                    lookahead += 2;
                    while lookahead + 1 < chars.len()
                        && !(chars[lookahead] == '*' && chars[lookahead + 1] == '/')
                    {
                        lookahead += 1;
                    }
                    lookahead = (lookahead + 2).min(chars.len());
                    continue;
                }
                break;
            }
            if matches!(chars.get(lookahead), Some(']') | Some('}')) {
                index += 1;
                continue;
            }
        }
        output.push(current);
        index += 1;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_dirs() -> (PathBuf, PathBuf) {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        (
            env::temp_dir().join(format!("packwire-manifest-builtins-{suffix}")),
            env::temp_dir().join(format!("packwire-manifest-user-{suffix}")),
        )
    }

    #[test]
    fn strips_comments_and_trailing_commas_without_changing_strings() {
        let input = r#"{
            // comment
            "url": "https://example.test/a//b",
            "items": [1, 2,], /* block comment */
        }"#;
        let cleaned = strip_json_comments_and_trailing_commas(input);
        let value: Value = serde_json::from_str(&cleaned).unwrap();
        assert_eq!(value["url"], "https://example.test/a//b");
        assert_eq!(value["items"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn parses_all_builtin_manifests() {
        let builtin = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("assets")
            .join("manifests");
        let (_, user) = temporary_dirs();
        let loader = ManifestLoader::with_dirs(builtin, user);
        assert!(loader.list_available_manifests().len() >= 44);
    }

    #[test]
    fn finds_manifest_by_alias_and_rejects_traversal() {
        let builtin = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("assets")
            .join("manifests");
        let (_, user) = temporary_dirs();
        let loader = ManifestLoader::with_dirs(builtin, user);
        assert!(loader.find_manifest("python").is_some());
        assert!(loader.find_manifest("../python").is_none());
    }
}
