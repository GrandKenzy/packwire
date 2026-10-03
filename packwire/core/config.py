import platform
import os
import sys
from pathlib import Path

SYSTEM = platform.system().lower()  # "windows", "darwin" (macos), "linux"
MACHINE = platform.machine().lower()  # "amd64", "x86_64", "arm64", "aarch64"


def get_default_root() -> Path:
    """Determine standard cross-platform user data directory for Packwire."""
    home = Path.home()
    if SYSTEM == "windows":
        app_data = os.environ.get("APPDATA")
        return Path(app_data) / "packwire" if app_data else home / ".packwire"
    elif SYSTEM == "darwin":
        return home / "Library" / "Application Support" / "packwire"
    else:
        # Linux and other UNIX: follow XDG Base Directory specification
        xdg_data = os.environ.get("XDG_DATA_HOME")
        return Path(xdg_data) / "packwire" if xdg_data else home / ".local" / "share" / "packwire"


PACKWIRE_ROOT = Path(os.environ.get("PACKWIRE_ROOT", get_default_root()))

# Standard subdirectories
APPS_DIR = PACKWIRE_ROOT / "apps"
SHIMS_DIR = PACKWIRE_ROOT / "shims"
CACHE_DIR = PACKWIRE_ROOT / "cache"
MANIFESTS_DIR = PACKWIRE_ROOT / "manifests"
STATE_FILE = PACKWIRE_ROOT / "installed.json"
CONFIG_FILE = PACKWIRE_ROOT / "configs.json"


def get_builtin_manifests_dir() -> Path:
    """Resolve manifests directory, supporting both source and PyInstaller frozen executables."""
    if getattr(sys, "frozen", False) and hasattr(sys, "_MEIPASS"):
        candidate = Path(sys._MEIPASS) / "packwire" / "core" / "manifests"
        if candidate.exists():
            return candidate
        candidate_root = Path(sys._MEIPASS) / "manifests"
        if candidate_root.exists():
            return candidate_root
    return Path(__file__).resolve().parent / "manifests"


# Builtin manifests in repository
BUILTIN_MANIFESTS_DIR = get_builtin_manifests_dir()

DEFAULT_CONFIG = {
    "theme": "default",
    "check_updates_on_start": True
}


def load_config() -> dict:
    """Load user preferences from configs.json."""
    ensure_directories()
    if not CONFIG_FILE.exists():
        save_config(DEFAULT_CONFIG)
        return DEFAULT_CONFIG.copy()
    try:
        import json
        data = json.loads(CONFIG_FILE.read_text(encoding="utf-8"))
        merged = DEFAULT_CONFIG.copy()
        merged.update(data)
        return merged
    except Exception:
        return DEFAULT_CONFIG.copy()


def save_config(cfg: dict) -> None:
    """Save user preferences to configs.json."""
    ensure_directories()
    import json
    CONFIG_FILE.write_text(json.dumps(cfg, indent=2, ensure_ascii=False), encoding="utf-8")


def get_config(key: str, default=None):
    """Retrieve a single config value."""
    cfg = load_config()
    return cfg.get(key, default)


def set_config(key: str, value) -> None:
    """Set and persist a single config value in configs.json."""
    cfg = load_config()
    cfg[key] = value
    save_config(cfg)


def ensure_directories():
    """Ensure all required packwire directories and base files exist."""
    for directory in (PACKWIRE_ROOT, APPS_DIR, SHIMS_DIR, CACHE_DIR, MANIFESTS_DIR):
        directory.mkdir(parents=True, exist_ok=True)
    if not STATE_FILE.exists():
        STATE_FILE.write_text("{}", encoding="utf-8")
    if not CONFIG_FILE.exists():
        import json
        CONFIG_FILE.write_text(json.dumps(DEFAULT_CONFIG, indent=2), encoding="utf-8")
