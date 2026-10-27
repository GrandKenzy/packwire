//! Carga, limpieza y descubrimiento de manifests.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::models::Manifest;

pub type ManifestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

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
        if !file_path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("no se encontró el manifest: {}", file_path.display()),
            )
            .into());
        }

        let raw = fs::read_to_string(file_path)?;
        let cleaned = strip_json_comments_and_trailing_commas(&raw);
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
        [self.builtin_dir.clone(), self.manifests_dir.clone()]
    }
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
