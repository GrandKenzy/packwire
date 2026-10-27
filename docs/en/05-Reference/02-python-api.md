# Python API Reference

Packwire exports a typed, thread-safe Python programming API from the root `packwire` package.

---

## 1. Package Management Functions

```python
import packwire

# Install software
success, msg = packwire.install(
    target="python",
    version=None,
    channel="stable",
    mode="here",
    force=False,
    logger=None
)

# Uninstall software
success, msg = packwire.uninstall("python@stable")

# Complete uninstallation
success, msg = packwire.uninstall_all(remove_path=True)

# Reinstall software
success, msg = packwire.reinstall("python@stable")

# Update packages
results = packwire.update("python@stable")
for pkg_id, ok, msg in results:
    print(pkg_id, ok, msg)

# Check updates without installing
updates = packwire.check_updates()

# Query installed and available packages
installed = packwire.list_installed()
available = packwire.list_available()
manifest = packwire.get_manifest("git")
```

---

## 2. System and Environment Functions

```python
# Add shims directory to user PATH
packwire.add_path()

# Configure Windows Defender exclusions
ok, msg = packwire.add_defender_exclusion()

# Launch graphical interface
packwire.launch_gui(web_mode=False, port=5050)

# Submit background job to worker queue
job_id = packwire.task_queue.submit_install("python", mode="here")
job_info = packwire.task_queue.get_job(job_id)
```
