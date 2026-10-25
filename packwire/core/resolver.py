import urllib.request
import json
import re
from typing import Optional, Dict, Any, Tuple
from packwire.core.models import Manifest


class PythonResolver:
    """Resolver for official Python releases."""

    RELEASES_API = "https://www.python.org/api/v2/downloads/release/?is_published=true"
    RELEASE_FILES_API = "https://www.python.org/api/v2/downloads/release_file/?release="

    _cached_releases = None

    @classmethod
    def get_all_releases(cls) -> list:
        if cls._cached_releases is not None:
            return cls._cached_releases
        req = urllib.request.Request(
            cls.RELEASES_API,
            headers={"User-Agent": "Packwire/0.1.0"}
        )
        try:
            with urllib.request.urlopen(req, timeout=10) as res:
                cls._cached_releases = json.loads(res.read().decode("utf-8"))
                return cls._cached_releases
        except Exception as e:
            print(f"[Packwire Resolver] Error al consultar API de Python: {e}")
            return []

    @classmethod
    def resolve_latest_stable(cls) -> Tuple[str, str, str]:
        """
        Find latest stable Python release.
        Returns (version_str, download_url_zip, release_page_url)
        """
        releases = cls.get_all_releases()
        latest_rel = None

        # Filter for Python 3 stable releases and sort by version tuple descending
        py3_candidates = []
        for r in releases:
            name = r.get("name", "")
            if not r.get("pre_release") and name.startswith("Python 3."):
                m = re.match(r"^Python (3\.\d+\.\d+)$", name)
                if m:
                    ver_str = m.group(1)
                    parts = tuple(int(x) for x in ver_str.split("."))
                    py3_candidates.append((parts, r))

        if py3_candidates:
            py3_candidates.sort(key=lambda item: item[0], reverse=True)
            latest_rel = py3_candidates[0][1]

        if not latest_rel:
            raise RuntimeError("No se pudo resolver la última versión estable de Python 3.")

        # Extract version number, e.g. "Python 3.14.8" -> "3.14.8"
        name = latest_rel.get("name", "")
        version = name.replace("Python ", "").strip()
        release_id = latest_rel.get("resource_uri", "").rstrip("/").split("/")[-1]

        # Query files for this release to find embeddable zip
        download_url = None
        if release_id:
            files_url = f"{cls.RELEASE_FILES_API}{release_id}"
            try:
                req = urllib.request.Request(files_url, headers={"User-Agent": "Packwire/0.1.0"})
                with urllib.request.urlopen(req, timeout=10) as res:
                    files_data = json.loads(res.read().decode("utf-8"))
                    for f in files_data:
                        if "Windows embeddable package (64-bit)" in f.get("name", ""):
                            download_url = f.get("url")
                            break
            except Exception:
                pass

        if not download_url:
            # Fallback URL format
            download_url = f"https://www.python.org/ftp/python/{version}/python-{version}-embed-amd64.zip"

        slug = latest_rel.get("slug", version.replace(".", ""))
        site_url = f"https://www.python.org/downloads/release/{slug}/"

        return version, download_url, site_url

    @classmethod
    def resolve_fixed(cls, version: str) -> Tuple[str, str, str]:
        """
        Resolve fixed version URLs for Python.
        """
        # Ensure version is like 3.14.8 or 3.14.0
        parts = version.split(".")
        if len(parts) == 2:
            version = f"{version}.0"

        download_url = f"https://www.python.org/ftp/python/{version}/python-{version}-embed-amd64.zip"
        ver_slug = "python-" + version.replace(".", "")
        site_url = f"https://www.python.org/downloads/release/{ver_slug}/"
        return version, download_url, site_url


class NodeResolver:
    """Resolver for official Node.js releases."""
    DIST_INDEX_API = "https://nodejs.org/dist/index.json"
    _cached_releases = None

    @classmethod
    def get_all_releases(cls) -> list:
        if cls._cached_releases is not None:
            return cls._cached_releases
        req = urllib.request.Request(
            cls.DIST_INDEX_API,
            headers={"User-Agent": "Packwire/0.1.0"}
        )
        try:
            with urllib.request.urlopen(req, timeout=10) as res:
                cls._cached_releases = json.loads(res.read().decode("utf-8"))
                return cls._cached_releases
        except Exception as e:
            print(f"[Packwire Resolver] Error al consultar API de Node.js: {e}")
            return []

    @classmethod
    def resolve_latest_lts(cls) -> Tuple[str, str, str]:
        """
        Find latest LTS Node.js release.
        Returns (version_str, download_url_zip, release_page_url)
        """
        releases = cls.get_all_releases()
        for item in releases:
            if item.get("lts") and "win-x64-zip" in item.get("files", []):
                ver_tag = item.get("version", "").lstrip("v")
                dl_url = f"https://nodejs.org/dist/v{ver_tag}/node-v{ver_tag}-win-x64.zip"
                site_url = "https://nodejs.org/en/download"
                return ver_tag, dl_url, site_url

        # Fallback default if offline
        return "24.21.0", "https://nodejs.org/dist/v24.21.0/node-v24.21.0-win-x64.zip", "https://nodejs.org/en/download"

    @classmethod
    def resolve_fixed(cls, version: str) -> Tuple[str, str, str]:
        ver_tag = version.lstrip("v")
        dl_url = f"https://nodejs.org/dist/v{ver_tag}/node-v{ver_tag}-win-x64.zip"
        site_url = "https://nodejs.org/en/download"
        return ver_tag, dl_url, site_url


class GitHubReleaseResolver:
    """Resolve official, non-prerelease assets from a GitHub repository."""

    API_BASE = "https://api.github.com/repos"
    _cache: Dict[str, list] = {}

    @classmethod
    def get_releases(cls, repository: str) -> list:
        if repository in cls._cache:
            return cls._cache[repository]
        url = f"{cls.API_BASE}/{repository}/releases?per_page=30"
        req = urllib.request.Request(url, headers={
            "Accept": "application/vnd.github+json",
            "User-Agent": "Packwire/0.1.4"
        })
        try:
            with urllib.request.urlopen(req, timeout=15) as res:
                releases = json.loads(res.read().decode("utf-8"))
                cls._cache[repository] = releases
                return releases
        except Exception as exc:
            raise RuntimeError(
                f"No se pudo consultar el catálogo oficial de releases de GitHub ({repository}): {exc}"
            ) from exc

    @classmethod
    def resolve(
        cls,
        repository: str,
        asset_pattern: str,
        version_pattern: str,
        site_url: str,
        release_tag: Optional[str] = None,
    ) -> Tuple[str, str, str, Optional[str]]:
        asset_re = re.compile(asset_pattern, re.IGNORECASE)
        version_re = re.compile(version_pattern, re.IGNORECASE)
        for release in cls.get_releases(repository):
            if release_tag and release.get("tag_name") != release_tag:
                continue
            if release.get("draft") or release.get("prerelease"):
                continue
            for asset in release.get("assets", []):
                asset_name = asset.get("name", "")
                if not asset_re.fullmatch(asset_name):
                    continue
                match = version_re.search(asset_name) or version_re.search(release.get("tag_name", ""))
                if not match:
                    continue
                digest = asset.get("digest") or ""
                if digest.startswith("sha256:"):
                    digest = digest.split(":", 1)[1]
                return (
                    match.group(1),
                    asset.get("browser_download_url", ""),
                    site_url,
                    digest or None,
                )
        raise RuntimeError(
            f"No se encontró un asset estable compatible en {repository} ({asset_pattern})."
        )


class W64DevKitResolver:
    @classmethod
    def resolve_latest(cls) -> Tuple[str, str, str, Optional[str]]:
        # Current releases are self-extracting 7z executables, not the old ZIP.
        return GitHubReleaseResolver.resolve(
            "skeeto/w64devkit",
            r"w64devkit-x64-(\d+(?:\.\d+)+)\.7z\.exe",
            r"(\d+(?:\.\d+)+)(?=\.7z\.exe)",
            "https://github.com/skeeto/w64devkit/releases",
        )

    @classmethod
    def resolve_fixed(cls, version: str) -> Tuple[str, str, str, Optional[str]]:
        return GitHubReleaseResolver.resolve(
            "skeeto/w64devkit",
            rf"w64devkit-x64-{re.escape(version)}\.7z\.exe",
            rf"({re.escape(version)})",
            "https://github.com/skeeto/w64devkit/releases",
            release_tag=f"v{version.lstrip('v')}",
        )


class LLVMResolver:
    @classmethod
    def resolve_latest(cls) -> Tuple[str, str, str, Optional[str]]:
        return GitHubReleaseResolver.resolve(
            "llvm/llvm-project",
            r"clang\+llvm-(\d+(?:\.\d+)+)-x86_64-pc-windows-msvc\.tar\.xz",
            r"(\d+(?:\.\d+)+)",
            "https://github.com/llvm/llvm-project/releases",
        )

    @classmethod
    def resolve_fixed(cls, version: str) -> Tuple[str, str, str, Optional[str]]:
        version = version.lstrip("v")
        return GitHubReleaseResolver.resolve(
            "llvm/llvm-project",
            rf"clang\+llvm-{re.escape(version)}-x86_64-pc-windows-msvc\.tar\.xz",
            rf"({re.escape(version)})",
            "https://github.com/llvm/llvm-project/releases",
            release_tag=f"llvmorg-{version}",
        )


class RufusResolver:
    @classmethod
    def resolve_latest(cls) -> Tuple[str, str, str, Optional[str]]:
        return GitHubReleaseResolver.resolve(
            "pbatard/rufus",
            r"rufus-(\d+(?:\.\d+)+)p\.exe",
            r"(\d+(?:\.\d+)+)(?=p\.exe)",
            "https://rufus.ie/en/",
        )


class VentoyResolver:
    @classmethod
    def resolve_latest(cls) -> Tuple[str, str, str, Optional[str]]:
        return GitHubReleaseResolver.resolve(
            "ventoy/Ventoy",
            r"ventoy-(\d+(?:\.\d+)+)-windows\.zip",
            r"(\d+(?:\.\d+)+)(?=-windows\.zip)",
            "https://www.ventoy.net/en/download.html",
        )


class PowerShellResolver:
    @classmethod
    def resolve_latest(cls) -> Tuple[str, str, str, Optional[str]]:
        return GitHubReleaseResolver.resolve(
            "PowerShell/PowerShell",
            r"PowerShell-(\d+(?:\.\d+)+)-win-x64\.zip",
            r"(\d+(?:\.\d+)+)(?=-win-x64\.zip)",
            "https://learn.microsoft.com/powershell/",
        )


def resolve_manifest(manifest: Manifest) -> Manifest:
    """
    Resolve dynamic fields for a manifest based on package type (stable vs fixed).
    """
    pkg_name = manifest.name.lower()

    if pkg_name == "python":
        if manifest.type == "stable":
            ver, dl_url, site_url = PythonResolver.resolve_latest_stable()
            manifest.version = ver
            manifest.id = "python@stable"
            if manifest.install.here and not manifest.install.here.url:
                manifest.install.here.url = dl_url
            if manifest.install.site and not manifest.install.site.url:
                manifest.install.site.url = site_url

        elif manifest.type == "fixed":
            if not manifest.version:
                raise ValueError("Un paquete 'fixed' debe especificar una versión en el manifest.")
            ver, dl_url, site_url = PythonResolver.resolve_fixed(manifest.version)
            manifest.version = ver
            manifest.id = f"python@{ver}"
            if manifest.install.here and not manifest.install.here.url:
                manifest.install.here.url = dl_url
            if manifest.install.site and not manifest.install.site.url:
                manifest.install.site.url = site_url

    elif pkg_name in ("node", "nodejs", "node.js"):
        if manifest.type == "stable":
            ver, dl_url, site_url = NodeResolver.resolve_latest_lts()
            manifest.version = ver
            manifest.id = "nodejs@stable"
            if manifest.install.here and not manifest.install.here.url:
                manifest.install.here.url = dl_url
            if manifest.install.site and not manifest.install.site.url:
                manifest.install.site.url = site_url

        elif manifest.type == "fixed":
            if not manifest.version:
                manifest.version = "24.21.0"
            ver, dl_url, site_url = NodeResolver.resolve_fixed(manifest.version)
            manifest.version = ver
            manifest.id = f"nodejs@{ver}"
            if manifest.install.here and not manifest.install.here.url:
                manifest.install.here.url = dl_url
            if manifest.install.site and not manifest.install.site.url:
                manifest.install.site.url = site_url

    source = (manifest.upstream or {}).get("source", "")
    if source == "github_w64devkit" and manifest.type == "stable":
        ver, dl_url, site_url, digest = W64DevKitResolver.resolve_latest()
        manifest.version = ver
        if manifest.install.here:
            manifest.install.here.url = dl_url
            manifest.install.here.expected_hash = digest
        if manifest.install.site:
            manifest.install.site.url = site_url

    elif source == "github_w64devkit" and manifest.type == "fixed":
        if not manifest.version:
            raise ValueError("Un paquete fixed de w64devkit debe especificar una versión.")
        ver, dl_url, site_url, digest = W64DevKitResolver.resolve_fixed(manifest.version)
        manifest.version = ver
        if manifest.install.here:
            manifest.install.here.url = dl_url
            manifest.install.here.expected_hash = digest
        if manifest.install.site:
            manifest.install.site.url = site_url

    elif source == "github_llvm" and manifest.type == "stable":
        ver, dl_url, site_url, digest = LLVMResolver.resolve_latest()
        manifest.version = ver
        if manifest.install.here:
            manifest.install.here.url = dl_url
            manifest.install.here.expected_hash = digest
        if manifest.install.site:
            manifest.install.site.url = site_url

    elif source == "github_llvm" and manifest.type == "fixed":
        if not manifest.version:
            raise ValueError("Un paquete fixed de LLVM debe especificar una versión.")
        ver, dl_url, site_url, digest = LLVMResolver.resolve_fixed(manifest.version)
        manifest.version = ver
        if manifest.install.here:
            manifest.install.here.url = dl_url
            manifest.install.here.expected_hash = digest
        if manifest.install.site:
            manifest.install.site.url = site_url

    elif source == "github_rufus" and manifest.type == "stable":
        ver, dl_url, site_url, digest = RufusResolver.resolve_latest()
        manifest.version = ver
        if manifest.install.here:
            manifest.install.here.url = dl_url
            manifest.install.here.expected_hash = digest
        if manifest.install.site:
            manifest.install.site.url = site_url

    elif source == "github_ventoy" and manifest.type == "stable":
        ver, dl_url, site_url, digest = VentoyResolver.resolve_latest()
        manifest.version = ver
        if manifest.install.here:
            manifest.install.here.url = dl_url
            manifest.install.here.expected_hash = digest
        if manifest.install.site:
            manifest.install.site.url = site_url

    elif source == "github_powershell" and manifest.type == "stable":
        ver, dl_url, site_url, digest = PowerShellResolver.resolve_latest()
        manifest.version = ver
        if manifest.install.here:
            manifest.install.here.url = dl_url
            manifest.install.here.expected_hash = digest
        if manifest.install.site:
            manifest.install.site.url = site_url

    # Recalculate hash after resolution
    manifest.manifest_hash = manifest.calculate_hash()
    return manifest
