"""
Packwire GUI Package.
Provides modern HTML/CSS interface via native desktop WebView or local server.
"""

from .window import launch_gui
from .bridge import GuiBridge

__all__ = ["launch_gui", "GuiBridge"]
