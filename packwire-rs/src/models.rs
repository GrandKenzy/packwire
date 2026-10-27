//! Modelos tipados de Packwire y compatibilidad con los manifests existentes.

use std::collections::{BTreeMap, HashMap};
use std::io::Write;

use serde::{Deserialize, Serialize};
use serde_json::ser::{Formatter, Serializer};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

/// Formatter canónico con orden estable y separadores compactos.
struct CanonicalJsonFormatter;

impl Formatter for CanonicalJsonFormatter {
    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b", ")
        }
    }

    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b", ")
        }
    }

    fn begin_object_value<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all(b": ")
    }
}

fn string_or_null(value: Option<&Value>) -> Option<String> {
    value.and_then(|value| match value {
        Value::String(value) => Some(value.clone()),
        Value::Null => None,
        value => Some(value.to_string()),
    })
}

fn bool_or(value: Option<&Value>, fallback: bool) -> bool {
    value.and_then(Value::as_bool).unwrap_or(fallback)
}

fn string_vec_or(value: Option<&Value>, fallback: &[&str]) -> Vec<String> {
    match value {
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect(),
        _ => fallback.iter().map(|value| (*value).to_owned()).collect(),
    }
}

fn object(value: &Value) -> Result<&Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| "se esperaba un objeto JSON".to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HereConfig {
    pub url: Option<String>,
    pub format: String,
    pub binaries: Vec<String>,
    pub addpath: bool,
    pub silent_args: Vec<String>,
    pub expected_hash: Option<String>,
}

impl Default for HereConfig {
    fn default() -> Self {
        Self {
            url: None,
            format: "zip".into(),
            binaries: vec!["python.exe".into()],
            addpath: true,
            silent_args: Vec::new(),
            expected_hash: None,
        }
    }
}

impl HereConfig {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let data = object(value)?;
        Ok(Self {
            url: string_or_null(data.get("url")),
            format: data
                .get("format")
                .and_then(Value::as_str)
                .unwrap_or("zip")
                .to_owned(),
            binaries: string_vec_or(data.get("binaries"), &["python.exe"]),
            addpath: data
                .get("addpath")
                .or_else(|| data.get("add_path"))
                .and_then(Value::as_bool)
                .unwrap_or(true),
            silent_args: string_vec_or(data.get("silent_args"), &[]),
            expected_hash: string_or_null(data.get("expected_hash").or_else(|| data.get("sha256"))),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteConfig {
    pub url: String,
}

impl SiteConfig {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let data = object(value)?;
        Ok(Self {
            url: data
                .get("url")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandConfig {
    pub script: Option<String>,
    pub windows: Option<String>,
    pub linux: Option<String>,
    pub darwin: Option<String>,
    pub shell: String,
    pub elevated: bool,
    pub binaries: Vec<String>,
}

impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            script: None,
            windows: None,
            linux: None,
            darwin: None,
            shell: "powershell".into(),
            elevated: false,
            binaries: Vec::new(),
        }
    }
}

impl CommandConfig {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        if let Some(script) = value.as_str() {
            return Ok(Self {
                script: Some(script.to_owned()),
                ..Self::default()
            });
        }

        let data = object(value)?;
        Ok(Self {
            script: string_or_null(data.get("script").or_else(|| data.get("command"))),
            windows: string_or_null(data.get("windows")),
            linux: string_or_null(data.get("linux")),
            darwin: string_or_null(data.get("darwin").or_else(|| data.get("macos"))),
            shell: data
                .get("shell")
                .and_then(Value::as_str)
                .unwrap_or("powershell")
                .to_owned(),
            elevated: bool_or(data.get("elevated"), false),
            binaries: string_vec_or(data.get("binaries"), &[]),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallConfig {
    pub mode: String,
    pub clean: bool,
    pub here: Option<HereConfig>,
    pub site: Option<SiteConfig>,
    pub command: Option<CommandConfig>,
    pub os: Option<HashMap<String, Value>>,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            mode: "here".into(),
            clean: true,
            here: None,
            site: None,
            command: None,
            os: None,
        }
    }
}

impl InstallConfig {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let data = object(value)?;
        let command_value = data
            .get("command")
            .or_else(|| data.get("console"))
            .or_else(|| data.get("script"));

        let os = data.get("os").and_then(Value::as_object).map(|value| {
            value
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect()
        });

        Ok(Self {
            mode: data
                .get("mode")
                .and_then(Value::as_str)
                .unwrap_or("here")
                .to_owned(),
            clean: bool_or(data.get("clean"), true),
            here: data.get("here").map(HereConfig::from_value).transpose()?,
            site: data.get("site").map(SiteConfig::from_value).transpose()?,
            command: command_value.map(CommandConfig::from_value).transpose()?,
            os,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub package_type: String,
    pub version: Option<String>,
    pub icon: String,
    pub category: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub upstream: Option<HashMap<String, Value>>,
    pub auto_update: bool,
    pub install: InstallConfig,
    pub manifest_hash: Option<String>,
}

impl Manifest {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let data = object(value)?;
        let version = match data.get("version") {
            Some(Value::Array(values)) => Some(
                values
                    .iter()
                    .map(|value| match value {
                        Value::String(value) => value.clone(),
                        value => value.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join("."),
            ),
            value => string_or_null(value),
        };

        let mut package_type = data
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("stable")
            .to_lowercase();
        if package_type == "latest" {
            package_type = "stable".into();
        }

        let name = data
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Unknown")
            .to_owned();
        let id = data
            .get("id")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| match version.as_deref() {
                Some(version) if package_type == "fixed" => {
                    format!("{}@{}", name.to_lowercase(), version)
                }
                _ if package_type == "stable" => format!("{}@stable", name.to_lowercase()),
                _ => format!("{}@default", name.to_lowercase()),
            });

        let upstream = data
            .get("upstream")
            .and_then(Value::as_object)
            .map(|value| {
                value
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect()
            });

        let mut manifest = Self {
            id,
            name,
            package_type,
            version,
            icon: data
                .get("icon")
                .and_then(Value::as_str)
                .unwrap_or("📦")
                .to_owned(),
            category: data
                .get("category")
                .and_then(Value::as_str)
                .unwrap_or("general")
                .to_owned(),
            description: data
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            aliases: string_vec_or(data.get("aliases"), &[]),
            upstream,
            auto_update: bool_or(data.get("auto_update"), false),
            install: InstallConfig::from_value(
                data.get("install").unwrap_or(&Value::Object(Map::new())),
            )?,
            manifest_hash: None,
        };
        manifest.manifest_hash = Some(manifest.calculate_hash());
        Ok(manifest)
    }

    /// Calcula el hash SHA-256 canónico del manifest.
    pub fn calculate_hash(&self) -> String {
        let mut install = BTreeMap::new();
        install.insert(
            "addpath",
            json!(self
                .install
                .here
                .as_ref()
                .is_some_and(|value| value.addpath)),
        );
        install.insert(
            "binaries",
            json!(self
                .install
                .here
                .as_ref()
                .map(|value| &value.binaries)
                .or_else(|| self.install.command.as_ref().map(|value| &value.binaries))
                .cloned()
                .unwrap_or_default()),
        );
        install.insert(
            "command_script",
            json!(self
                .install
                .command
                .as_ref()
                .and_then(|value| value.script.clone())),
        );
        install.insert("clean", json!(self.install.clean));
        install.insert(
            "here_hash",
            json!(self
                .install
                .here
                .as_ref()
                .and_then(|value| value.expected_hash.clone())),
        );
        install.insert(
            "here_url",
            json!(self
                .install
                .here
                .as_ref()
                .and_then(|value| value.url.clone())),
        );
        install.insert("mode", json!(self.install.mode));
        install.insert(
            "site_url",
            json!(self.install.site.as_ref().map(|value| value.url.clone())),
        );

        let mut canonical = BTreeMap::new();
        canonical.insert("install", json!(install));
        canonical.insert("name", json!(self.name.to_lowercase()));
        canonical.insert("type", json!(self.package_type));
        canonical.insert("version", json!(self.version));

        let mut canonical_bytes = Vec::new();
        let mut serializer =
            Serializer::with_formatter(&mut canonical_bytes, CanonicalJsonFormatter);
        canonical
            .serialize(&mut serializer)
            .expect("canonical model is serializable");
        let canonical_json =
            String::from_utf8(canonical_bytes).expect("canonical JSON debe ser UTF-8");
        let digest = Sha256::digest(canonical_json.as_bytes());
        format!("{digest:x}")
    }

    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).expect("Manifest siempre debe serializarse")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageState {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub package_type: String,
    pub mode: String,
    pub install_path: Option<String>,
    pub binaries: Vec<String>,
    pub shims: Vec<String>,
    pub installed_at: String,
    pub manifest_hash: String,
}

impl PackageState {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let data = object(value)?;
        Ok(Self {
            id: data
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| "PackageState requiere id".to_owned())?
                .to_owned(),
            name: data
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| "PackageState requiere name".to_owned())?
                .to_owned(),
            version: string_or_null(data.get("version")).unwrap_or_default(),
            package_type: data
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("stable")
                .to_owned(),
            mode: data
                .get("mode")
                .and_then(Value::as_str)
                .unwrap_or("here")
                .to_owned(),
            install_path: string_or_null(data.get("install_path")),
            binaries: string_vec_or(data.get("binaries"), &[]),
            shims: string_vec_or(data.get("shims"), &[]),
            installed_at: string_or_null(data.get("installed_at")).unwrap_or_default(),
            manifest_hash: string_or_null(data.get("manifest_hash")).unwrap_or_default(),
        })
    }

    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).expect("PackageState siempre debe serializarse")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_manifest_compatibly() {
        let value = json!({
            "name": "Python",
            "type": "latest",
            "version": [3, 14],
            "install": {
                "mode": "here",
                "here": {"url": "https://example.test/python.zip", "sha256": "abc"}
            }
        });

        let manifest = Manifest::from_value(&value).unwrap();
        assert_eq!(manifest.id, "python@stable");
        assert_eq!(manifest.package_type, "stable");
        assert_eq!(manifest.version.as_deref(), Some("3.14"));
        assert_eq!(
            manifest.install.here.unwrap().expected_hash.as_deref(),
            Some("abc")
        );
        assert_eq!(
            manifest.manifest_hash.as_deref(),
            Some("0bfff438060abea54353a6a2e97b1113e2ce387161e674bfe0d0c24382373328")
        );
    }

    #[test]
    fn parses_command_string_and_aliases() {
        let value =
            json!({"command": "echo ok", "binaries": ["tool.exe"], "macos": "brew install tool"});
        let command = CommandConfig::from_value(&value).unwrap();
        assert_eq!(command.script.as_deref(), Some("echo ok"));

        let value = json!({"script": {"macos": "brew install tool"}});
        let command = InstallConfig::from_value(&value).unwrap().command.unwrap();
        assert_eq!(command.darwin.as_deref(), Some("brew install tool"));
    }

    #[test]
    fn parses_all_builtin_manifests() {
        let manifests_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("assets")
            .join("manifests");
        let mut count = 0;

        for entry in std::fs::read_dir(manifests_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let content = std::fs::read_to_string(path).unwrap();
            let value: Value = serde_json::from_str(&content).unwrap();
            Manifest::from_value(&value).unwrap();
            count += 1;
        }

        assert!(count >= 44);
    }
}
