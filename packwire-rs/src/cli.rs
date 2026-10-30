//! Interfaz de línea de comandos mínima para el ejecutable nativo Packwire.

use std::io;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};

use crate::config::{paths, Paths};
use crate::installer::{InstallResult, Installer};
use crate::manifest::{sync_remote_catalog, ManifestLoader};
use crate::models::Manifest;
use crate::pather::Pather;
use crate::resolver::{resolve_manifest, Resolver};
use crate::state::load_state;

#[derive(Debug, Parser)]
#[command(
    name = "packwire",
    version,
    about = "Packwire: Gestor y descargador moderno de paquetes para Windows"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Instala un paquete desde el catálogo de manifests.
    Install {
        /// Nombre o identificador, por ejemplo python, python@stable o python@3.14.
        package: String,
        /// Versión específica a instalar.
        #[arg(short = 'v', long)]
        version: Option<String>,
        /// Canal del paquete.
        #[arg(long, value_enum, default_value_t = Channel::Stable)]
        channel: Channel,
        /// Modo de instalación.
        #[arg(short = 'm', long, value_enum, default_value_t = InstallMode::Here)]
        mode: InstallMode,
        /// Fuerza la reinstalación o ignora colisiones de shims.
        #[arg(short, long)]
        force: bool,
    },
    /// Lista los paquetes instalados actualmente.
    List,
    /// Lista los manifests disponibles en el catálogo.
    Available,
    /// Muestra el manifest de un paquete.
    Info {
        /// Nombre, alias o identificador del paquete.
        package: String,
    },
    /// Comprueba o modifica el PATH de los shims.
    Path {
        /// Añade el directorio de shims al PATH del usuario.
        #[arg(long)]
        add: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Channel {
    Stable,
    Fixed,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum InstallMode {
    Here,
    Site,
    Command,
}

impl InstallMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Here => "here",
            Self::Site => "site",
            Self::Command => "command",
        }
    }
}

/// Dependencias de la CLI, separadas para que las pruebas no dependan de
/// variables globales ni de una instalación previa.
pub struct CliContext {
    pub paths: Paths,
    loader: ManifestLoader,
    resolver: Resolver,
    installer: Installer,
}

impl CliContext {
    pub fn new() -> InstallResult<Self> {
        let paths = paths();
        let sync = sync_remote_catalog(&paths.manifests_dir);
        if sync.updated {
            println!("[Packwire] {}", sync.message);
        }
        Self::with_paths(paths)
    }

    pub fn with_paths(paths: Paths) -> InstallResult<Self> {
        let loader = ManifestLoader::new(paths.manifests_dir.clone());
        let resolver = Resolver::new()?;
        let installer = Installer::new(paths.clone())?;
        Ok(Self {
            paths,
            loader,
            resolver,
            installer,
        })
    }

    pub fn execute<I, T>(&self, args: I) -> i32
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let cli = match Cli::try_parse_from(args) {
            Ok(cli) => cli,
            Err(error) => {
                let code = error.exit_code();
                let _ = error.print();
                return code;
            }
        };

        let Some(command) = cli.command else {
            let mut command = Cli::command();
            let _ = command.print_help();
            println!();
            return 0;
        };

        match self.execute_command(command) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("[-] {error}");
                1
            }
        }
    }

    fn execute_command(&self, command: Commands) -> InstallResult<()> {
        match command {
            Commands::Install {
                package,
                version,
                channel,
                mode,
                force,
            } => self.install(&package, version.as_deref(), channel, mode, force),
            Commands::List => self.list(),
            Commands::Available => self.available(),
            Commands::Info { package } => self.info(&package),
            Commands::Path { add } => self.path(add),
        }
    }

    fn install(
        &self,
        package: &str,
        version: Option<&str>,
        channel: Channel,
        mode: InstallMode,
        force: bool,
    ) -> InstallResult<()> {
        println!("[*] Iniciando instalación de '{package}'...");
        let mut manifest = self.resolve_target(package, version, channel)?;
        let logger = |message: &str| println!("[Packwire] {message}");
        let (success, message) =
            self.installer
                .install(&mut manifest, Some(mode.as_str()), force, Some(&logger))?;
        if success {
            println!("[+] Éxito: {message}");
            Ok(())
        } else {
            Err(io::Error::other(message).into())
        }
    }

    fn resolve_target(
        &self,
        package: &str,
        version: Option<&str>,
        channel: Channel,
    ) -> InstallResult<Manifest> {
        if matches!(channel, Channel::Fixed) && version.is_none() && !package.contains('@') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "el canal 'fixed' requiere --version o un identificador como paquete@versión",
            )
            .into());
        }
        let mut manifest = if let Some(version) = version {
            let base = package.split('@').next().unwrap_or(package);
            self.loader
                .find_manifest(&format!("{base}@{version}"))
                .or_else(|| self.loader.find_manifest(package))
        } else if !package.contains('@') && matches!(channel, Channel::Stable) {
            self.loader
                .find_manifest(&format!("{package}@stable"))
                .or_else(|| self.loader.find_manifest(package))
        } else {
            self.loader.find_manifest(package)
        }
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("No se encontró ningún manifest para '{package}'."),
            )
        })?;

        if let Some(version) = version {
            manifest.version = Some(version.to_owned());
            manifest.package_type = "fixed".into();
            manifest.id = format!(
                "{}@{}",
                package.split('@').next().unwrap_or(package).to_lowercase(),
                version
            );
            resolve_manifest(&self.resolver, &mut manifest)?;
        } else if !package.contains('@') && matches!(channel, Channel::Stable) {
            manifest.package_type = "stable".into();
            manifest.id = format!("{}@stable", package.to_lowercase());
            resolve_manifest(&self.resolver, &mut manifest)?;
        }
        Ok(manifest)
    }

    fn list(&self) -> InstallResult<()> {
        let mut installed: Vec<_> = load_state(&self.paths).into_values().collect();
        installed.sort_by(|left, right| left.id.cmp(&right.id));
        if installed.is_empty() {
            println!("[*] No hay paquetes instalados actualmente en Packwire.");
            return Ok(());
        }

        println!(
            "{:<20} {:<12} {:<10} {:<8} {:<8} RUTA",
            "ID", "NOMBRE", "VERSIÓN", "TIPO", "MODO"
        );
        println!("{}", "-".repeat(80));
        for package in installed {
            let path = package
                .install_path
                .as_deref()
                .unwrap_or("(Navegador/Site)");
            println!(
                "{:<20} {:<12} {:<10} {:<8} {:<8} {}",
                package.id, package.name, package.version, package.package_type, package.mode, path
            );
        }
        Ok(())
    }

    fn available(&self) -> InstallResult<()> {
        let mut manifests = self.loader.list_available_manifests();
        manifests.sort_by(|left, right| left.id.cmp(&right.id));
        println!(
            "{:<20} {:<12} {:<8} {:<22} DESCRIPCIÓN",
            "ID", "NOMBRE", "TIPO", "CATEGORÍA"
        );
        println!("{}", "-".repeat(80));
        for manifest in manifests {
            println!(
                "{:<20} {:<12} {:<8} {:<22} {}",
                manifest.id,
                manifest.name,
                manifest.package_type,
                manifest.category,
                manifest.description
            );
        }
        Ok(())
    }

    fn info(&self, package: &str) -> InstallResult<()> {
        let manifest = self.loader.find_manifest(package).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("No se encontró información para '{package}'."),
            )
        })?;
        println!("{}", serde_json::to_string_pretty(&manifest.to_value())?);
        Ok(())
    }

    fn path(&self, add: bool) -> InstallResult<()> {
        let pather = Pather::new(self.paths.shims_dir.clone())?;
        if add {
            if pather.add_shims_to_user_path()? {
                println!("[+] Directorio de shims añadido al PATH del usuario.");
            } else {
                println!("[*] El directorio de shims ya está en el PATH del usuario.");
            }
        } else {
            println!("Directorio de shims: {}", self.paths.shims_dir.display());
            println!(
                "¿Está en PATH? {}",
                if pather.is_shims_in_path() {
                    "SÍ"
                } else {
                    "NO (usa --add para agregarlo)"
                }
            );
        }
        Ok(())
    }
}

pub fn run() -> i32 {
    match CliContext::new() {
        Ok(context) => context.execute(std::env::args_os()),
        Err(error) => {
            eprintln!("[-] No se pudo iniciar Packwire: {error}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_paths() -> Paths {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Paths::from_root(std::env::temp_dir().join(format!("packwire-cli-test-{suffix}")))
    }

    #[test]
    fn parses_the_minimal_cli_surface() {
        let cli = Cli::try_parse_from([
            "packwire",
            "install",
            "python",
            "--version",
            "3.14",
            "--mode",
            "here",
            "--force",
        ])
        .unwrap();
        assert!(matches!(
            cli.command,
            Some(Commands::Install { force: true, .. })
        ));
        assert!(Cli::try_parse_from(["packwire", "ui"]).is_err());
        assert!(Cli::try_parse_from(["packwire", "remove", "python"]).is_err());
    }

    #[test]
    fn lists_available_manifests_without_network() {
        let paths = test_paths();
        let context = CliContext::with_paths(paths.clone()).unwrap();
        assert!(context.loader.list_available_manifests().len() >= 44);
        let _ = fs::remove_dir_all(paths.packwire_root);
    }

    #[test]
    fn list_and_info_commands_use_the_rust_state_and_manifest_formats() {
        let paths = test_paths();
        let context = CliContext::with_paths(paths.clone()).unwrap();
        assert_eq!(context.execute(["packwire", "list"]), 0);
        assert_eq!(context.execute(["packwire", "info", "python"]), 0);
        fs::remove_dir_all(paths.packwire_root).unwrap();
    }
}
