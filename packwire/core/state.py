import json
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from datetime import datetime
from packwire.core.config import STATE_FILE, ensure_directories
from packwire.core.models import PackageState


def load_state() -> Dict[str, PackageState]:
    """Load registry of installed packages."""
    ensure_directories()
    if not STATE_FILE.exists():
        return {}
    try:
        data = json.loads(STATE_FILE.read_text(encoding="utf-8"))
        return {pkg_id: PackageState.from_dict(item) for pkg_id, item in data.items()}
    except Exception:
        return {}


def save_state(state: Dict[str, PackageState]) -> None:
    """Save registry of installed packages to disk."""
    ensure_directories()
    data = {pkg_id: pkg.to_dict() for pkg_id, pkg in state.items()}
    STATE_FILE.write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8")


def check_collision(
    pkg_id: str,
    binaries: List[str]
) -> Tuple[bool, Optional[str], List[str]]:
    """
    Check if installing this package causes a collision.
    Returns:
        (is_duplicate, duplicate_message, colliding_binaries)
    """
    state = load_state()

    # 1. Exact duplicate package check
    if pkg_id in state:
        existing = state[pkg_id]
        return True, f"El paquete '{pkg_id}' ya está instalado (versión {existing.version}).", []

    # 2. Binary shim collision check
    colliding_binaries = []
    for existing_id, existing_pkg in state.items():
        for b in binaries:
            base_b = Path(b).stem.lower()
            existing_shims = [Path(s).stem.lower() for s in existing_pkg.shims]
            if base_b in existing_shims or b in existing_pkg.binaries:
                colliding_binaries.append(f"{b} (ya provisto por '{existing_id}')")

    return False, None, colliding_binaries


def register_installed(pkg_state: PackageState) -> None:
    """Register or update an installed package in state."""
    state = load_state()
    if not pkg_state.installed_at:
        pkg_state.installed_at = datetime.now().isoformat()
    state[pkg_state.id] = pkg_state
    save_state(state)


def unregister_installed(pkg_id: str) -> Optional[PackageState]:
    """Unregister an installed package from state."""
    state = load_state()
    if pkg_id in state:
        removed = state.pop(pkg_id)
        save_state(state)
        return removed
    return None


def get_installed(pkg_id: str) -> Optional[PackageState]:
    """Get package state by id."""
    state = load_state()
    return state.get(pkg_id)
