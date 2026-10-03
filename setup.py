#!/usr/bin/env python
from setuptools import setup, find_packages

setup(
    name="packwire",
    version="0.1.0",
    description="Gestor y descargador moderno, elegante y portátil de paquetes para Windows",
    author="Packwire Team",
    packages=find_packages(),
    include_package_data=True,
    package_data={
        "packwire.core": [
            "manifests/*.json",
            "gui/web/*.*",
            "gui/web/global/*.css",
        ]
    },
    install_requires=[
        "pywebview>=4.0.0",
        "bottle>=0.12.0",
    ],
    extras_require={
        "build": [
            "pyinstaller>=6.0.0",
            "build>=1.0.0",
            "wheel>=0.40.0",
        ]
    },
    entry_points={
        "console_scripts": [
            "packwire=packwire.cli:main",
        ],
    },
    python_requires=">=3.9",
)
