from dataclasses import dataclass, field, asdict
from typing import List, Optional, Dict, Any
import hashlib
import json


@dataclass
class HereConfig:
    url: Optional[str] = None
    format: str = "zip"
    binaries: List[str] = field(default_factory=lambda: ["python.exe"])
    addpath: bool = True
    silent_args: List[str] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "HereConfig":
        return cls(
            url=data.get("url"),
            format=data.get("format", "zip"),
            binaries=data.get("binaries", ["python.exe"]),
            addpath=data.get("addpath", data.get("add_path", True)),
            silent_args=data.get("silent_args", [])
        )


@dataclass
class SiteConfig:
    url: str

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "SiteConfig":
        return cls(url=data.get("url", ""))


@dataclass
class CommandConfig:
    script: Optional[str] = None
    windows: Optional[str] = None
    linux: Optional[str] = None
    darwin: Optional[str] = None
    shell: str = "powershell"
    elevated: bool = False
    binaries: List[str] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Any) -> "CommandConfig":
        if isinstance(data, str):
            return cls(script=data)
        if not isinstance(data, dict):
            return cls()
        return cls(
            script=data.get("script") or data.get("command"),
            windows=data.get("windows"),
            linux=data.get("linux"),
            darwin=data.get("darwin") or data.get("macos"),
            shell=data.get("shell", "powershell"),
            elevated=data.get("elevated", False),
            binaries=data.get("binaries", [])
        )


@dataclass
class InstallConfig:
    mode: str = "here"  # "here", "site", "command", "multi_os"
    clean: bool = True
    here: Optional[HereConfig] = None
    site: Optional[SiteConfig] = None
    command: Optional[CommandConfig] = None
    os: Optional[Dict[str, Any]] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "InstallConfig":
        here_data = data.get("here")
        site_data = data.get("site")
        cmd_data = data.get("command") or data.get("console") or data.get("script")
        return cls(
            mode=data.get("mode", "here"),
            clean=data.get("clean", True),
            here=HereConfig.from_dict(here_data) if here_data else None,
            site=SiteConfig.from_dict(site_data) if site_data else None,
            command=CommandConfig.from_dict(cmd_data) if cmd_data else None,
            os=data.get("os")
        )


@dataclass
class Manifest:
    id: str
    name: str
    type: str  # "stable" or "fixed"
    version: Optional[str] = None
    icon: str = "📦"
    category: str = "general"
    description: str = ""
    aliases: List[str] = field(default_factory=list)
    upstream: Optional[Dict[str, Any]] = None
    install: InstallConfig = field(default_factory=InstallConfig)
    manifest_hash: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Manifest":
        install_data = data.get("install", {})
        
        # Support user's initial format where version might be [3, 14] or string "3.14"
        ver = data.get("version")
        if isinstance(ver, list):
            ver = ".".join(str(v) for v in ver)
        elif ver is not None:
            ver = str(ver)

        pkg_type = data.get("type", "stable").lower()
        if pkg_type == "latest":
            pkg_type = "stable"

        pkg_id = data.get("id")
        if not pkg_id:
            # Construct deterministic semantic ID
            name_slug = data.get("name", "package").lower()
            if pkg_type == "stable":
                pkg_id = f"{name_slug}@stable"
            elif ver:
                pkg_id = f"{name_slug}@{ver}"
            else:
                pkg_id = f"{name_slug}@default"

        manifest = cls(
            id=pkg_id,
            name=data.get("name", "Unknown"),
            type=pkg_type,
            version=ver,
            icon=data.get("icon", "📦"),
            category=data.get("agroup", data.get("category", "general")),
            description=data.get("description", ""),
            aliases=data.get("aliases", []),
            upstream=data.get("upstream"),
            install=InstallConfig.from_dict(install_data)
        )
        manifest.manifest_hash = manifest.calculate_hash()
        return manifest

    def calculate_hash(self) -> str:
        """Compute SHA256 of canonical fields to ensure deterministic integrity."""
        canonical_data = {
            "name": self.name.lower(),
            "type": self.type,
            "version": self.version,
            "install": {
                "mode": self.install.mode,
                "clean": self.install.clean,
                "here_url": self.install.here.url if self.install.here else None,
                "site_url": self.install.site.url if self.install.site else None,
                "command_script": self.install.command.script if self.install.command else None,
                "binaries": self.install.here.binaries if self.install.here else (self.install.command.binaries if self.install.command else []),
                "addpath": self.install.here.addpath if self.install.here else False,
            }
        }
        encoded = json.dumps(canonical_data, sort_keys=True).encode("utf-8")
        return hashlib.sha256(encoded).hexdigest()

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class PackageState:
    id: str
    name: str
    version: str
    type: str  # "stable" or "fixed"
    mode: str  # "here" or "site"
    install_path: Optional[str] = None
    binaries: List[str] = field(default_factory=list)
    shims: List[str] = field(default_factory=list)
    installed_at: str = ""
    manifest_hash: str = ""

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "PackageState":
        return cls(
            id=data["id"],
            name=data["name"],
            version=str(data.get("version", "")),
            type=data.get("type", "stable"),
            mode=data.get("mode", "here"),
            install_path=data.get("install_path"),
            binaries=data.get("binaries", []),
            shims=data.get("shims", []),
            installed_at=data.get("installed_at", ""),
            manifest_hash=data.get("manifest_hash", "")
        )

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)
