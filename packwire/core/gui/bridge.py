import json
import subprocess
import webbrowser
from pathlib import Path
from typing import Dict, Any, List, Optional

import packwire
from packwire.core.config import SYSTEM, MACHINE, PACKWIRE_ROOT, SHIMS_DIR, APPS_DIR


class GuiBridge:
    """Bridge exposed to JavaScript inside WebView or Web API."""

    def __init__(self, progress_emitter=None):
        self.progress_emitter = progress_emitter

    def get_system_info(self) -> Dict[str, Any]:
        """Return system and configuration details."""
        in_path = packwire._pather.is_shims_in_path()
        is_win11 = False
        if SYSTEM == "windows":
            try:
                import sys
                winver = sys.getwindowsversion()
                is_win11 = (winver.major == 10 and winver.build >= 22000) or winver.major > 10
            except Exception:
                is_win11 = False
        result = {
            "os": SYSTEM,
            "arch": MACHINE,
            "shims_in_path": in_path,
            "root_dir": str(PACKWIRE_ROOT),
            "shims_dir": str(SHIMS_DIR),
            "apps_dir": str(APPS_DIR),
            "version": packwire.__version__,
            "is_windows_11": is_win11,
            "is_windows": SYSTEM == "windows"
        }
        if SYSTEM == "windows":
            result["hardware"] = self._collect_windows_diagnostics()
        return result

    @staticmethod
    def _collect_windows_diagnostics() -> Dict[str, Any]:
        """Collect useful local facts without exposing personal identifiers."""
        script = r"""
$ErrorActionPreference = 'SilentlyContinue'
$os = Get-CimInstance Win32_OperatingSystem
$computer = Get-CimInstance Win32_ComputerSystem
$cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
$bios = Get-CimInstance Win32_BIOS | Select-Object -First 1
$displayVersion = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion').DisplayVersion
$secureBoot = $null
try { $secureBoot = [bool](Confirm-SecureBootUEFI) } catch { }
$tpm = $null
try { $tpm = Get-Tpm } catch { }
$uptime = $null
if ($os.LastBootUpTime) { $uptime = (Get-Date) - $os.LastBootUpTime }

$gpus = @(Get-CimInstance Win32_VideoController | Where-Object { $_.Name } | ForEach-Object {
    [pscustomobject]@{
        name = $_.Name
        driver = $_.DriverVersion
        memory_gb = if ($_.AdapterRAM) { [math]::Round(([double]$_.AdapterRAM / 1GB), 1) } else { $null }
    }
})
$volumes = @(Get-CimInstance Win32_LogicalDisk -Filter "DriveType=3" | Where-Object { $_.Size } | ForEach-Object {
    [pscustomobject]@{
        drive = $_.DeviceID
        filesystem = $_.FileSystem
        size_gb = [math]::Round(([double]$_.Size / 1GB), 1)
        free_gb = [math]::Round(([double]$_.FreeSpace / 1GB), 1)
    }
})
$network = @(Get-NetAdapter -Physical | Where-Object { $_.Status -eq 'Up' } | ForEach-Object {
    [pscustomobject]@{
        name = $_.Name
        status = $_.Status
        link_speed = $_.LinkSpeed
    }
})

[pscustomobject]@{
    os = [pscustomobject]@{
        caption = $os.Caption
        display_version = $displayVersion
        version = $os.Version
        build = $os.BuildNumber
        architecture = $os.OSArchitecture
    }
    computer = [pscustomobject]@{
        manufacturer = $computer.Manufacturer
        model = $computer.Model
        memory_gb = if ($computer.TotalPhysicalMemory) { [math]::Round(([double]$computer.TotalPhysicalMemory / 1GB), 1) } else { $null }
        memory_free_gb = if ($os.FreePhysicalMemory) { [math]::Round(([double]$os.FreePhysicalMemory / 1MB), 1) } else { $null }
        hypervisor_present = [bool]$computer.HypervisorPresent
    }
    cpu = [pscustomobject]@{
        name = $cpu.Name
        cores = $cpu.NumberOfCores
        logical_processors = $cpu.NumberOfLogicalProcessors
        max_clock_mhz = $cpu.MaxClockSpeed
        virtualization_firmware = $cpu.VirtualizationFirmwareEnabled
    }
    firmware = [pscustomobject]@{
        bios_vendor = $bios.Manufacturer
        bios_version = $bios.SMBIOSBIOSVersion
        bios_date = if ($bios.ReleaseDate) { $bios.ReleaseDate.ToString('yyyy-MM-dd') } else { $null }
        mode = if ($secureBoot -ne $null) { 'UEFI' } else { 'No confirmado' }
    }
    security = [pscustomobject]@{
        secure_boot = $secureBoot
        tpm_present = if ($tpm) { [bool]$tpm.TpmPresent } else { $null }
        tpm_ready = if ($tpm) { [bool]$tpm.TpmReady } else { $null }
        tpm_spec = if ($tpm) { $tpm.SpecVersion } else { $null }
    }
    uptime_hours = if ($uptime) { [math]::Round($uptime.TotalHours, 1) } else { $null }
    gpus = $gpus
    volumes = $volumes
    network = $network
} | ConvertTo-Json -Depth 6 -Compress
"""
        try:
            completed = subprocess.run(
                ["powershell.exe", "-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script],
                capture_output=True,
                text=True,
                timeout=10,
                check=False,
            )
            if completed.returncode != 0 or not completed.stdout.strip():
                return {}
            data = json.loads(completed.stdout.strip())
            for key in ("gpus", "volumes", "network"):
                value = data.get(key)
                if value is None:
                    data[key] = []
                elif isinstance(value, dict):
                    data[key] = [value]
            return data
        except (OSError, subprocess.SubprocessError, json.JSONDecodeError, TypeError, ValueError):
            return {}

    def open_guide(self, guide_name: str) -> Dict[str, Any]:
        """Open a local documentation guide in user's default browser."""
        try:
            from packwire.core.gui.server import get_web_dir
            web_dir = get_web_dir()
            guide_path = (web_dir / "guides" / guide_name).resolve()
            if not guide_path.exists():
                root_guide = (Path(__file__).resolve().parent.parent.parent.parent / "guides" / guide_name).resolve()
                if root_guide.exists():
                    guide_path = root_guide
            if guide_path.exists():
                webbrowser.open(guide_path.as_uri())
                return {"success": True, "path": str(guide_path)}
            else:
                return {"success": False, "message": f"Guía no encontrada: {guide_name}"}
        except Exception as e:
            return {"success": False, "message": str(e)}

    def list_available(self) -> List[Dict[str, Any]]:
        """List all packages in catalog with resolved version if stable."""
        manifests = packwire.list_available()
        # Ensure versions are resolved for display
        results = []
        for m in manifests:
            resolved = packwire.get_manifest(m["id"]) or m
            results.append(resolved)
        return results

    def list_installed(self) -> Dict[str, Dict[str, Any]]:
        """List all currently installed packages."""
        return packwire.list_installed()

    def queue_install(
        self,
        package_id: str,
        mode: Optional[str] = None,
        version: Optional[str] = None,
        force: bool = False
    ) -> Dict[str, Any]:
        """Enqueue package installation into the background threaded worker."""
        try:
            m = packwire.get_manifest(package_id)
            pkg_name = m["name"] if m else package_id
            target_mode = mode or (m["install"]["mode"] if m and m.get("install") else "here")
            job_id = packwire.task_queue.submit_install(
                package_id=package_id,
                package_name=pkg_name,
                mode=target_mode,
                version=version,
                force=force
            )
            return {"success": True, "job_id": job_id}
        except Exception as e:
            return {"success": False, "message": str(e)}

    def get_job(self, job_id: str) -> Optional[Dict[str, Any]]:
        """Retrieve live status, progress and log stream for a background job."""
        return packwire.task_queue.get_job(job_id)

    def list_jobs(self) -> List[Dict[str, Any]]:
        """List active and recent background jobs."""
        return packwire.task_queue.list_jobs()

    def install(
        self,
        package_id: str,
        mode: Optional[str] = None,
        version: Optional[str] = None,
        force: bool = False
    ) -> Dict[str, Any]:
        """Execute package installation synchronously."""
        try:
            ok, msg = packwire.install(
                target=package_id,
                version=version,
                mode=mode,
                force=force
            )
            return {"success": ok, "message": msg}
        except Exception as e:
            return {"success": False, "message": str(e)}

    def uninstall(self, package_id: str) -> Dict[str, Any]:
        """Uninstall a package."""
        try:
            ok, msg = packwire.uninstall(package_id)
            return {"success": ok, "message": msg}
        except Exception as e:
            return {"success": False, "message": str(e)}

    def uninstall_all(self, remove_path: bool = True) -> Dict[str, Any]:
        """Completely uninstall all Packwire packages, shims, cache, and remove from PATH."""
        try:
            ok, msg = packwire.uninstall_all(remove_path=remove_path)
            return {"success": ok, "message": msg}
        except Exception as e:
            return {"success": False, "message": str(e)}

    def reinstall(self, package_id: str) -> Dict[str, Any]:
        """Reinstall a package cleanly."""
        try:
            ok, msg = packwire.reinstall(package_id)
            return {"success": ok, "message": msg}
        except Exception as e:
            return {"success": False, "message": str(e)}

    def check_updates(self) -> Dict[str, Any]:
        """Check for updates in background for non-fixed installed packages."""
        try:
            return packwire.check_updates()
        except Exception as e:
            print(f"[GuiBridge] Error al buscar actualizaciones: {e}")
            return {}

    def update(self, package_id: Optional[str] = None) -> List[Dict[str, Any]]:
        """Check and update packages."""
        results = packwire.update(package_id)
        return [{"id": pkg_id, "success": ok, "message": msg} for pkg_id, ok, msg in results]

    def add_path(self) -> Dict[str, Any]:
        """Add shims directory to user PATH."""
        try:
            success = packwire.add_path()
            return {
                "success": success,
                "message": "Directorio de shims añadido al PATH." if success else "No se pudo actualizar el PATH."
            }
        except Exception as e:
            return {"success": False, "message": str(e)}

    def add_defender_exclusion(self) -> Dict[str, Any]:
        """Configure Windows Defender exclusion to prevent false positive deletion of as.exe."""
        try:
            success, msg = packwire.add_defender_exclusion()
            return {"success": success, "message": msg}
        except Exception as e:
            return {"success": False, "message": str(e)}

    def open_site(self, url: str) -> Dict[str, Any]:
        """Open a URL in default browser."""
        try:
            webbrowser.open(url)
            return {"success": True}
        except Exception as e:
            return {"success": False, "message": str(e)}

    def list_themes(self) -> List[str]:
        """Scan global/ css directory and discover available themes dynamically."""
        from pathlib import Path
        import sys
        if getattr(sys, "frozen", False) and hasattr(sys, "_MEIPASS"):
            candidate = Path(sys._MEIPASS) / "packwire" / "core" / "gui" / "web" / "global"
            global_css_dir = candidate if candidate.exists() else Path(sys._MEIPASS) / "web" / "global"
        else:
            global_css_dir = Path(__file__).resolve().parent / "web" / "global"
        themes = set()
        if global_css_dir.exists():
            for f in global_css_dir.glob("*.css"):
                parts = f.stem.split("_", 1)
                if len(parts) == 2:
                    themes.add(parts[0])
        sorted_themes = sorted(list(themes))
        if "default" in sorted_themes:
            sorted_themes.remove("default")
            sorted_themes.insert(0, "default")
        return sorted_themes or ["default", "dark"]

    def get_config(self, key: Optional[str] = None) -> Any:
        """Retrieve user configuration from configs.json."""
        if key:
            return packwire.get_config(key)
        return packwire.load_config()

    def set_config(self, key: str, value: Any) -> Dict[str, Any]:
        """Update and persist a configuration entry into configs.json."""
        try:
            packwire.set_config(key, value)
            return {"success": True}
        except Exception as e:
            return {"success": False, "message": str(e)}

