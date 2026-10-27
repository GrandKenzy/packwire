# System Data Plan & Privacy

Packwire includes a hardware diagnostic collector (`GuiBridge._collect_windows_diagnostics`) used to inform users about machine compatibility for toolchains.

---

## 1. Evaluated Metrics

- **Operating System:** Name, version string, build number, architecture (64-bit / ARM).
- **Processor (CPU):** Manufacturer model, physical cores, logical threads, clock speed, virtualization flags.
- **Memory (RAM):** Total installed physical memory and available free memory.
- **Graphics (GPU):** Video adapter names, driver versions, and dedicated VRAM.
- **Firmware & Security:** BIOS release date, UEFI mode confirmation, TPM readiness, and Secure Boot status.
- **Storage Volumes:** Local drive letters, filesystem types (NTFS/ReFS), capacity, and free space.

---

## 2. Privacy Guarantees

- **100% Local Execution:** Diagnosed metrics are evaluated via standard Windows CIM/WMI cmdlets and kept entirely in memory on your computer.
- **Zero Remote Telemetry:** Packwire transmits **no data** to external servers, cloud services, or third parties.
- **No Personal Identifiers:** Usernames, computer network names, serial numbers, IP addresses, and file contents are strictly omitted.
