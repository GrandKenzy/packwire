import packwire

print("=" * 60)
print(" PACKWIRE: Demostración del Gestor de Paquetes ")
print("=" * 60)

# 1. Listar paquetes disponibles en el catálogo
print("\n[1] Paquetes disponibles en los manifests:")
for pkg in packwire.list_available():
    print(f"  - {pkg['id']} ({pkg['name']}) -> Tipo: {pkg['type']}, Categoría: {pkg['category']}")

# 2. Consultar el estado actual de paquetes instalados
print("\n[2] Paquetes actualmente instalados:")
installed = packwire.list_installed()
if not installed:
    print("  (Ninguno instalado todavía)")
else:
    for pkg_id, info in installed.items():
        print(f"  - {pkg_id}: v{info.get('version')} [{info.get('mode')}] en {info.get('install_path')}")

# 3. Inspeccionar el manifest resuelto de Python Stable
print("\n[3] Obteniendo información de Python Stable (resolución dinámica):")
py_manifest = packwire.get_manifest("python@stable")
if py_manifest:
    print(f"  ID: {py_manifest['id']}")
    print(f"  Versión resuelta: {py_manifest['version']}")
    print(f"  Tipo: {py_manifest['type']}")
    print(f"  URL 'here': {py_manifest['install']['here']['url']}")
    print(f"  URL 'site': {py_manifest['install']['site']['url']}")
    print(f"  Hash canónico: {py_manifest['manifest_hash'][:16]}...")

# 4. Estado de los Shims en el PATH
print("\n[4] Comprobando integración con PATH de Windows:")
in_path = packwire._pather.is_shims_in_path()
print(f"  Carpeta de Shims: {packwire.SHIMS_DIR}")
print(f"  ¿Está en PATH?: {'Sí' if in_path else 'No'}")

print("\n" + "=" * 60)
print(" Para instalar por comando o script:")
print("   packwire.install('python', channel='stable', mode='here')")
print("   packwire.install('python@3.14', mode='site')")
print(" O desde terminal:")
print("   python -m packwire install python --mode here")
print("=" * 60)
