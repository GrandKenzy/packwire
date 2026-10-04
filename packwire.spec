# -*- mode: python ; coding: utf-8 -*-
from pathlib import Path
import sys

block_cipher = None

try:
    project_dir = Path(SPECPATH).resolve()
except NameError:
    project_dir = Path('.').resolve()

datas = [
    (str(project_dir / 'packwire' / 'core' / 'manifests'), 'packwire/core/manifests'),
    (str(project_dir / 'packwire' / 'core' / 'gui' / 'web'), 'packwire/core/gui/web'),
]

icon_path = project_dir / 'packwire' / 'core' / 'gui' / 'web' / 'logo.ico'
icon_file = str(icon_path) if (icon_path.exists() and sys.platform == "win32") else None

hiddenimports = [
    'bottle',
    'pywebview',
    'webview',
    'ctypes',
    'json',
    'urllib.request',
    'urllib.error',
    'shutil',
    'zipfile',
    'tarfile',
    'threading',
    'subprocess',
    'http.server',
]

if sys.platform == "win32":
    hiddenimports.extend([
        'webview.platforms.winforms',
        'webview.platforms.edgechromium',
        'ctypes.wintypes',
        'winreg',
    ])
elif sys.platform == "darwin":
    hiddenimports.extend([
        'webview.platforms.cocoa',
    ])
else:
    hiddenimports.extend([
        'webview.platforms.gtk',
    ])

a = Analysis(
    ['packwire/__main__.py'],
    pathex=[str(project_dir)],
    binaries=[],
    datas=datas,
    hiddenimports=hiddenimports,
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=['tkinter', 'matplotlib', 'numpy', 'scipy', 'pandas', 'IPython'],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name='packwire',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=True,  # Dual: Can be run from CLI with console output or launch GUI
    disable_windowed_traceback=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon=icon_file,
)
