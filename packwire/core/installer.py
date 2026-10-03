import zipfile
import subprocess
import webbrowser
import shutil
import uuid
from pathlib import Path
from typing import Optional, Tuple, List, Callable

from packwire.core.config import APPS_DIR, CACHE_DIR, SYSTEM, ensure_directories
from packwire.core.models import Manifest, PackageState, InstallConfig
from packwire.core.state import check_collision, register_installed, get_installed
from packwire.core.visitors.downloader import DownloaderVisitor
from packwire.core.visitors.pather import PatherVisitor
from packwire.core.visitors.process_manifest import ManifestVisitor


class Installer:
    """Orchestrates package installations in 'here', 'command', and 'site' modes."""

    def __init__(
        self,
        downloader: Optional[DownloaderVisitor] = None,
        pather: Optional[PatherVisitor] = None,
        manifest_visitor: Optional[ManifestVisitor] = None
    ):
        self.downloader = downloader or DownloaderVisitor()
        self.pather = pather or PatherVisitor()
        self.manifest_visitor = manifest_visitor or ManifestVisitor()
        ensure_directories()

    def install(
        self,
        manifest: Manifest,
        mode_override: Optional[str] = None,
        force: bool = False,
        logger: Optional[Callable[[str], None]] = None
    ) -> Tuple[bool, str]:
        """
        Execute installation based on the manifest.
        mode_override: can override the default mode ('here', 'command', 'site')
        """
        # Resolve OS-specific overrides if configured (e.g. multi-os)
        if manifest.install.os and isinstance(manifest.install.os, dict):
            os_data = manifest.install.os.get(SYSTEM)
            if os_data:
                override_cfg = InstallConfig.from_dict(os_data)
                if override_cfg.mode:
                    manifest.install.mode = override_cfg.mode
                if override_cfg.here:
                    manifest.install.here = override_cfg.here
                if override_cfg.command:
                    manifest.install.command = override_cfg.command
                if override_cfg.site:
                    manifest.install.site = override_cfg.site

        mode = (mode_override or manifest.install.mode or "here").lower()

        # Check collision and duplicate
        binaries = []
        if manifest.install.here:
            binaries = manifest.install.here.binaries
        elif manifest.install.command:
            binaries = manifest.install.command.binaries

        is_dup, dup_msg, collisions = check_collision(manifest.id, binaries)

        if is_dup and not force:
            return False, dup_msg

        if collisions and not force:
            warn_msg = f"Advertencia: colisión detectada en binarios ({', '.join(collisions)})."
            print(f"[Packwire Warning] {warn_msg}")
            if logger:
                logger(warn_msg)

        if mode == "site":
            return self._install_site(manifest, logger=logger)
        elif mode == "here":
            return self._install_here(manifest, collisions, force, logger=logger)
        elif mode in ("command", "console", "script"):
            return self._install_command(manifest, force, logger=logger)
        else:
            return False, f"Modo de instalación no reconocido: '{mode}'"

    def _install_site(
        self,
        manifest: Manifest,
        logger: Optional[Callable[[str], None]] = None
    ) -> Tuple[bool, str]:
        """Handle 'site' installation mode (redirect to official site)."""
        if not manifest.install.site or not manifest.install.site.url:
            return False, f"El paquete '{manifest.id}' no tiene configurada una URL en 'site'."

        url = manifest.install.site.url
        msg = f"Redirigiendo al sitio oficial para instalación manual: {url}"
        print(f"[Packwire] {msg}")
        if logger:
            logger(msg)
        webbrowser.open(url)

        # Register state
        state = PackageState(
            id=manifest.id,
            name=manifest.name,
            version=manifest.version or "site-redirect",
            type=manifest.type,
            mode="site",
            install_path=None,
            binaries=[],
            shims=[],
            manifest_hash=manifest.manifest_hash or ""
        )
        register_installed(state)

        return True, f"Navegador abierto en {url}. Instalación manual iniciada."

    def _install_here(
        self,
        manifest: Manifest,
        collisions: List[str],
        force: bool,
        logger: Optional[Callable[[str], None]] = None
    ) -> Tuple[bool, str]:
        """Handle 'here' installation mode (automated download and setup)."""
        def log(msg: str):
            print(f"[Packwire] {msg}")
            if logger:
                logger(msg)

        here_cfg = manifest.install.here
        if not here_cfg or not here_cfg.url:
            return False, f"El paquete '{manifest.id}' no tiene una URL configurada para el modo 'here'."

        # Target directory: APPS_DIR / <name> / <version>
        ver_slug = manifest.version or "latest"
        target_dir = APPS_DIR / manifest.name.lower() / ver_slug
        target_dir.mkdir(parents=True, exist_ok=True)

        log(f"Instalando '{manifest.name}' ({ver_slug}) en: {target_dir}")

        # 1. Download
        download_url = here_cfg.url
        log(f"Iniciando descarga desde: {download_url}")
        downloaded_file = self.downloader.download(download_url, force=force)
        log(f"Descarga completada: {downloaded_file.name}")

        # 2. Extract or run
        suffix = downloaded_file.suffix.lower()
        is_zip = here_cfg.format.lower() == "zip" or suffix == ".zip"
        is_tar = suffix in [".tar", ".gz", ".xz", ".tgz", ".bz2"] or ".tar." in downloaded_file.name.lower()

        if is_zip:
            log(f"Extrayendo paquete ZIP en {target_dir.name}...")
            with zipfile.ZipFile(downloaded_file, "r") as zip_ref:
                zip_ref.extractall(target_dir)

            # Special hook for Python embeddable zip: enable site-packages
            self._patch_python_embed_pth(target_dir)
        elif is_tar:
            log(f"Extrayendo paquete TAR/XZ en {target_dir.name}...")
            import tarfile
            with tarfile.open(downloaded_file, "r:*") as tar_ref:
                tar_ref.extractall(target_dir)
        elif suffix in [".exe", ".msi"]:
            log(f"Ejecutando instalador ejecutable...")
            args = [str(downloaded_file)] + here_cfg.silent_args
            res = subprocess.run(args, capture_output=True, text=True)
            if res.returncode != 0:
                return False, f"Error durante la instalación ejecutable: {res.stderr}"

        # 3. Create shims
        created_shims = []
        if here_cfg.addpath:
            is_default = (len(collisions) == 0) or force
            log(f"Creando shims para binarios: {', '.join(here_cfg.binaries)}...")
            created_shims = self.pather.create_shims(
                target_dir=target_dir,
                binaries=here_cfg.binaries,
                version_suffix=manifest.version,
                is_default=is_default
            )

            # Ensure shims dir is in User PATH
            self.pather.add_shims_to_user_path()

        # 4. Clean cache if requested
        if manifest.install.clean and downloaded_file.exists():
            downloaded_file.unlink(missing_ok=True)
            log(f"Archivo temporal limpio: {downloaded_file.name}")

        # 5. Register state
        state = PackageState(
            id=manifest.id,
            name=manifest.name,
            version=ver_slug,
            type=manifest.type,
            mode="here",
            install_path=str(target_dir),
            binaries=here_cfg.binaries,
            shims=created_shims,
            manifest_hash=manifest.manifest_hash or ""
        )
        register_installed(state)

        log(f"¡'{manifest.name}' ({ver_slug}) instalado con éxito!")
        return True, f"'{manifest.name}' ({ver_slug}) instalado con éxito en modo 'here'."

    def _install_command(
        self,
        manifest: Manifest,
        force: bool,
        logger: Optional[Callable[[str], None]] = None
    ) -> Tuple[bool, str]:
        """Handle 'command' / console installation mode with streaming and UAC elevation."""
        def log(msg: str):
            print(f"[Packwire Command] {msg}")
            if logger:
                logger(msg)

        cmd_cfg = manifest.install.command
        if not cmd_cfg:
            return False, f"El paquete '{manifest.id}' no tiene configurado ningún comando en 'install.command'."

        # 1. Resolve script for current platform
        script = None
        if SYSTEM == "windows":
            script = cmd_cfg.windows or cmd_cfg.script
        elif SYSTEM == "darwin":
            script = cmd_cfg.darwin or cmd_cfg.script
        else:
            script = cmd_cfg.linux or cmd_cfg.script

        if not script:
            return False, f"El paquete '{manifest.id}' no cuenta con comando para la plataforma '{SYSTEM}'."

        log(f"Iniciando instalación por comando para '{manifest.name}' ({SYSTEM})...")

        # 2. Check elevation
        if cmd_cfg.elevated:
            if SYSTEM == "windows":
                log("⚠️ Este paquete requiere permisos de Administrador. Solicitando elevación UAC en Windows...")
                temp_script = CACHE_DIR / f"cmd_{manifest.name.lower().replace('/', '_')}_{uuid.uuid4().hex[:6]}.ps1"
                temp_log = CACHE_DIR / f"log_{manifest.name.lower().replace('/', '_')}_{uuid.uuid4().hex[:6]}.txt"

                wrapped_script = f"""
$ErrorActionPreference = 'Stop'
try {{
    Start-Transcript -Path '{temp_log}' -Append
    {script}
    Stop-Transcript
}} catch {{
    Add-Content -Path '{temp_log}' -Value "ERROR: $_"
    exit 1
}}
"""
                temp_script.write_text(wrapped_script, encoding="utf-8")

                uac_ps = [
                    "powershell.exe",
                    "-NoProfile",
                    "-ExecutionPolicy", "Bypass",
                    "-Command",
                    f'Start-Process powershell -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File \\"{temp_script}\\"" -Verb RunAs -Wait'
                ]
                try:
                    log("Esperando confirmación en el cuadro de diálogo UAC...")
                    proc = subprocess.run(uac_ps, capture_output=True, text=True)
                    if temp_log.exists():
                        log_content = temp_log.read_text(encoding="utf-8", errors="replace")
                        for line in log_content.splitlines():
                            if line.strip():
                                log(line)
                        temp_log.unlink(missing_ok=True)
                    temp_script.unlink(missing_ok=True)

                    if proc.returncode != 0:
                        return False, "El comando elevado finalizó con error o la elevación fue cancelada por el usuario."
                    log("Comando elevado completado exitosamente.")
                except Exception as e:
                    temp_script.unlink(missing_ok=True)
                    temp_log.unlink(missing_ok=True)
                    return False, f"Fallo al invocar UAC: {e}"

            elif SYSTEM == "darwin":
                log("⚠️ Este paquete requiere permisos de Administrador. Solicitando autorización en macOS...")
                escaped_script = script.replace("\\", "\\\\").replace('"', '\\"')
                osa_cmd = [
                    "osascript",
                    "-e",
                    f'do shell script "{escaped_script}" with administrator privileges'
                ]
                try:
                    proc = subprocess.run(osa_cmd, capture_output=True, text=True)
                    if proc.returncode != 0:
                        return False, f"Fallo en la autorización o comando de macOS: {proc.stderr}"
                    log("Comando completado exitosamente con privilegios en macOS.")
                except Exception as e:
                    return False, f"Fallo al invocar autorización en macOS: {e}"

            elif SYSTEM == "linux":
                log("⚠️ Este paquete requiere permisos de Administrador (root) en Linux...")
                import os
                is_root = (os.geteuid() == 0) if hasattr(os, "geteuid") else False
                if is_root:
                    args = ["/bin/bash", "-c", script]
                elif shutil.which("pkexec") and os.environ.get("DISPLAY"):
                    args = ["pkexec", "/bin/bash", "-c", script]
                else:
                    args = ["sudo", "/bin/bash", "-c", script]

                try:
                    proc = subprocess.Popen(
                        args,
                        stdout=subprocess.PIPE,
                        stderr=subprocess.STDOUT,
                        text=True,
                        bufsize=1,
                        universal_newlines=True
                    )
                    for line in proc.stdout:
                        clean = line.rstrip()
                        if clean:
                            log(clean)
                    proc.wait()
                    if proc.returncode != 0:
                        return False, f"El comando elevado finalizó con código de error {proc.returncode}"
                    log("Comando elevado completado exitosamente en Linux.")
                except Exception as e:
                    return False, f"Error al ejecutar comando elevado en Linux: {e}"
        else:
            # Standard streaming execution
            shell_name = cmd_cfg.shell or ("powershell" if SYSTEM == "windows" else "bash")
            log(f"Ejecutando script de consola con {shell_name}...")
            if SYSTEM == "windows":
                if shell_name == "cmd":
                    args = ["cmd.exe", "/c", script]
                else:
                    args = ["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script]
            else:
                args = ["/bin/bash", "-c", script]

            try:
                proc = subprocess.Popen(
                    args,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    text=True,
                    bufsize=1,
                    universal_newlines=True
                )
                for line in proc.stdout:
                    clean = line.rstrip()
                    if clean:
                        log(clean)
                proc.wait()
                if proc.returncode != 0:
                    return False, f"El comando finalizó con código de error {proc.returncode}"
            except Exception as e:
                return False, f"Error al ejecutar comando: {e}"

        # 3. Verify resulting binaries
        binaries = cmd_cfg.binaries or []
        created_shims = []
        if binaries:
            log(f"Verificando disponibilidad de binarios: {', '.join(binaries)}...")
            for b in binaries:
                found = shutil.which(b) or shutil.which(Path(b).stem)
                if found:
                    log(f"✓ Binario encontrado en el sistema: {found}")
                else:
                    log(f"ℹ️ El binario '{b}' estará disponible tras reiniciar la sesión o la terminal.")

        ver_slug = manifest.version or "command"
        state = PackageState(
            id=manifest.id,
            name=manifest.name,
            version=ver_slug,
            type=manifest.type,
            mode="command",
            install_path="system/command",
            binaries=binaries,
            shims=created_shims,
            manifest_hash=manifest.manifest_hash or ""
        )
        register_installed(state)

        log(f"¡'{manifest.name}' instalado con éxito!")
        return True, f"'{manifest.name}' instalado correctamente mediante comandos de consola."

    @staticmethod
    def _patch_python_embed_pth(target_dir: Path):
        """
        Uncomment 'import site' in python*._pth files in embeddable Python distribution.
        This enables pip and site-packages compatibility.
        """
        for pth_file in target_dir.glob("*._pth"):
            try:
                content = pth_file.read_text(encoding="utf-8")
                if "#import site" in content:
                    content = content.replace("#import site", "import site")
                    pth_file.write_text(content, encoding="utf-8")
            except Exception:
                pass
