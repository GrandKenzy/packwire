//! Orquestación de instalaciones en modos `here`, `command` y `site`.
//! La extracción de archivos se realiza de forma explícita para impedir
//! Zip-Slip y enlaces peligrosos.

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use tar::Archive;
use xz2::read::XzDecoder;
use zip::ZipArchive;

use crate::config::{ensure_directories, system, Paths};
use crate::downloader::Downloader;
use crate::models::{CommandConfig, HereConfig, InstallConfig, Manifest, PackageState};
use crate::pather::Pather;
use crate::state::{check_collision, register_installed};

pub type InstallResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type Logger<'a> = dyn Fn(&str) + Send + Sync + 'a;

#[derive(Debug, Clone)]
pub struct Installer {
    pub paths: Paths,
    pub downloader: Downloader,
    pub pather: Pather,
}

impl Installer {
    pub fn new(paths: Paths) -> InstallResult<Self> {
        ensure_directories(&paths)?;
        Ok(Self {
            downloader: Downloader::new(paths.cache_dir.clone())?,
            pather: Pather::new(paths.shims_dir.clone())?,
            paths,
        })
    }

    pub fn install(
        &self,
        manifest: &mut Manifest,
        mode_override: Option<&str>,
        force: bool,
        logger: Option<&Logger<'_>>,
    ) -> InstallResult<(bool, String)> {
        self.apply_os_override(manifest)?;

        let mode = mode_override
            .unwrap_or(&manifest.install.mode)
            .trim()
            .to_lowercase();
        let binaries = manifest_binaries(manifest);
        let collision = check_collision(&self.paths, &manifest.id, &binaries);

        if collision.is_duplicate && !force {
            return Ok((
                false,
                collision
                    .duplicate_message
                    .unwrap_or_else(|| format!("El paquete '{}' ya está instalado.", manifest.id)),
            ));
        }

        if !collision.collisions.is_empty() && !force {
            let warning = format!(
                "Advertencia: colisión detectada en binarios ({}).",
                collision.collisions.join(", ")
            );
            emit(logger, &warning);
        }

        match mode.as_str() {
            "site" => self.install_site(manifest, logger),
            "here" => self.install_here(manifest, &collision.collisions, force, logger),
            "command" | "console" | "script" => self.install_command(manifest, force, logger),
            _ => Ok((
                false,
                format!("Modo de instalación no reconocido: '{mode}'"),
            )),
        }
    }

    fn apply_os_override(&self, manifest: &mut Manifest) -> InstallResult<()> {
        let Some(os_data) = manifest
            .install
            .os
            .as_ref()
            .and_then(|data| data.get(system()))
        else {
            return Ok(());
        };

        let override_config = InstallConfig::from_value(os_data)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if !override_config.mode.is_empty() {
            manifest.install.mode = override_config.mode;
        }
        if override_config.here.is_some() {
            manifest.install.here = override_config.here;
        }
        if override_config.command.is_some() {
            manifest.install.command = override_config.command;
        }
        if override_config.site.is_some() {
            manifest.install.site = override_config.site;
        }
        Ok(())
    }

    fn install_site(
        &self,
        manifest: &Manifest,
        logger: Option<&Logger<'_>>,
    ) -> InstallResult<(bool, String)> {
        let Some(site) = manifest.install.site.as_ref() else {
            return Ok((
                false,
                format!(
                    "El paquete '{}' no tiene configurada una URL en 'site'.",
                    manifest.id
                ),
            ));
        };
        if site.url.trim().is_empty() {
            return Ok((
                false,
                format!(
                    "El paquete '{}' no tiene configurada una URL en 'site'.",
                    manifest.id
                ),
            ));
        }

        let message = format!(
            "Redirigiendo al sitio oficial para instalación manual: {}",
            site.url
        );
        emit(logger, &message);
        open_url(&site.url)?;

        let state = PackageState {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest
                .version
                .clone()
                .unwrap_or_else(|| "site-redirect".into()),
            package_type: manifest.package_type.clone(),
            mode: "site".into(),
            install_path: None,
            binaries: Vec::new(),
            shims: Vec::new(),
            installed_at: String::new(),
            manifest_hash: manifest.manifest_hash.clone().unwrap_or_default(),
        };
        register_installed(&self.paths, state)?;
        Ok((
            true,
            format!(
                "Navegador abierto en {}. Instalación manual iniciada.",
                site.url
            ),
        ))
    }

    fn install_here(
        &self,
        manifest: &Manifest,
        collisions: &[String],
        force: bool,
        logger: Option<&Logger<'_>>,
    ) -> InstallResult<(bool, String)> {
        let Some(here) = manifest.install.here.as_ref() else {
            return Ok((
                false,
                format!(
                    "El paquete '{}' no tiene una URL configurada para el modo 'here'.",
                    manifest.id
                ),
            ));
        };
        let Some(url) = here.url.as_deref() else {
            return Ok((
                false,
                format!(
                    "El paquete '{}' no tiene una URL configurada para el modo 'here'.",
                    manifest.id
                ),
            ));
        };

        let version = manifest.version.as_deref().unwrap_or("latest");
        let target_dir = self
            .paths
            .apps_dir
            .join(safe_component(&manifest.id))
            .join(safe_component(version));
        fs::create_dir_all(&target_dir)?;
        emit(
            logger,
            &format!(
                "Instalando '{}' ({version}) en: {}",
                manifest.name,
                target_dir.display()
            ),
        );

        emit(logger, &format!("Iniciando descarga desde: {url}"));
        let downloaded =
            self.downloader
                .download(url, None, here.expected_hash.as_deref(), None, force)?;
        emit(
            logger,
            &format!("Descarga completada: {}", downloaded.display()),
        );

        let extraction = self.extract_artifact(&downloaded, &target_dir, here, manifest, logger);
        if let Err(error) = extraction {
            return Ok((false, error.to_string()));
        }

        let mut created_shims = Vec::new();
        if here.addpath {
            let is_default = collisions.is_empty() || force;
            emit(
                logger,
                &format!(
                    "Creando shims para binarios: {}...",
                    here.binaries.join(", ")
                ),
            );
            created_shims = self.pather.create_shims(
                &target_dir,
                &here.binaries,
                manifest.version.as_deref(),
                is_default,
            )?;
            self.pather.add_shims_to_user_path()?;
        }

        if manifest.install.clean {
            let _ = fs::remove_file(&downloaded);
            emit(
                logger,
                &format!("Archivo temporal limpio: {}", downloaded.display()),
            );
        }

        let state = PackageState {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: version.into(),
            package_type: manifest.package_type.clone(),
            mode: "here".into(),
            install_path: Some(target_dir.to_string_lossy().into_owned()),
            binaries: here.binaries.clone(),
            shims: created_shims,
            installed_at: String::new(),
            manifest_hash: manifest.manifest_hash.clone().unwrap_or_default(),
        };
        register_installed(&self.paths, state)?;

        emit(
            logger,
            &format!("¡'{}' ({version}) instalado con éxito!", manifest.name),
        );
        Ok((
            true,
            format!(
                "'{}' ({version}) instalado con éxito en modo 'here'.",
                manifest.name
            ),
        ))
    }

    fn extract_artifact(
        &self,
        downloaded: &Path,
        target_dir: &Path,
        config: &HereConfig,
        manifest: &Manifest,
        logger: Option<&Logger<'_>>,
    ) -> InstallResult<()> {
        let format = config.format.to_lowercase();
        let file_name = downloaded
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_lowercase();
        let extension = downloaded
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_lowercase();

        if matches!(format.as_str(), "7z" | "7z-sfx" | "7z.exe") {
            emit(
                logger,
                &format!(
                    "Extrayendo paquete 7z firmado en {}...",
                    target_dir.display()
                ),
            );
            let mut command = Command::new(downloaded);
            command.args(["-y", &format!("-o{}", target_dir.display())]);
            hide_console_window(&mut command);
            let output = command
                .output()
                .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
            if !output.status.success() {
                let detail = if output.stderr.is_empty() {
                    String::from_utf8_lossy(&output.stdout).trim().to_owned()
                } else {
                    String::from_utf8_lossy(&output.stderr).trim().to_owned()
                };
                return Err(format!(
                    "Error durante la extracción 7z ({}): {detail}",
                    output.status.code().unwrap_or(-1)
                )
                .into());
            }
        } else if format == "zip" || extension == "zip" {
            emit(
                logger,
                &format!("Extrayendo paquete ZIP en {}...", target_dir.display()),
            );
            self.extract_zip(downloaded, target_dir)?;
            patch_python_embed_pth(target_dir);

            if manifest.id.to_lowercase().contains("gcc")
                || manifest
                    .aliases
                    .iter()
                    .any(|alias| alias.to_lowercase().contains("gcc"))
            {
                let has_as = walkdir::WalkDir::new(target_dir)
                    .follow_links(false)
                    .into_iter()
                    .filter_map(Result::ok)
                    .any(|entry| {
                        entry
                            .file_name()
                            .to_string_lossy()
                            .eq_ignore_ascii_case("as.exe")
                    });
                if !has_as {
                    emit(
                        logger,
                        "[!] AVISO: no se encontró 'as.exe' en la carpeta de GCC.",
                    );
                    emit(
                        logger,
                        "Windows Defender puede haberlo eliminado; revisa el Historial de protección.",
                    );
                }
            }
        } else if is_tar_archive(&file_name, &extension) {
            emit(
                logger,
                &format!("Extrayendo paquete TAR en {}...", target_dir.display()),
            );
            self.extract_tar(downloaded, target_dir)?;
        } else if matches!(format.as_str(), "portable-exe" | "portable") {
            emit(logger, "Copiando aplicación portable...");
            let binary_name = config
                .binaries
                .first()
                .map(String::as_str)
                .or_else(|| downloaded.file_name().and_then(|value| value.to_str()))
                .ok_or("el archivo portable no tiene nombre")?;
            let destination = target_dir.join(safe_component(binary_name));
            fs::copy(downloaded, destination)?;
        } else if matches!(extension.as_str(), "exe" | "msi") {
            emit(logger, "Ejecutando instalador ejecutable...");
            let mut command = Command::new(downloaded);
            command.args(&config.silent_args);
            hide_console_window(&mut command);
            let output = command.output()?;
            if !output.status.success() {
                return Err(format!(
                    "Error durante la instalación ejecutable: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                )
                .into());
            }
        }
        Ok(())
    }

    fn extract_zip(&self, archive_path: &Path, target_dir: &Path) -> InstallResult<()> {
        let file = File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        for index in 0..archive.len() {
            let mut member = archive.by_index(index)?;
            let member_name = member.name().to_owned();
            let destination = safe_archive_member(target_dir, &member_name)?;

            if member
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
            {
                return Err(
                    format!("el ZIP contiene un enlace no permitido: {member_name}").into(),
                );
            }
            if member.is_dir() {
                fs::create_dir_all(destination)?;
                continue;
            }
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut output = File::create(destination)?;
            io::copy(&mut member, &mut output)?;
        }
        Ok(())
    }

    fn extract_tar(&self, archive_path: &Path, target_dir: &Path) -> InstallResult<()> {
        let file = File::open(archive_path)?;
        let reader = ArchiveReader::from_path(file, archive_path)?;
        let mut archive = Archive::new(reader);
        for entry in archive.entries()? {
            let mut entry = entry?;
            let member_name = entry.path()?.to_string_lossy().into_owned();
            let destination = safe_archive_member(target_dir, &member_name)?;
            let entry_type = entry.header().entry_type();
            if entry_type.is_symlink() || entry_type.is_hard_link() {
                return Err(
                    format!("el archivo contiene un enlace no permitido: {member_name}").into(),
                );
            }
            if entry_type.is_dir() {
                fs::create_dir_all(destination)?;
            } else if entry_type.is_file() {
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut output = File::create(destination)?;
                io::copy(&mut entry, &mut output)?;
            } else {
                return Err(format!("tipo de entrada TAR no permitido: {member_name}").into());
            }
        }
        Ok(())
    }

    fn install_command(
        &self,
        manifest: &Manifest,
        _force: bool,
        logger: Option<&Logger<'_>>,
    ) -> InstallResult<(bool, String)> {
        let Some(command_config) = manifest.install.command.as_ref() else {
            return Ok((
                false,
                format!(
                    "El paquete '{}' no tiene configurado ningún comando en 'install.command'.",
                    manifest.id
                ),
            ));
        };
        let platform = system();
        let script = command_for_platform(command_config, platform);
        let Some(script) = script else {
            return Ok((
                false,
                format!(
                    "El paquete '{}' no cuenta con comando para la plataforma '{platform}'.",
                    manifest.id
                ),
            ));
        };

        emit(
            logger,
            &format!(
                "Iniciando instalación por comando para '{}' ({platform})...",
                manifest.name
            ),
        );
        let status = if command_config.elevated {
            run_elevated(script, platform, logger)?
        } else {
            run_script(script, &command_config.shell, platform, logger)?
        };
        if !status.success() {
            return Ok((
                false,
                format!(
                    "El comando finalizó con código de error {}",
                    status.code().unwrap_or(-1)
                ),
            ));
        }

        for binary in &command_config.binaries {
            if let Some(found) = find_on_path(binary) {
                emit(
                    logger,
                    &format!("✓ Binario encontrado en el sistema: {found}"),
                );
            } else {
                emit(
                    logger,
                    &format!(
                        "ℹ️ El binario '{binary}' estará disponible tras reiniciar la sesión o la terminal."
                    ),
                );
            }
        }

        let version = manifest.version.clone().unwrap_or_else(|| "command".into());
        let state = PackageState {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: version.clone(),
            package_type: manifest.package_type.clone(),
            mode: "command".into(),
            install_path: Some("system/command".into()),
            binaries: command_config.binaries.clone(),
            shims: Vec::new(),
            installed_at: String::new(),
            manifest_hash: manifest.manifest_hash.clone().unwrap_or_default(),
        };
        register_installed(&self.paths, state)?;
        emit(
            logger,
            &format!("¡'{}' instalado con éxito!", manifest.name),
        );
        Ok((
            true,
            format!(
                "'{}' instalado correctamente mediante comandos de consola.",
                manifest.name
            ),
        ))
    }
}

fn manifest_binaries(manifest: &Manifest) -> Vec<String> {
    manifest
        .install
        .here
        .as_ref()
        .map(|here| here.binaries.clone())
        .or_else(|| {
            manifest
                .install
                .command
                .as_ref()
                .map(|command| command.binaries.clone())
        })
        .unwrap_or_default()
}

fn emit(logger: Option<&Logger<'_>>, message: &str) {
    if let Some(logger) = logger {
        logger(message);
    }
}

fn safe_component(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|character| {
            if matches!(
                character,
                '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
            ) {
                '-'
            } else {
                character
            }
        })
        .collect();
    if sanitized.is_empty() || sanitized == "." || sanitized == ".." {
        "unnamed".into()
    } else {
        sanitized
    }
}

fn safe_archive_member(target_dir: &Path, member_name: &str) -> io::Result<PathBuf> {
    let path = Path::new(member_name);
    if path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("entrada de archivo fuera del destino: {member_name}"),
        ));
    }

    let mut relative = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("entrada de archivo fuera del destino: {member_name}"),
                ));
            }
            Component::CurDir => {}
            Component::Normal(value) => relative.push(value),
        }
    }

    let destination = target_dir.join(relative);
    let root = target_dir
        .canonicalize()
        .unwrap_or_else(|_| target_dir.to_path_buf());
    let parent = destination.parent().unwrap_or(target_dir).to_path_buf();
    let canonical_parent = parent.canonicalize().unwrap_or(parent);
    if !canonical_parent.starts_with(&root) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("entrada de archivo fuera del destino: {member_name}"),
        ));
    }
    Ok(destination)
}

fn is_tar_archive(file_name: &str, extension: &str) -> bool {
    extension == "tar"
        || matches!(extension, "gz" | "xz" | "tgz" | "bz2")
        || file_name.contains(".tar.")
}

enum ArchiveReader {
    Plain(File),
    Gzip(GzDecoder<File>),
    Xz(XzDecoder<File>),
    Bzip(BzDecoder<File>),
}

impl ArchiveReader {
    fn from_path(file: File, archive_path: &Path) -> io::Result<Self> {
        let name = archive_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_lowercase();
        if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            Ok(Self::Gzip(GzDecoder::new(file)))
        } else if name.ends_with(".tar.xz") || name.ends_with(".txz") {
            Ok(Self::Xz(XzDecoder::new(file)))
        } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
            Ok(Self::Bzip(BzDecoder::new(file)))
        } else {
            Ok(Self::Plain(file))
        }
    }
}

impl Read for ArchiveReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Plain(reader) => reader.read(buffer),
            Self::Gzip(reader) => reader.read(buffer),
            Self::Xz(reader) => reader.read(buffer),
            Self::Bzip(reader) => reader.read(buffer),
        }
    }
}

fn patch_python_embed_pth(target_dir: &Path) {
    let Ok(entries) = fs::read_dir(target_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("_pth") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        if content.contains("#import site") {
            let _ = fs::write(path, content.replace("#import site", "import site"));
        }
    }
}

fn command_for_platform<'a>(config: &'a CommandConfig, platform: &str) -> Option<&'a str> {
    match platform {
        "windows" => config.windows.as_deref().or(config.script.as_deref()),
        "macos" => config.darwin.as_deref().or(config.script.as_deref()),
        _ => config.linux.as_deref().or(config.script.as_deref()),
    }
}

fn run_script(
    script: &str,
    shell: &str,
    platform: &str,
    logger: Option<&Logger<'_>>,
) -> InstallResult<std::process::ExitStatus> {
    let mut command = if platform == "windows" {
        if shell.eq_ignore_ascii_case("cmd") {
            let mut command = Command::new("cmd.exe");
            command.args(["/c", script]);
            command
        } else {
            let mut command = Command::new("powershell.exe");
            command.args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                script,
            ]);
            command
        }
    } else {
        let mut command = Command::new("/bin/bash");
        command.args(["-c", script]);
        command
    };
    run_streaming(&mut command, logger)
}

fn run_elevated(
    script: &str,
    platform: &str,
    logger: Option<&Logger<'_>>,
) -> InstallResult<std::process::ExitStatus> {
    if platform == "windows" {
        let script_path = temp_script_path("packwire-elevated", "ps1");
        fs::write(&script_path, script)?;
        let escaped = script_path.to_string_lossy().replace('\'', "''");
        let command_line = format!(
            "Start-Process powershell.exe -Verb RunAs -Wait -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','{escaped}'"
        );
        let mut command = Command::new("powershell.exe");
        command.args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &command_line,
        ]);
        hide_console_window(&mut command);
        let result = command.status();
        let _ = fs::remove_file(script_path);
        return Ok(result?);
    }

    if platform == "macos" {
        let escaped = script.replace('\\', "\\\\").replace('"', "\\\"");
        let mut command = Command::new("osascript");
        command.args([
            "-e",
            &format!("do shell script \"{escaped}\" with administrator privileges"),
        ]);
        return Ok(run_streaming(&mut command, logger)?);
    }

    let is_root = Command::new("id")
        .args(["-u"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim() == "0")
        .unwrap_or(false);
    let mut command = if is_root {
        let mut command = Command::new("/bin/bash");
        command.args(["-c", script]);
        command
    } else if command_exists("pkexec") && std::env::var_os("DISPLAY").is_some() {
        let mut command = Command::new("pkexec");
        command.args(["/bin/bash", "-c", script]);
        command
    } else {
        let mut command = Command::new("sudo");
        command.args(["/bin/bash", "-c", script]);
        command
    };
    run_streaming(&mut command, logger)
}

fn run_streaming(
    command: &mut Command,
    logger: Option<&Logger<'_>>,
) -> InstallResult<std::process::ExitStatus> {
    hide_console_window(command);
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let status = std::thread::scope(|scope| -> InstallResult<std::process::ExitStatus> {
        let stderr_reader = stderr.map(|stderr| {
            scope.spawn(|| {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    if !line.trim().is_empty() {
                        emit(logger, &line);
                    }
                }
            })
        });
        if let Some(stdout) = stdout {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if !line.trim().is_empty() {
                    emit(logger, &line);
                }
            }
        }
        let status = child.wait()?;
        if let Some(reader) = stderr_reader {
            reader
                .join()
                .map_err(|_| io::Error::other("falló el lector de salida del comando"))?;
        }
        Ok(status)
    })?;
    Ok(status)
}

fn command_exists(command: &str) -> bool {
    let lookup = if cfg!(windows) { "where.exe" } else { "which" };
    let mut process = Command::new(lookup);
    process.arg(command);
    hide_console_window(&mut process);
    process
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn find_on_path(binary: &str) -> Option<String> {
    let lookup = if cfg!(windows) { "where.exe" } else { "which" };
    let candidate = Path::new(binary)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(binary);
    let mut process = Command::new(lookup);
    process.arg(candidate);
    hide_console_window(&mut process);
    let output = process.output().ok()?;
    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_owned()
    })
}

fn open_url(url: &str) -> InstallResult<()> {
    let status = if cfg!(windows) {
        let mut command = Command::new("cmd.exe");
        command.args(["/c", "start", "", url]);
        hide_console_window(&mut command);
        command.status()?
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

/// Evita que los procesos auxiliares de instalación abran una ventana negra
/// cuando Packwire se ejecuta como aplicación GUI en Windows.
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

fn temp_script_path(prefix: &str, extension: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!("{prefix}-{timestamp}.{extension}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use zip::write::{SimpleFileOptions, ZipWriter};

    #[test]
    fn rejects_archive_path_traversal() {
        let root = std::env::temp_dir().join("packwire-installer-test");
        assert!(safe_archive_member(&root, "../../evil.exe").is_err());
        assert!(safe_archive_member(&root, "C:\\evil.exe").is_err());
        assert!(safe_archive_member(&root, "bin/tool.exe").is_ok());
    }

    #[test]
    fn zip_extraction_rejects_zip_slip_before_writing_outside_target() {
        let root = std::env::temp_dir().join(format!(
            "packwire-installer-zip-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let target = root.join("target");
        fs::create_dir_all(&target).unwrap();
        let archive_path = root.join("payload.zip");
        let file = File::create(&archive_path).unwrap();
        let mut archive = ZipWriter::new(file);
        archive
            .start_file("../../evil.exe", SimpleFileOptions::default())
            .unwrap();
        archive.write_all(b"blocked").unwrap();
        archive.finish().unwrap();

        let installer = Installer::new(Paths::from_root(root.clone())).unwrap();
        assert!(installer.extract_zip(&archive_path, &target).is_err());
        assert!(!root.parent().unwrap().join("evil.exe").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn selects_platform_command_and_keeps_safe_paths() {
        let config = CommandConfig {
            script: Some("fallback".into()),
            windows: Some("windows-command".into()),
            linux: Some("linux-command".into()),
            darwin: None,
            shell: "bash".into(),
            elevated: false,
            binaries: Vec::new(),
        };
        assert_eq!(
            command_for_platform(&config, "windows"),
            Some("windows-command")
        );
        assert_eq!(
            command_for_platform(&config, "linux"),
            Some("linux-command")
        );
        assert_eq!(safe_component(r#"python:3.14/zip"#), "python-3.14-zip");
    }

    #[test]
    fn install_config_os_override_is_compatible_with_manifest_shape() {
        let manifest = Manifest::from_value(&json!({
            "name": "tool",
            "type": "stable",
            "install": {
                "mode": "here",
                "os": {"linux": {"mode": "command", "command": "echo ok"}}
            }
        }))
        .unwrap();
        assert_eq!(manifest.install.os.unwrap()["linux"]["mode"], "command");
    }
}
