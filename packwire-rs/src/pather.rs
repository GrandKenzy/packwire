//! Creación de shims y configuración del PATH.

use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use walkdir::WalkDir;

#[cfg(windows)]
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
#[cfg(windows)]
use winreg::RegKey;

pub type PatherResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone)]
pub struct Pather {
    pub shims_dir: PathBuf,
}

impl Pather {
    pub fn new(shims_dir: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&shims_dir)?;
        Ok(Self { shims_dir })
    }

    pub fn create_shims(
        &self,
        target_dir: &Path,
        binaries: &[String],
        version_suffix: Option<&str>,
        is_default: bool,
    ) -> PatherResult<Vec<String>> {
        let mut created = Vec::new();

        for binary in binaries {
            let target =
                find_target_binary(target_dir, binary).unwrap_or_else(|| target_dir.join(binary));
            let base_name = file_stem(binary).to_lowercase();
            let mut names = Vec::new();

            if is_default {
                names.push(base_name.clone());
            }
            if let Some(version) = version_suffix {
                let clean_version: String = version
                    .chars()
                    .filter(|character| *character != '.' && *character != '-')
                    .collect();
                names.push(format!("{base_name}{clean_version}"));
                names.push(format!("{base_name}-{version}"));
            }

            for name in names {
                validate_shim_name(&name)?;
                let target = target.to_string_lossy().replace('"', "\\\"");

                if cfg!(windows) {
                    let cmd_name = format!("{name}.cmd");
                    let ps1_name = format!("{name}.ps1");
                    write_file(
                        &self.shims_dir.join(&cmd_name),
                        format!("@echo off\r\n\"{target}\" %*\r\n").as_bytes(),
                    )?;
                    write_file(
                        &self.shims_dir.join(&ps1_name),
                        format!("& \"{target}\" $args\r\n").as_bytes(),
                    )?;
                    created.push(cmd_name);
                } else {
                    let shim_path = self.shims_dir.join(&name);
                    write_file(
                        &shim_path,
                        format!("#!/bin/sh\nexec \"{target}\" \"$@\"\n").as_bytes(),
                    )?;
                    make_executable(&shim_path)?;
                    created.push(name);
                }
            }
        }

        created.sort();
        created.dedup();
        Ok(created)
    }

    pub fn remove_shims(&self, shim_names: &[String]) -> PatherResult<()> {
        for name in shim_names {
            let base = file_stem(name).to_owned();
            let candidates = [
                name.clone(),
                base.clone(),
                format!("{base}.cmd"),
                format!("{base}.ps1"),
                format!("{base}.bat"),
            ];

            for candidate in candidates {
                if validate_shim_name(&candidate).is_err() {
                    continue;
                }
                let path = self.shims_dir.join(candidate);
                if path.is_file() {
                    fs::remove_file(path)?;
                }
            }
        }
        Ok(())
    }

    pub fn is_shims_in_path(&self) -> bool {
        let normalized = normalize_path(&self.shims_dir);

        if cfg!(windows) {
            #[cfg(windows)]
            if let Some(user_path) = read_user_path() {
                if split_windows_path(&user_path)
                    .iter()
                    .any(|path| normalize_path(Path::new(path)) == normalized)
                {
                    return true;
                }
            }
            split_process_path(';')
                .iter()
                .any(|path| normalize_path(Path::new(path)) == normalized)
        } else {
            profile_files().iter().any(|file| {
                fs::read_to_string(file)
                    .map(|content| content.to_lowercase().contains(&normalized.to_lowercase()))
                    .unwrap_or(false)
            })
        }
    }

    pub fn add_shims_to_user_path(&self) -> PatherResult<bool> {
        self.add_directory_to_user_path_impl(&self.shims_dir, true)
    }

    pub fn add_directory_to_user_path(&self, target_dir: &Path) -> PatherResult<bool> {
        self.add_directory_to_user_path_impl(target_dir, false)
    }

    pub fn remove_shims_from_user_path(&self) -> PatherResult<bool> {
        let normalized = normalize_path(&self.shims_dir);
        let mut changed = false;

        if cfg!(windows) {
            #[cfg(windows)]
            if let Some(user_path) = read_user_path() {
                let original = split_windows_path(&user_path);
                let filtered: Vec<&str> = original
                    .iter()
                    .copied()
                    .filter(|path| normalize_path(Path::new(path)) != normalized)
                    .collect();
                if filtered.len() != original.len() {
                    write_user_path(&filtered.join(";"))?;
                    broadcast_environment_change();
                    changed = true;
                }
            }
        } else {
            for profile in profile_files() {
                if !profile.exists() {
                    continue;
                }
                let content = fs::read_to_string(&profile)?;
                let filtered: Vec<&str> = content
                    .lines()
                    .filter(|line| !line.contains(self.shims_dir.to_string_lossy().as_ref()))
                    .collect();
                if filtered.len() != content.lines().count() {
                    fs::write(&profile, format!("{}\n", filtered.join("\n")))?;
                    changed = true;
                }
            }
        }

        let current = split_process_path(if cfg!(windows) { ';' } else { ':' });
        let filtered: Vec<&str> = current
            .iter()
            .map(String::as_str)
            .filter(|path| normalize_path(Path::new(path)) != normalized)
            .collect();
        env::set_var("PATH", filtered.join(if cfg!(windows) { ";" } else { ":" }));
        Ok(changed)
    }

    pub fn add_defender_exclusion(&self, path: Option<&Path>) -> PatherResult<(bool, String)> {
        if !cfg!(windows) {
            return Ok((
                false,
                "Windows Defender solo está disponible en Windows.".into(),
            ));
        }

        let target = path.unwrap_or(&self.shims_dir);
        let target_text = powershell_single_quote(&target.to_string_lossy());
        let script = format!(
            "Add-MpPreference -ExclusionPath '{target_text}' -ErrorAction Stop; \
             Add-MpPreference -ExclusionProcess 'as.exe' -ErrorAction Stop; \
             Add-MpPreference -ExclusionProcess 'gcc.exe' -ErrorAction Stop"
        );

        let status = if is_windows_admin() {
            Command::new("powershell.exe")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    &script,
                ])
                .status()?
        } else {
            let escaped_script = powershell_single_quote(&script);
            let elevated = format!(
                "Start-Process powershell.exe -Verb RunAs -Wait -ArgumentList \
                 '-NoProfile','-ExecutionPolicy','Bypass','-Command','{escaped_script}'"
            );
            Command::new("powershell.exe")
                .args(["-NoProfile", "-Command", &elevated])
                .status()?
        };

        if status.success() {
            Ok((
                true,
                format!("Exclusión configurada para '{}'.", target.display()),
            ))
        } else {
            Ok((false, "No se pudo configurar Windows Defender.".into()))
        }
    }

    fn add_directory_to_user_path_impl(
        &self,
        target_dir: &Path,
        allow_posix: bool,
    ) -> PatherResult<bool> {
        let target = target_dir
            .to_string_lossy()
            .trim_end_matches(['\\', '/'])
            .to_owned();
        let normalized = normalize_path(Path::new(&target));

        if cfg!(windows) {
            #[cfg(windows)]
            {
                let current = read_user_path().unwrap_or_default();
                let mut paths = split_windows_path(&current);
                if !paths
                    .iter()
                    .any(|path| normalize_path(Path::new(path)) == normalized)
                {
                    paths.insert(0, &target);
                    write_user_path(&paths.join(";"))?;
                    broadcast_environment_change();
                }
            }
        } else if allow_posix {
            let export_line = format!("\nexport PATH=\"{target}:$PATH\"\n");
            for profile in profile_files() {
                if profile.exists() {
                    let content = fs::read_to_string(&profile)?;
                    if !content.contains(&target) {
                        let mut file = OpenOptions::new().append(true).open(profile)?;
                        file.write_all(export_line.as_bytes())?;
                    }
                }
            }
        } else {
            return Ok(false);
        }

        let current = split_process_path(if cfg!(windows) { ';' } else { ':' });
        if !current
            .iter()
            .any(|path| normalize_path(Path::new(path)) == normalized)
        {
            let separator = if cfg!(windows) { ";" } else { ":" };
            env::set_var(
                "PATH",
                format!("{target}{separator}{}", current.join(separator)),
            );
        }
        Ok(true)
    }
}

fn find_target_binary(target_dir: &Path, binary: &str) -> Option<PathBuf> {
    let mut candidates = vec![binary.to_owned()];
    if cfg!(windows) {
        if !binary.to_lowercase().ends_with(".exe") {
            candidates.push(format!("{binary}.exe"));
        }
    } else if binary.to_lowercase().ends_with(".exe") {
        candidates.insert(0, file_stem(binary).to_owned());
    } else {
        candidates.push(file_stem(binary).to_owned());
    }

    for candidate in &candidates {
        for path in [
            target_dir.join(candidate),
            target_dir.join("bin").join(candidate),
        ] {
            if path.is_file() {
                return Some(path);
            }
        }
    }

    WalkDir::new(target_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .find_map(|entry| {
            candidates
                .iter()
                .any(|candidate| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .eq_ignore_ascii_case(candidate)
                })
                .then(|| entry.path().to_path_buf())
        })
}

fn file_stem(value: &str) -> &str {
    Path::new(value)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(value)
}

fn validate_shim_name(name: &str) -> PatherResult<()> {
    let path = Path::new(name);
    if name.is_empty()
        || name == "."
        || name == ".."
        || path.file_name().and_then(|value| value.to_str()) != Some(name)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("nombre de shim no válido: {name}"),
        )
        .into());
    }
    Ok(())
}

fn write_file(path: &Path, content: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)?;
    file.write_all(content)
}

#[cfg(unix)]
fn make_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    fs::set_permissions(path, permissions)
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .to_lowercase()
}

fn split_process_path(separator: char) -> Vec<String> {
    env::var_os("PATH")
        .unwrap_or_default()
        .to_string_lossy()
        .split(separator)
        .filter(|path| !path.trim().is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn profile_files() -> Vec<PathBuf> {
    dirs::home_dir()
        .map(|home| {
            vec![
                home.join(".zshrc"),
                home.join(".bashrc"),
                home.join(".profile"),
            ]
        })
        .unwrap_or_default()
}

#[cfg(windows)]
fn split_windows_path(path: &str) -> Vec<&str> {
    path.split(';')
        .filter(|value| !value.trim().is_empty())
        .collect()
}

#[cfg(windows)]
fn read_user_path() -> Option<String> {
    let environment = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags("Environment", KEY_READ)
        .ok()?;
    environment.get_value("Path").ok()
}

#[cfg(windows)]
fn write_user_path(path: &str) -> PatherResult<()> {
    let current_user = RegKey::predef(HKEY_CURRENT_USER);
    let (environment, _) = current_user.create_subkey("Environment")?;
    environment.set_value("Path", &path)?;
    Ok(())
}

#[cfg(windows)]
fn broadcast_environment_change() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };

    let mut result = 0;
    let environment: Vec<u16> = "Environment".encode_utf16().chain(Some(0)).collect();
    unsafe {
        let _ = SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            environment.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            &mut result,
        );
    }
}

#[cfg(not(windows))]
fn broadcast_environment_change() {}

#[cfg(windows)]
fn is_windows_admin() -> bool {
    Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)",
        ])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .eq_ignore_ascii_case("True")
        })
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn is_windows_admin() -> bool {
    false
}

fn powershell_single_quote(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_paths() -> (PathBuf, PathBuf) {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        (
            env::temp_dir().join(format!("packwire-pather-target-{suffix}")),
            env::temp_dir().join(format!("packwire-pather-shims-{suffix}")),
        )
    }

    #[test]
    fn creates_and_removes_shims() {
        let (target, shims) = temporary_paths();
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("tool.exe"), b"placeholder").unwrap();
        let pather = Pather::new(shims.clone()).unwrap();

        let names = pather
            .create_shims(&target, &["tool.exe".into()], Some("1.2.3"), true)
            .unwrap();
        assert!(names
            .iter()
            .any(|name| name == "tool.cmd" || name == "tool"));
        assert!(shims.exists());

        pather.remove_shims(&names).unwrap();
        assert!(!shims
            .join(if cfg!(windows) { "tool.cmd" } else { "tool" })
            .exists());
        fs::remove_dir_all(target).unwrap();
        fs::remove_dir_all(shims).unwrap();
    }

    #[test]
    fn rejects_shim_path_traversal() {
        assert!(validate_shim_name("../evil.cmd").is_err());
        assert!(validate_shim_name("nested/evil.cmd").is_err());
    }
}
