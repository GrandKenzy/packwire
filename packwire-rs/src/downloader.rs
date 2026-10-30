//! Descarga de artefactos con caché, progreso y verificación SHA-256.

use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use reqwest::blocking::Client;
use sha2::{Digest, Sha256};

pub type DownloadResult<T> = Result<T, Box<dyn Error + Send + Sync>>;
pub type ProgressCallback<'a> = dyn Fn(u64, Option<u64>, f64) + Send + Sync + 'a;

const BLOCK_SIZE: usize = 64 * 1024;
const USER_AGENT: &str = "Packwire/1.2.0 (Windows; Rust)";

/// Descargador que almacena los artefactos en un directorio de caché.
#[derive(Debug, Clone)]
pub struct Downloader {
    pub cache_dir: PathBuf,
    client: Client,
}

impl Downloader {
    pub fn new(cache_dir: PathBuf) -> DownloadResult<Self> {
        fs::create_dir_all(&cache_dir)?;
        let client = Client::builder().user_agent(USER_AGENT).build()?;
        Ok(Self { cache_dir, client })
    }

    /// Descarga un archivo, reutilizando la caché cuando es válida.
    ///
    /// `progress_callback` recibe `(bytes_descargados, bytes_totales, bytes_por_segundo)`.
    /// El total es `None` si el servidor no envía `Content-Length`.
    pub fn download(
        &self,
        url: &str,
        dest_filename: Option<&str>,
        expected_hash: Option<&str>,
        progress_callback: Option<&ProgressCallback<'_>>,
        force: bool,
    ) -> DownloadResult<PathBuf> {
        let filename = destination_filename(url, dest_filename)?;
        let destination = self.cache_dir.join(&filename);

        if destination.exists() && !force {
            if let Some(expected_hash) = expected_hash {
                if sha256_file(&destination)? == expected_hash.to_lowercase() {
                    return Ok(destination);
                }
            } else if fs::metadata(&destination)?.len() > 0 {
                return Ok(destination);
            }
        }

        let part_path = self.cache_dir.join(format!("{filename}.part"));
        let result = self.download_to_part(url, &part_path, expected_hash, progress_callback);

        match result {
            Ok(()) => {
                replace_file(&part_path, &destination)?;
                Ok(destination)
            }
            Err(error) => {
                let _ = fs::remove_file(&part_path);
                Err(error)
            }
        }
    }

    fn download_to_part(
        &self,
        url: &str,
        part_path: &Path,
        expected_hash: Option<&str>,
        progress_callback: Option<&ProgressCallback<'_>>,
    ) -> DownloadResult<()> {
        let mut response = self.client.get(url).send()?.error_for_status()?;
        let total = response.content_length();
        let mut output = File::create(part_path)?;
        let mut buffer = [0_u8; BLOCK_SIZE];
        let mut downloaded = 0_u64;
        let start = Instant::now();

        loop {
            let read = response.read(&mut buffer)?;
            if read == 0 {
                break;
            }

            output.write_all(&buffer[..read])?;
            downloaded += read as u64;
            if let Some(callback) = progress_callback {
                let elapsed = start.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 {
                    downloaded as f64 / elapsed
                } else {
                    0.0
                };
                callback(downloaded, total, speed);
            }
        }

        output.flush()?;
        output.sync_all()?;

        if let Some(total) = total {
            if downloaded < total {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!("descarga incompleta: recibidos {downloaded} de {total} bytes"),
                )
                .into());
            }
        }

        if let Some(expected_hash) = expected_hash {
            let actual_hash = sha256_file(part_path)?;
            if actual_hash != expected_hash.to_lowercase() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "error de integridad: hash esperado ({expected_hash}) no coincide con el descargado ({actual_hash})"
                    ),
                )
                .into());
            }
        }

        Ok(())
    }
}

fn destination_filename(url: &str, requested: Option<&str>) -> DownloadResult<String> {
    let filename = requested
        .map(ToOwned::to_owned)
        .or_else(|| {
            url.split(['?', '#'])
                .next()
                .and_then(|value| value.trim_end_matches('/').rsplit('/').next())
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| "download.bin".into());

    let path = Path::new(&filename);
    if filename.is_empty()
        || filename == "."
        || filename == ".."
        || path.file_name().and_then(|value| value.to_str()) != Some(filename.as_str())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("nombre de destino no válido: {filename}"),
        )
        .into());
    }

    Ok(filename)
}

fn sha256_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; BLOCK_SIZE];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    match fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(rename_error) if cfg!(target_os = "windows") && destination.exists() => {
            fs::remove_file(destination).map_err(|_| rename_error)?;
            fs::rename(source, destination)
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_cache() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("packwire-downloader-test-{suffix}"))
    }

    #[test]
    fn derives_and_validates_destination_names() {
        assert_eq!(
            destination_filename("https://example.test/releases/tool.zip?download=1", None)
                .unwrap(),
            "tool.zip"
        );
        assert!(
            destination_filename("https://example.test/tool.zip", Some("../tool.zip")).is_err()
        );
        assert!(
            destination_filename("https://example.test/tool.zip", Some("tool/other.zip")).is_err()
        );
    }

    #[test]
    fn reuses_a_matching_cached_file() {
        let cache = temporary_cache();
        fs::create_dir_all(&cache).unwrap();
        let destination = cache.join("tool.bin");
        fs::write(&destination, b"packwire").unwrap();
        let hash = sha256_file(&destination).unwrap();
        let downloader = Downloader::new(cache.clone()).unwrap();

        let result = downloader
            .download(
                "https://invalid.example/tool.bin",
                None,
                Some(&hash),
                None,
                false,
            )
            .unwrap();
        assert_eq!(result, destination);

        fs::remove_dir_all(cache).unwrap();
    }

    #[test]
    fn hashes_files_with_sha256() {
        let cache = temporary_cache();
        fs::create_dir_all(&cache).unwrap();
        let file = cache.join("hash.txt");
        fs::write(&file, b"packwire").unwrap();
        assert_eq!(
            sha256_file(&file).unwrap(),
            "b57fc524dd90a03184f31719f547a715a8e211c8a6f0c826b380943e9ba4344b"
        );
        fs::remove_dir_all(cache).unwrap();
    }
}
