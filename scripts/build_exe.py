#!/usr/bin/env python
"""
Shortcut script to build Packwire standalone executable.
Usage: python scripts/build_exe.py [--all|--wheel]
"""
import sys
from pathlib import Path

# Add project root to sys.path
root_dir = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(root_dir))

from packwire.packager import main

if __name__ == "__main__":
    main()
