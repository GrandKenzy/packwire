# Diagnostics & Error Catalog

Comprehensive troubleshooting matrix of standard error conditions, root causes, and recommended solutions.

---

## 1. Error Matrix

| Error Code / Symptom | Root Cause | Mitigation |
| :--- | :--- | :--- |
| `COLLISION_DETECTED` | Another installed package owns one of the target binaries. | Use `--force` flag or uninstall conflicting package. |
| `WINERROR_225_ANTIVIRUS` | Windows Defender blocked extraction due to heuristic false positive. | Run `packwire fix-defender` or restore file from Windows Security History. |
| `MISSING_AS_EXE` | GNU Assembler (`as.exe`) in GCC distribution removed by antivirus. | Open Windows Security > Protection History > Restore `as.exe`. |
| `NETWORK_TIMEOUT` | Upstream download endpoint unreachable or timed out. | Check Internet connection; retry command (cached `.part` will resume). |
| `CHECKSUM_MISMATCH` | Downloaded file SHA-256 differs from `expected_hash`. | Re-run with `--force` to re-download or report manifest hash mismatch. |
| `UAC_ELEVATION_REJECTED` | User dismissed administrative elevation prompt. | Re-run command and approve UAC dialog. |
