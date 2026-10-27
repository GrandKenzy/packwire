# Execution Flow

The installation pipeline in Packwire follows an orchestrated lifecycle to ensure reproducibility, data integrity, and atomic state transitions across all deployment modes.

---

## 1. Lifecycle Diagram

```text
[User / CLI / GUI]
         │
         ▼
[packwire.install()] ───────────► [ManifestVisitor]
         │                               │
         │ (Load & validate)             ▼
         ├───────────────────────► [Resolver] (Dynamic upstream resolution)
         │                               │
         │ (Resolved manifest)           ▼
         ▼
  [Installer] ───────────────────► [Collision Engine] (state.py)
         │                               │ (Verify binary collisions)
         ├───────────────────────────────┘
         ▼
  [Deployment Mode]
         │
         ├───► [here]    ────► [DownloaderVisitor] ──► Atomic extraction ──┐
         ├───► [command] ────► [Subprocess / UAC]  ──► Script / CLI        ─┤
         └───► [site]    ────► [WebBrowser]        ──► External site       ─┤
                                                                            │
         ┌──────────────────────────────────────────────────────────────────┘
         ▼
  [PatherVisitor] ───────────────► Generate Shims (.cmd / .ps1 / POSIX)
         │                        in %APPDATA%/packwire/shims
         ▼
  [State Registration] ──────────► Atomic write to installed.json
         │
         ▼
  [Completion] (TaskQueue notification / CLI exit)
```

---

## 2. Pipeline Stages

### Stage 1: Identification & Manifest Discovery
When an install command is issued, `ManifestVisitor` looks up the package manifest in built-in and user manifest directories. Canonical names are normalized (e.g. `python@stable`, `gcc`, `docker`).

### Stage 2: Dynamic Upstream Resolution
If the package belongs to the `stable` channel or requires dynamic asset resolution, `Resolver` queries official vendor APIs (such as python.org, nodejs.org, or GitHub Releases for LLVM/w64devkit) to obtain exact URLs, checksums, and version tags.

### Stage 3: Collision Verification
Before performing disk writes, `state.py` validates whether another installed package owns any of the target binaries. If collisions exist, installation requires an explicit `--force` flag.

### Stage 4: Execution Mode
- **`here`:** Headless download via `DownloaderVisitor` with progress callbacks and SHA-256 verification, followed by atomic unarchiving (ZIP/TAR/7z) with Zip-Slip path validation.
- **`command`:** Official console installation commands with elevated privilege support (UAC on Windows, `sudo`/`pkexec` on Linux, `osascript` on macOS).
- **`site`:** Automatic redirection to official download portals for complex interactive software.

### Stage 5: Shim Generation & System PATH
`PatherVisitor` writes proxy wrapper scripts to `shims/` and ensures the shim folder is present in user environment variables without administrative prompts.

### Stage 6: Atomic State Registration
The completed installation is committed to `installed.json` with installation path, binaries list, shim list, timestamp, and canonical manifest hash.
