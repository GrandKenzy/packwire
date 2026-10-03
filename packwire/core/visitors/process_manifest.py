import json
import re
from pathlib import Path
from typing import Optional, Dict, Any, List

from packwire.core.models import Manifest
from packwire.core.config import BUILTIN_MANIFESTS_DIR, MANIFESTS_DIR
from packwire.core.resolver import resolve_manifest


class ManifestVisitor:
    """Visitor responsible for loading, parsing, validating and resolving manifests."""

    def __init__(self, manifests_dir: Optional[Path] = None):
        self.manifests_dir = manifests_dir or MANIFESTS_DIR

    @staticmethod
    def _strip_comments(text: str) -> str:
        """Remove C-style // and /* */ comments only when outside string literals."""
        def replacer(match):
            s = match.group(0)
            if s.startswith("/"):
                return ""
            return s

        pattern = re.compile(r'"(?:\\.|[^"\\])*"|//[^\r\n]*|/\*[\s\S]*?\*/')
        cleaned = re.sub(pattern, replacer, text)

        # Remove trailing commas only outside strings
        def comma_replacer(match):
            s = match.group(0)
            if s.startswith('"'):
                return s
            return match.group(1)

        cleaned = re.sub(r'"(?:\\.|[^"\\])*"|,\s*([\]}])', comma_replacer, cleaned)
        return cleaned

    def load_from_file(self, file_path: Path, auto_resolve: bool = True) -> Manifest:
        """Load and parse a manifest from a file path."""
        if not file_path.exists():
            raise FileNotFoundError(f"No se encontró el archivo de manifest: {file_path}")

        raw_text = file_path.read_text(encoding="utf-8")
        clean_text = self._strip_comments(raw_text).strip()

        # Handle multiple JSON objects in a single file if user concatenated them
        data = None
        try:
            data = json.loads(clean_text)
        except json.JSONDecodeError:
            # If multiple objects exist, try to parse the first one
            match = re.search(r"\{[\s\S]*?\}(?=\s*\{|\s*$)", clean_text)
            if match:
                data = json.loads(match.group(0))
            else:
                raise ValueError(f"El manifest {file_path.name} no contiene JSON válido.")

        manifest = Manifest.from_dict(data)

        if auto_resolve:
            manifest = resolve_manifest(manifest)

        return manifest

    def find_manifest(self, identifier: str) -> Optional[Manifest]:
        """
        Locate manifest by identifier (e.g. 'python', 'python@stable', 'python@3.14').
        Searches built-in directory and user manifests directory.
        """
        search_dirs = [
            BUILTIN_MANIFESTS_DIR,
            self.manifests_dir,
            Path(__file__).resolve().parent.parent / "packs"
        ]

        normalized_id = identifier.lower().replace("@", "-").replace(":", "-")

        # 1. Exact or common filename patterns
        potential_names = [
            f"{normalized_id}.json",
            f"{identifier}.json",
            f"{normalized_id.replace('python-', 'python-fixed-')}.json",
            f"{normalized_id.replace('python-', 'python-stable-')}.json",
            f"{normalized_id}-stable.json" if "@" not in identifier else None,
            "manifest.json"
        ]

        for d in search_dirs:
            if not d.exists():
                continue

            for name in filter(None, potential_names):
                target = d / name
                if target.is_file():
                    try:
                        return self.load_from_file(target)
                    except Exception as e:
                        print(f"[ManifestVisitor] Error al leer {target}: {e}")

            # 2. Iterate all json files in search directory and match by manifest.id, name, or aliases
            for f in d.glob("*.json"):
                try:
                    m = self.load_from_file(f, auto_resolve=False)
                    ident_low = identifier.lower()
                    if m.id.lower() == ident_low:
                        return resolve_manifest(m)

                    all_names = [m.name.lower()] + [a.lower() for a in getattr(m, "aliases", [])]
                    if ident_low in all_names:
                        return resolve_manifest(m)
                    if "@" in ident_low:
                        base_ident = ident_low.split("@")[0]
                        if base_ident in all_names:
                            return resolve_manifest(m)
                    if ident_low.replace("-", "").replace("_", "") in [n.replace("-", "").replace("_", "") for n in all_names]:
                        return resolve_manifest(m)
                except Exception:
                    continue

            # Subdirectory search (e.g. packs/<uuid>/manifest.json or packs/python/manifest.json)
            for sub in d.iterdir():
                if sub.is_dir():
                    sub_manifest = sub / "manifest.json"
                    if sub_manifest.is_file():
                        try:
                            m = self.load_from_file(sub_manifest, auto_resolve=False)
                            if m.id.lower() == identifier.lower() or m.name.lower() == identifier.lower():
                                return resolve_manifest(m)
                        except Exception:
                            continue

        return None

    def list_available_manifests(self) -> List[Manifest]:
        """List all available built-in and user manifests."""
        search_dirs = [BUILTIN_MANIFESTS_DIR, self.manifests_dir]
        manifests = []
        seen_ids = set()

        for d in search_dirs:
            if not d.exists():
                continue
            for f in d.glob("*.json"):
                try:
                    m = self.load_from_file(f, auto_resolve=False)
                    if m.id not in seen_ids:
                        manifests.append(m)
                        seen_ids.add(m.id)
                except Exception:
                    continue

        return manifests
