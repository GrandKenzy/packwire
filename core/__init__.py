"""
Packwire Core Compatibility Module.
Redirects legacy 'core' imports to 'packwire.core'.
"""

import sys
import packwire.core as _pc

# Re-export and register in sys.modules
sys.modules["core"] = _pc
sys.modules["core.config"] = _pc.config
sys.modules["core.models"] = _pc.models
sys.modules["core.state"] = _pc.state
sys.modules["core.resolver"] = _pc.resolver
sys.modules["core.installer"] = _pc.installer
sys.modules["core.task_queue"] = _pc.task_queue
sys.modules["core.visitors"] = _pc.visitors
sys.modules["core.gui"] = _pc.gui
sys.modules["core.gui.bridge"] = _pc.gui.bridge
sys.modules["core.gui.server"] = _pc.gui.server
sys.modules["core.gui.window"] = _pc.gui.window

__all__ = ["_pc"]
