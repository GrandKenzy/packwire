//! Resolución de versiones y assets oficiales.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use regex::Regex;
use reqwest::blocking::Client;
use serde_json::Value;

use crate::models::Manifest;

pub type ResolverResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const USER_AGENT: &str = "Packwire/1.2.0 (Rust)";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAsset {
    pub version: String,
    pub download_url: String,
    pub site_url: String,
    pub digest: Option<String>,
}

#[derive(Clone)]
pub struct Resolver {
    client: Client,
    python_cache: Arc<Mutex<Option<Vec<Value>>>>,
    node_cache: Arc<Mutex<Option<Vec<Value>>>>,
    github_cache: Arc<Mutex<HashMap<String, Vec<Value>>>>,
}

impl std::fmt::Debug for Resolver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Resolver").finish_non_exhaustive()
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new().expect("el cliente HTTP de Packwire debe poder inicializarse")
    }
}

impl Resolver {
    pub fn new() -> ResolverResult<Self> {
        Ok(Self {
            client: Client::builder()
                .user_agent(USER_AGENT)
                .timeout(Duration::from_secs(15))
                .build()?,
            python_cache: Arc::new(Mutex::new(None)),
            node_cache: Arc::new(Mutex::new(None)),
            github_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn python_releases(&self) -> ResolverResult<Vec<Value>> {
        if let Some(releases) = self
            .python_cache
            .lock()
            .map_err(|_| std::io::Error::other("la caché de Python está bloqueada"))?
            .clone()
        {
            return Ok(releases);
        }
        let releases: Vec<Value> = self
            .client
            .get("https://www.python.org/api/v2/downloads/release/?is_published=true")
            .send()?
            .error_for_status()?
            .json()?;
        *self
            .python_cache
            .lock()
            .map_err(|_| std::io::Error::other("la caché de Python está bloqueada"))? =
            Some(releases.clone());
        Ok(releases)
    }

    pub fn resolve_python_latest(&self) -> ResolverResult<ResolvedAsset> {
        let release_regex = Regex::new(r"^Python (3\.\d+\.\d+)$")?;
        let mut candidates: Vec<(Vec<u64>, Value)> = self
            .python_releases()?
            .into_iter()
            .filter_map(|release| {
                if release
                    .get("pre_release")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    return None;
                }
                let name = release.get("name").and_then(Value::as_str)?;
                let captures = release_regex.captures(name)?;
                let version = captures.get(1)?.as_str();
                let parts = version
                    .split('.')
                    .map(str::parse::<u64>)
                    .collect::<Result<Vec<_>, _>>()
                    .ok()?;
                Some((parts, release))
            })
            .collect();
        candidates.sort_by(|left, right| right.0.cmp(&left.0));

        let release = candidates
            .into_iter()
            .next()
            .map(|(_, release)| release)
            .ok_or("no se encontró una versión estable de Python 3")?;
        let name = release
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let version = name.trim_start_matches("Python ").to_owned();
        let release_id = release
            .get("resource_uri")
            .and_then(Value::as_str)
            .and_then(|uri| uri.trim_end_matches('/').rsplit('/').next())
            .unwrap_or_default();

        let download_url = if release_id.is_empty() {
            None
        } else {
            let url = format!(
                "https://www.python.org/api/v2/downloads/release_file/?release={release_id}"
            );
            self.client
                .get(url)
                .send()
                .ok()
                .and_then(|response| response.json::<Vec<Value>>().ok())
                .and_then(|files| {
                    files.into_iter().find_map(|file| {
                        let name = file.get("name").and_then(Value::as_str).unwrap_or_default();
                        name.contains("Windows embeddable package (64-bit)")
                            .then(|| {
                                file.get("url")
                                    .and_then(Value::as_str)
                                    .map(ToOwned::to_owned)
                            })
                            .flatten()
                    })
                })
        }
        .unwrap_or_else(|| {
            format!("https://www.python.org/ftp/python/{version}/python-{version}-embed-amd64.zip")
        });
        let slug = release
            .get("slug")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| version.replace('.', ""));

        Ok(ResolvedAsset {
            version,
            download_url,
            site_url: format!("https://www.python.org/downloads/release/{slug}/"),
            digest: None,
        })
    }

    pub fn resolve_python_fixed(&self, version: &str) -> ResolvedAsset {
        let version = if version.split('.').count() == 2 {
            format!("{version}.0")
        } else {
            version.to_owned()
        };
        ResolvedAsset {
            download_url: format!(
                "https://www.python.org/ftp/python/{version}/python-{version}-embed-amd64.zip"
            ),
            site_url: format!(
                "https://www.python.org/downloads/release/python-{}/",
                version.replace('.', "")
            ),
            version,
            digest: None,
        }
    }

    pub fn node_releases(&self) -> ResolverResult<Vec<Value>> {
        if let Some(releases) = self
            .node_cache
            .lock()
            .map_err(|_| std::io::Error::other("la caché de Node.js está bloqueada"))?
            .clone()
        {
            return Ok(releases);
        }
        let releases: Vec<Value> = self
            .client
            .get("https://nodejs.org/dist/index.json")
            .send()?
            .error_for_status()?
            .json()?;
        *self
            .node_cache
            .lock()
            .map_err(|_| std::io::Error::other("la caché de Node.js está bloqueada"))? =
            Some(releases.clone());
        Ok(releases)
    }

    pub fn resolve_node_latest(&self) -> ResolverResult<ResolvedAsset> {
        for release in self.node_releases()? {
            let is_lts = release
                .get("lts")
                .map(|value| !value.is_null() && value.as_bool() != Some(false))
                .unwrap_or(false);
            let has_windows_zip = release
                .get("files")
                .and_then(Value::as_array)
                .map(|files| {
                    files
                        .iter()
                        .any(|file| file.as_str() == Some("win-x64-zip"))
                })
                .unwrap_or(false);
            if is_lts && has_windows_zip {
                let version = release
                    .get("version")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .trim_start_matches('v')
                    .to_owned();
                return Ok(node_asset(&version));
            }
        }
        Ok(node_asset("24.21.0"))
    }

    pub fn resolve_node_fixed(&self, version: &str) -> ResolvedAsset {
        node_asset(version.trim_start_matches('v'))
    }

    pub fn github_releases(&self, repository: &str) -> ResolverResult<Vec<Value>> {
        if let Some(releases) = self
            .github_cache
            .lock()
            .map_err(|_| std::io::Error::other("la caché de GitHub está bloqueada"))?
            .get(repository)
            .cloned()
        {
            return Ok(releases);
        }
        let url = format!("https://api.github.com/repos/{repository}/releases?per_page=30");
        let releases: Vec<Value> = self
            .client
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .send()?
            .error_for_status()?
            .json()?;
        self.github_cache
            .lock()
            .map_err(|_| std::io::Error::other("la caché de GitHub está bloqueada"))?
            .insert(repository.to_owned(), releases.clone());
        Ok(releases)
    }

    pub fn resolve_github(
        &self,
        repository: &str,
        asset_pattern: &str,
        version_pattern: &str,
        site_url: &str,
        release_tag: Option<&str>,
    ) -> ResolverResult<ResolvedAsset> {
        let asset_regex = Regex::new(&format!("(?i)^(?:{asset_pattern})$"))?;
        let version_regex = Regex::new(&format!("(?i){version_pattern}"))?;

        for release in self.github_releases(repository)? {
            if release_tag
                .is_some_and(|tag| release.get("tag_name").and_then(Value::as_str) != Some(tag))
            {
                continue;
            }
            if release
                .get("draft")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                || release
                    .get("prerelease")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            {
                continue;
            }
            for asset in release
                .get("assets")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let asset_name = asset
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if !asset_regex.is_match(asset_name) {
                    continue;
                }
                let capture = version_regex.captures(asset_name).or_else(|| {
                    version_regex.captures(
                        release
                            .get("tag_name")
                            .and_then(Value::as_str)
                            .unwrap_or_default(),
                    )
                });
                let Some(capture) = capture.and_then(|captures| captures.get(1)) else {
                    continue;
                };
                let digest = asset
                    .get("digest")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .strip_prefix("sha256:")
                    .unwrap_or_else(|| {
                        asset
                            .get("digest")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                    });
                return Ok(ResolvedAsset {
                    version: capture.as_str().to_owned(),
                    download_url: asset
                        .get("browser_download_url")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned(),
                    site_url: site_url.to_owned(),
                    digest: (!digest.is_empty()).then(|| digest.to_owned()),
                });
            }
        }
        Err(format!("no se encontró un asset estable compatible en {repository}").into())
    }
}

fn node_asset(version: &str) -> ResolvedAsset {
    ResolvedAsset {
        version: version.to_owned(),
        download_url: format!("https://nodejs.org/dist/v{version}/node-v{version}-win-x64.zip"),
        site_url: "https://nodejs.org/en/download".into(),
        digest: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture_resolver() -> Resolver {
        let resolver = Resolver::new().expect("el cliente de prueba debe inicializarse");
        let releases = vec![json!({
            "tag_name": "v1.2.3",
            "draft": false,
            "prerelease": false,
            "assets": [{
                "name": "W64DEVKIT-X64-1.2.3.7Z.EXE",
                "browser_download_url": "https://example.test/w64devkit.exe",
                "digest": "sha256:abc123"
            }]
        })];
        resolver
            .github_cache
            .lock()
            .expect("la caché debe estar disponible")
            .insert("skeeto/w64devkit".into(), releases);
        resolver
    }

    #[test]
    fn resolves_fixed_node_without_network() {
        let asset = Resolver::new()
            .expect("el cliente de prueba debe inicializarse")
            .resolve_node_fixed("v22.1.0");
        assert_eq!(asset.version, "22.1.0");
        assert_eq!(
            asset.download_url,
            "https://nodejs.org/dist/v22.1.0/node-v22.1.0-win-x64.zip"
        );
    }

    #[test]
    fn resolves_github_asset_case_insensitively_and_strips_sha256_prefix() {
        let resolver = fixture_resolver();
        let asset = resolve_w64devkit_fixed(&resolver, "1.2.3").expect("asset válido");
        assert_eq!(asset.version, "1.2.3");
        assert_eq!(asset.download_url, "https://example.test/w64devkit.exe");
        assert_eq!(asset.digest.as_deref(), Some("abc123"));
    }

    #[test]
    fn resolves_manifest_and_recalculates_hash() {
        let resolver = fixture_resolver();
        let mut manifest = Manifest::from_value(&json!({
            "name": "w64devkit",
            "type": "fixed",
            "version": "1.2.3",
            "upstream": {"source": "github_w64devkit"},
            "install": {
                "here": {"url": null, "expected_hash": null},
                "site": {"url": ""}
            }
        }))
        .expect("manifest válido");

        resolve_manifest(&resolver, &mut manifest).expect("resolución válida");

        assert_eq!(manifest.id, "w64devkit@1.2.3");
        assert_eq!(
            manifest
                .install
                .here
                .as_ref()
                .and_then(|here| here.url.as_deref()),
            Some("https://example.test/w64devkit.exe")
        );
        assert_eq!(
            manifest
                .install
                .here
                .as_ref()
                .and_then(|here| here.expected_hash.as_deref()),
            Some("abc123")
        );
        assert_eq!(manifest.manifest_hash, Some(manifest.calculate_hash()));
    }
}

pub fn resolve_w64devkit_latest(resolver: &Resolver) -> ResolverResult<ResolvedAsset> {
    resolver.resolve_github(
        "skeeto/w64devkit",
        r"w64devkit-x64-(\d+(?:\.\d+)+)\.7z\.exe",
        r"(\d+(?:\.\d+)+)(?=\.7z\.exe)",
        "https://github.com/skeeto/w64devkit/releases",
        None,
    )
}

pub fn resolve_w64devkit_fixed(
    resolver: &Resolver,
    version: &str,
) -> ResolverResult<ResolvedAsset> {
    let version = version.trim_start_matches('v');
    let escaped = regex::escape(version);
    resolver.resolve_github(
        "skeeto/w64devkit",
        &format!(r"w64devkit-x64-{}\.7z\.exe", escaped),
        &format!(r"({})", escaped),
        "https://github.com/skeeto/w64devkit/releases",
        Some(&format!("v{version}")),
    )
}

pub fn resolve_llvm_latest(resolver: &Resolver) -> ResolverResult<ResolvedAsset> {
    resolver.resolve_github(
        "llvm/llvm-project",
        r"clang\+llvm-(\d+(?:\.\d+)+)-x86_64-pc-windows-msvc\.tar\.xz",
        r"(\d+(?:\.\d+)+)",
        "https://github.com/llvm/llvm-project/releases",
        None,
    )
}

pub fn resolve_llvm_fixed(resolver: &Resolver, version: &str) -> ResolverResult<ResolvedAsset> {
    let version = version.trim_start_matches('v');
    let escaped = regex::escape(version);
    resolver.resolve_github(
        "llvm/llvm-project",
        &format!(r"clang\+llvm-{}-x86_64-pc-windows-msvc\.tar\.xz", escaped),
        &format!(r"({})", escaped),
        "https://github.com/llvm/llvm-project/releases",
        Some(&format!("llvmorg-{version}")),
    )
}

pub fn resolve_rufus_latest(resolver: &Resolver) -> ResolverResult<ResolvedAsset> {
    resolver.resolve_github(
        "pbatard/rufus",
        r"rufus-(\d+(?:\.\d+)+)p\.exe",
        r"(\d+(?:\.\d+)+)(?=p\.exe)",
        "https://rufus.ie/en/",
        None,
    )
}

pub fn resolve_ventoy_latest(resolver: &Resolver) -> ResolverResult<ResolvedAsset> {
    resolver.resolve_github(
        "ventoy/Ventoy",
        r"ventoy-(\d+(?:\.\d+)+)-windows\.zip",
        r"(\d+(?:\.\d+)+)(?=-windows\.zip)",
        "https://www.ventoy.net/en/download.html",
        None,
    )
}

pub fn resolve_powershell_latest(resolver: &Resolver) -> ResolverResult<ResolvedAsset> {
    resolver.resolve_github(
        "PowerShell/PowerShell",
        r"PowerShell-(\d+(?:\.\d+)+)-win-x64\.zip",
        r"(\d+(?:\.\d+)+)(?=-win-x64\.zip)",
        "https://learn.microsoft.com/powershell/",
        None,
    )
}

fn apply_asset(manifest: &mut Manifest, asset: &ResolvedAsset, overwrite_urls: bool) {
    if let Some(here) = manifest.install.here.as_mut() {
        if overwrite_urls || here.url.is_none() {
            here.url = Some(asset.download_url.clone());
        }
        if overwrite_urls {
            here.expected_hash = asset.digest.clone();
        }
    }
    if let Some(site) = manifest.install.site.as_mut() {
        if overwrite_urls || site.url.is_empty() {
            site.url = asset.site_url.clone();
        }
    }
}

pub fn resolve_manifest(resolver: &Resolver, manifest: &mut Manifest) -> ResolverResult<()> {
    let package_name = manifest.name.to_lowercase();

    if package_name == "python" {
        let asset = match manifest.package_type.as_str() {
            "stable" => Some(resolver.resolve_python_latest()?),
            "fixed" => {
                let version = manifest.version.as_deref().ok_or(
                    "un paquete 'fixed' de Python debe especificar una versión en el manifest",
                )?;
                Some(resolver.resolve_python_fixed(version))
            }
            _ => None,
        };
        if let Some(asset) = asset {
            manifest.version = Some(asset.version.clone());
            manifest.id = if manifest.package_type == "stable" {
                "python@stable".into()
            } else {
                format!("python@{}", asset.version)
            };
            apply_asset(manifest, &asset, false);
        }
    } else if matches!(package_name.as_str(), "node" | "nodejs" | "node.js") {
        let asset = match manifest.package_type.as_str() {
            "stable" => Some(resolver.resolve_node_latest()?),
            "fixed" => {
                Some(resolver.resolve_node_fixed(manifest.version.as_deref().unwrap_or("24.21.0")))
            }
            _ => None,
        };
        if let Some(asset) = asset {
            manifest.version = Some(asset.version.clone());
            manifest.id = if manifest.package_type == "stable" {
                "nodejs@stable".into()
            } else {
                format!("nodejs@{}", asset.version)
            };
            apply_asset(manifest, &asset, false);
        }
    }

    let source = manifest
        .upstream
        .as_ref()
        .and_then(|upstream| upstream.get("source"))
        .and_then(Value::as_str)
        .unwrap_or_default();

    let asset = match (source, manifest.package_type.as_str()) {
        ("github_w64devkit", "stable") => Some(resolve_w64devkit_latest(resolver)?),
        ("github_w64devkit", "fixed") => Some(resolve_w64devkit_fixed(
            resolver,
            manifest
                .version
                .as_deref()
                .ok_or("un paquete fixed de w64devkit debe especificar una versión")?,
        )?),
        ("github_llvm", "stable") => Some(resolve_llvm_latest(resolver)?),
        ("github_llvm", "fixed") => Some(resolve_llvm_fixed(
            resolver,
            manifest
                .version
                .as_deref()
                .ok_or("un paquete fixed de LLVM debe especificar una versión")?,
        )?),
        ("github_rufus", "stable") => Some(resolve_rufus_latest(resolver)?),
        ("github_ventoy", "stable") => Some(resolve_ventoy_latest(resolver)?),
        ("github_powershell", "stable") => Some(resolve_powershell_latest(resolver)?),
        _ => None,
    };

    if let Some(asset) = asset {
        manifest.version = Some(asset.version.clone());
        apply_asset(manifest, &asset, true);
    }
    manifest.manifest_hash = Some(manifest.calculate_hash());
    Ok(())
}
