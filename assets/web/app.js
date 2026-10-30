// API Abstraction layer: supports both pywebview.api (Desktop) and fetch('/api/...') (Web Server)
const API = {
  async call(method, ...args) {
    if (window.pywebview && window.pywebview.api) {
      return await window.pywebview.api[method](...args);
    }
    // Fallback to HTTP REST endpoint
    try {
      const res = await fetch(`/api/${method}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ args: args })
      });
      return await res.json();
    } catch (e) {
      console.error(`API call failed for ${method}:`, e);
      return { success: false, message: e.toString() };
    }
  },

  getSystemInfo() { return this.call("get_system_info"); },
  listAvailable() { return this.call("list_available"); },
  listInstalled() { return this.call("list_installed"); },
  listSystemApps() { return this.call("list_system_apps"); },
  install(id, mode, version) { return this.call("install", id, mode, version); },
  reinstall(id) { return this.call("reinstall", id); },
  uninstall(id) { return this.call("uninstall", id); },
  update(id) { return this.call("update", id); },
  checkUpdates() { return this.call("check_updates"); },
  getCatalogSync() { return this.call("get_catalog_sync"); },
  checkAppUpdate() { return this.call("check_app_update"); },
  listThemes() { return this.call("list_themes"); },
  getConfig(key) { return this.call("get_config", key); },
  setConfig(key, value) { return this.call("set_config", key, value); },
  queueInstall(id, mode, version, force = false) { return this.call("queue_install", id, mode, version, force); },
  getJob(jobId) { return this.call("get_job", jobId); },
  listJobs() { return this.call("list_jobs"); },
  addPath() { return this.call("add_path"); },
  uninstallAll(removePath = true) { return this.call("uninstall_all", removePath); },
  openSite(url) { return this.call("open_site", url); },
  addDefenderExclusion() { return this.call("add_defender_exclusion"); },
  openGuide(guideName) { return this.call("open_guide", guideName); }
};

// Application State
let currentSystemInfo = {};
let currentFilter = "all";
let catalogSearchQuery = "";
let toolsSearchQuery = "";
let operatingSystemsSearchQuery = "";
let installedSearchQuery = "";
let availablePackages = [];
let installedPackages = {};
let systemApplications = [];
let systemAppsSupported = false;
let availableUpdates = {};

// Clean SVG Icons for Buttons (No cartoon emojis)
const ICONS = {
  download: `<svg class="btn-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>`,
  terminal: `<svg class="btn-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"></polyline><line x1="12" y1="19" x2="20" y2="19"></line></svg>`,
  refresh: `<svg class="btn-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/></svg>`,
  external: `<svg class="btn-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>`,
  trash: `<svg class="btn-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>`,
  arrowUp: `<svg class="btn-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="19" x2="12" y2="5"></line><polyline points="5 12 12 5 19 12"></polyline></svg>`,
  chevronDown: `<svg class="btn-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>`
};

// DOM Elements
const themeSelectorEl = document.getElementById("theme-selector");
const catalogCardsEl = document.getElementById("catalog-cards");
const toolsCardsEl = document.getElementById("tools-cards");
const operatingSystemsCardsEl = document.getElementById("operating-systems-cards");
const installedContainerEl = document.getElementById("installed-container");
const installedCounterEl = document.getElementById("installed-counter");
const platformTextEl = document.getElementById("platform-text");
const pathIndicatorEl = document.getElementById("path-indicator");
const pathTextEl = document.getElementById("path-text");
const btnFixPath = document.getElementById("btn-fix-path");
const toastContainer = document.getElementById("toast-container");

// Search DOM Elements
const catalogSearchInput = document.getElementById("catalog-search");
const btnClearCatalogSearch = document.getElementById("btn-clear-catalog-search");
const catalogSearchInfo = document.getElementById("catalog-search-info");
const toolsSearchInput = document.getElementById("tools-search");
const btnClearToolsSearch = document.getElementById("btn-clear-tools-search");
const toolsSearchInfo = document.getElementById("tools-search-info");
const operatingSystemsSearchInput = document.getElementById("operating-systems-search");
const btnClearOperatingSystemsSearch = document.getElementById("btn-clear-operating-systems-search");
const operatingSystemsSearchInfo = document.getElementById("operating-systems-search-info");

const installedSearchInput = document.getElementById("installed-search");
const btnClearInstalledSearch = document.getElementById("btn-clear-installed-search");
const installedSearchInfo = document.getElementById("installed-search-info");

// Modal Elements
const modalOverlay = document.getElementById("action-modal");
const modalTitle = document.getElementById("modal-title");
const modalDesc = document.getElementById("modal-desc");
const modalStatusText = document.getElementById("modal-status-text");
const modalProgressTrack = document.getElementById("modal-progress-track");
const modalTerminalBox = document.getElementById("modal-terminal-box");
const modalTerminalLogs = document.getElementById("modal-terminal-logs");
const terminalLivePill = document.getElementById("terminal-live-pill");
const modalFooter = document.getElementById("modal-footer");
const btnModalDone = document.getElementById("btn-modal-done");
const btnModalClose = document.getElementById("btn-modal-close");
const btnModalCancel = document.getElementById("btn-modal-cancel");
const btnModalConfirm = document.getElementById("btn-modal-confirm");

// Initialization
document.addEventListener("DOMContentLoaded", async () => {
  // If in pywebview, wait for ready event
  if (window.pywebview) {
    window.addEventListener("pywebviewready", initApp);
  } else {
    setTimeout(initApp, 100);
  }

  setupEventListeners();
  setupGuideModalListeners();
});

async function initApp() {
  await setupTheme();
  await loadSystemInfo();
  await refreshCatalog();
  await refreshInstalled();

  const catalogSync = await API.getCatalogSync();
  if (catalogSync && catalogSync.success && catalogSync.updated) {
    showToast(catalogSync.message, "info");
  }
  
  // Detect updates in background as soon as app starts
  checkUpdatesInBackground();
  checkAppUpdateInBackground();
}

async function checkAppUpdateInBackground() {
  try {
    const update = await API.checkAppUpdate();
    if (update && update.success && update.update_available) {
      showAppUpdateToast(update);
    }
  } catch (err) {
    // Update checks are optional and must never make the application noisy offline.
    console.warn("App update check failed:", err);
  }
}

async function checkUpdatesInBackground() {
  try {
    const updates = await API.checkUpdates() || {};
    availableUpdates = updates;
    const updateCount = Object.keys(updates).length;

    const installedTab = document.querySelector(".nav-tab[data-tab='installed']");
    const existingDot = document.getElementById("update-indicator-dot");

    if (updateCount > 0) {
      if (installedTab && !existingDot) {
        const dot = document.createElement("span");
        dot.id = "update-indicator-dot";
        dot.className = "update-pulse-dot";
        dot.title = `${updateCount} actualización(es) disponible(s)`;
        installedTab.appendChild(dot);
      }
      const names = Object.values(updates).map(u => `${u.name} (v${u.latest_version})`).join(", ");
      showToast(`Actualizaciones disponibles: ${names}`, "info");
    } else if (existingDot) {
      existingDot.remove();
    }

    renderInstalledList();
  } catch (err) {
    console.warn("Background update check failed:", err);
  }
}

async function setupTheme() {
  const themes = await API.listThemes() || ["default", "dark"];
  if (themeSelectorEl) {
    themeSelectorEl.innerHTML = "";
    themes.forEach(t => {
      const opt = document.createElement("option");
      opt.value = t;
      const displayMap = { "default": "Claro (Default)", "dark": "Oscuro (Dark)" };
      opt.textContent = displayMap[t] || (t.charAt(0).toUpperCase() + t.slice(1));
      themeSelectorEl.appendChild(opt);
    });

    let savedTheme = "default";
    try {
      const configTheme = await API.getConfig("theme");
      if (configTheme && typeof configTheme === "string") {
        savedTheme = configTheme;
      } else {
        savedTheme = localStorage.getItem("packwire_theme") || "default";
      }
    } catch (e) {
      savedTheme = localStorage.getItem("packwire_theme") || "default";
    }

    themeSelectorEl.value = savedTheme;
    localStorage.setItem("packwire_theme", savedTheme);
    applyTheme(savedTheme);

    themeSelectorEl.addEventListener("change", async (e) => {
      const selected = e.target.value;
      localStorage.setItem("packwire_theme", selected);
      applyTheme(selected);
      try {
        await API.setConfig("theme", selected);
      } catch (err) {
        console.warn("Could not save theme to config:", err);
      }
      showToast(`Tema guardado en configs.json: ${selected}`, "info");
    });
  }
}

function applyTheme(themeName) {
  const bgLink = document.getElementById("theme-style-background");
  const cardsLink = document.getElementById("theme-style-cards");
  const buttonsLink = document.getElementById("theme-style-buttons");

  if (bgLink) {
    bgLink.onerror = () => { bgLink.href = "global/default_background.css"; };
    bgLink.href = `global/${themeName}_background.css`;
  }
  if (cardsLink) {
    cardsLink.onerror = () => { cardsLink.href = "global/default_cards.css"; };
    cardsLink.href = `global/${themeName}_cards.css`;
  }
  if (buttonsLink) {
    buttonsLink.onerror = () => { buttonsLink.href = "global/default_buttons.css"; };
    buttonsLink.href = `global/${themeName}_buttons.css`;
  }
}

function setupEventListeners() {
  // Navigation Tabs
  document.querySelectorAll(".nav-tab").forEach(tab => {
    tab.addEventListener("click", () => {
      document.querySelectorAll(".nav-tab").forEach(t => t.classList.remove("active"));
      document.querySelectorAll(".tab-pane").forEach(p => p.classList.remove("active"));
      tab.classList.add("active");
      const target = document.getElementById(`tab-${tab.dataset.tab}`);
      if (target) target.classList.add("active");
      if (tab.dataset.tab === "tools") renderTools();
      if (tab.dataset.tab === "operating-systems") renderOperatingSystems();
    });
  });

  // Filters
  document.querySelectorAll(".filter-btn").forEach(btn => {
    btn.addEventListener("click", () => {
      document.querySelectorAll(".filter-btn").forEach(b => b.classList.remove("active"));
      btn.classList.add("active");
      currentFilter = btn.dataset.filter;
      renderCatalog();
    });
  });

  // Refresh installed button
  document.getElementById("btn-refresh-installed").addEventListener("click", refreshInstalled);

  // Fix path button
  const btnFixDefender = document.getElementById("btn-fix-defender");
  if (btnFixDefender) {
    btnFixDefender.addEventListener("click", async () => {
      btnFixDefender.disabled = true;
      btnFixDefender.textContent = "Configurando...";
      try {
        const res = await API.addDefenderExclusion();
        if (res && res.success) {
          showToast(res.message || "Exclusión enviada a Windows Defender.", "success");
        } else {
          showToast(res && res.message ? res.message : "No se pudo configurar la exclusión.", "warning");
        }
      } catch (err) {
        showToast("Error: " + err, "error");
      } finally {
        btnFixDefender.disabled = false;
        btnFixDefender.innerHTML = `
          <svg class="btn-icon" viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path><path d="M9 12l2 2 4-4"></path></svg>
          Añadir Exclusión en Defender
        `;
      }
    });
  }

  btnFixPath.addEventListener("click", async () => {
    btnFixPath.disabled = true;
    const res = await API.addPath();
    if (res.success) {
      showToast(res.message, "success");
      await loadSystemInfo();
    } else {
      showToast(res.message, "error");
    }
    btnFixPath.disabled = false;
  });

  const btnRefreshSystem = document.getElementById("btn-refresh-system");
  if (btnRefreshSystem) {
    btnRefreshSystem.addEventListener("click", async () => {
      btnRefreshSystem.disabled = true;
      try {
        await loadSystemInfo();
        showToast("Diagnóstico del sistema actualizado.", "success");
      } catch (err) {
        showToast("No se pudo actualizar el diagnóstico: " + err, "error");
      } finally {
        btnRefreshSystem.disabled = false;
      }
    });
  }

  // Modal close buttons
  btnModalDone.addEventListener("click", () => {
    modalOverlay.style.display = "none";
    resetModalButtons();
  });
  btnModalClose.addEventListener("click", () => {
    modalOverlay.style.display = "none";
    resetModalButtons();
  });

  // There is one destructive uninstall entry point in the System tab.
  const btnUninstallAllCard = document.getElementById("btn-uninstall-all-card");
  if (btnUninstallAllCard) {
    btnUninstallAllCard.addEventListener("click", startUninstallAll);
  }

  // Catalog search input listeners
  if (catalogSearchInput) {
    catalogSearchInput.addEventListener("input", (e) => {
      catalogSearchQuery = e.target.value;
      renderCatalog();
    });
    catalogSearchInput.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        catalogSearchInput.value = "";
        catalogSearchQuery = "";
        renderCatalog();
      }
    });
  }

  if (btnClearCatalogSearch) {
    btnClearCatalogSearch.addEventListener("click", () => {
      catalogSearchInput.value = "";
      catalogSearchQuery = "";
      catalogSearchInput.focus();
      renderCatalog();
    });
  }

  // Tools search input listeners
  if (toolsSearchInput) {
    toolsSearchInput.addEventListener("input", (e) => {
      toolsSearchQuery = e.target.value;
      renderTools();
    });
    toolsSearchInput.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        toolsSearchInput.value = "";
        toolsSearchQuery = "";
        renderTools();
      }
    });
  }

  if (btnClearToolsSearch) {
    btnClearToolsSearch.addEventListener("click", () => {
      toolsSearchInput.value = "";
      toolsSearchQuery = "";
      toolsSearchInput.focus();
      renderTools();
    });
  }

  // Operating systems search input listeners
  if (operatingSystemsSearchInput) {
    operatingSystemsSearchInput.addEventListener("input", (e) => {
      operatingSystemsSearchQuery = e.target.value;
      renderOperatingSystems();
    });
    operatingSystemsSearchInput.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        operatingSystemsSearchInput.value = "";
        operatingSystemsSearchQuery = "";
        renderOperatingSystems();
      }
    });
  }

  if (btnClearOperatingSystemsSearch) {
    btnClearOperatingSystemsSearch.addEventListener("click", () => {
      operatingSystemsSearchInput.value = "";
      operatingSystemsSearchQuery = "";
      operatingSystemsSearchInput.focus();
      renderOperatingSystems();
    });
  }

  // Installed search input listeners
  if (installedSearchInput) {
    installedSearchInput.addEventListener("input", (e) => {
      installedSearchQuery = e.target.value;
      renderInstalledList();
    });
    installedSearchInput.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        installedSearchInput.value = "";
        installedSearchQuery = "";
        renderInstalledList();
      }
    });
  }

  if (btnClearInstalledSearch) {
    btnClearInstalledSearch.addEventListener("click", () => {
      installedSearchInput.value = "";
      installedSearchQuery = "";
      installedSearchInput.focus();
      renderInstalledList();
    });
  }

  // Global hotkey: '/' to focus search in Catalog
  window.addEventListener("keydown", (e) => {
    if (e.key === "/" && document.activeElement !== catalogSearchInput && document.activeElement !== toolsSearchInput && document.activeElement !== installedSearchInput) {
      const activeTab = document.querySelector(".nav-tab.active");
      if (activeTab && activeTab.dataset.tab === "catalog" && catalogSearchInput) {
        e.preventDefault();
        catalogSearchInput.focus();
      } else if (activeTab && activeTab.dataset.tab === "tools" && toolsSearchInput) {
        e.preventDefault();
        toolsSearchInput.focus();
      } else if (activeTab && activeTab.dataset.tab === "operating-systems" && operatingSystemsSearchInput) {
        e.preventDefault();
        operatingSystemsSearchInput.focus();
      }
    }
  });
}

function setSystemValue(id, value, fallback = "No disponible") {
  const element = document.getElementById(id);
  if (element) element.textContent = (value === null || value === undefined || value === "") ? fallback : String(value);
}

function formatGib(value) {
  return (typeof value === "number" && Number.isFinite(value)) ? `${value.toFixed(1)} GiB` : "No disponible";
}

function renderSystemDiagnostics(hardware) {
  const computer = hardware.computer || {};
  const cpu = hardware.cpu || {};
  const firmware = hardware.firmware || {};
  const security = hardware.security || {};
  const os = hardware.os || {};

  setSystemValue("sys-device", [computer.manufacturer, computer.model].filter(Boolean).join(" · "));
  setSystemValue("sys-memory", `${formatGib(computer.memory_gb)} · libres: ${formatGib(computer.memory_free_gb)}`);
  setSystemValue("sys-cpu", cpu.name);
  setSystemValue("sys-cpu-details", [
    cpu.cores ? `${cpu.cores} núcleos` : null,
    cpu.logical_processors ? `${cpu.logical_processors} hilos` : null,
    cpu.max_clock_mhz ? `hasta ${cpu.max_clock_mhz} MHz` : null,
    cpu.virtualization_firmware === true ? "virtualización firmware: sí" : (cpu.virtualization_firmware === false ? "virtualización firmware: no" : null)
  ].filter(Boolean).join(" · "));

  const gpuLines = (hardware.gpus || []).map(gpu => {
    const details = [gpu.memory_gb ? `${gpu.memory_gb} GiB` : null, gpu.driver ? `driver ${gpu.driver}` : null].filter(Boolean).join(" · ");
    return details ? `${gpu.name} (${details})` : gpu.name;
  }).filter(Boolean);
  setSystemValue("sys-gpu", gpuLines.join("\n"));

  const volumeLines = (hardware.volumes || []).map(volume => {
    const free = volume.free_gb !== undefined ? `libres ${volume.free_gb} GiB` : "espacio libre no disponible";
    return `${volume.drive || "Volumen"}: ${volume.size_gb || "?"} GiB · ${free}${volume.filesystem ? ` · ${volume.filesystem}` : ""}`;
  });
  setSystemValue("sys-storage", volumeLines.join("\n"));

  setSystemValue("sys-firmware", [firmware.mode, firmware.bios_vendor].filter(Boolean).join(" · "));
  setSystemValue("sys-bios", [firmware.bios_version, firmware.bios_date].filter(Boolean).join(" · "));
  const secureBoot = security.secure_boot === true ? "Secure Boot: activado" : (security.secure_boot === false ? "Secure Boot: desactivado" : "Secure Boot: no disponible");
  const tpm = security.tpm_present === true ? `TPM: presente${security.tpm_spec ? ` (${security.tpm_spec})` : ""}${security.tpm_ready === true ? ", listo" : ""}` : (security.tpm_present === false ? "TPM: no detectado" : "TPM: no disponible");
  const hypervisor = computer.hypervisor_present === true ? "Hipervisor activo" : "Hipervisor no detectado";
  setSystemValue("sys-security", `${secureBoot} · ${tpm} · ${hypervisor}`);

  if (typeof hardware.uptime_hours === "number") {
    const days = Math.floor(hardware.uptime_hours / 24);
    const hours = Math.round(hardware.uptime_hours % 24);
    setSystemValue("sys-uptime", days ? `${days} d ${hours} h` : `${hours} h`);
  } else {
    setSystemValue("sys-uptime", null);
  }
  const networkLines = (hardware.network || []).map(adapter => `${adapter.name || "Adaptador"}${adapter.link_speed ? ` · ${adapter.link_speed}` : ""}`);
  setSystemValue("sys-network", networkLines.join("\n"));

  if (os.architecture) setSystemValue("sys-arch", os.architecture);
}

async function loadSystemInfo() {
  const info = await API.getSystemInfo();
  currentSystemInfo = info || {};
  const versionTag = document.querySelector(".version-tag");
  if (versionTag && info && info.version) {
    versionTag.textContent = "v" + info.version;
  }
  if (!info) return;

  const osMap = { windows: "Windows", darwin: "macOS", linux: "Linux" };
  const hardware = info.hardware || {};
  const hardwareOs = hardware.os || {};
  const osName = [hardwareOs.caption || osMap[info.os] || info.os, hardwareOs.display_version].filter(Boolean).join(" ");
  platformTextEl.textContent = `${osName} (${info.arch})`;

  if (info.shims_in_path) {
    pathIndicatorEl.className = "path-indicator ok";
    pathTextEl.textContent = "PATH Configurado";
    btnFixPath.style.display = "none";
  } else {
    pathIndicatorEl.className = "path-indicator warn";
    pathTextEl.textContent = "Shims fuera del PATH";
    btnFixPath.style.display = "inline-flex";
  }

  // System Tab Details
  document.getElementById("sys-os").textContent = osName;
  document.getElementById("sys-arch").textContent = info.arch;
  document.getElementById("sys-root").textContent = info.root_dir;
  document.getElementById("sys-shims").textContent = info.shims_dir;
  document.getElementById("sys-apps").textContent = info.apps_dir;
  renderSystemDiagnostics(hardware);

  const defenderCard = document.getElementById("defender-exclusion-card");
  if (defenderCard) {
    defenderCard.style.display = (info.os === "windows") ? "block" : "none";
  }
}

async function refreshCatalog() {
  catalogCardsEl.innerHTML = `<div class="skeleton-card">Consultando catálogo oficial...</div>`;
  availablePackages = await API.listAvailable() || [];
  renderCatalog();
  renderTools();
  renderOperatingSystems();
}

const DEFAULT_PACKAGE_ICON = "default-logo.svg";

// Icons are selected from the package meaning, not randomly from a pool.
// Official image URLs remain available for packages that publish their own logo.
function semanticPackageIcon(pkg) {
  const name = String((pkg && pkg.name) || "").toLowerCase();
  const category = String((pkg && pkg.category) || "").toLowerCase();
  const configuredIcon = String((pkg && pkg.icon) || "");
  const isOfficialImage = configuredIcon.startsWith("http://")
    || configuredIcon.startsWith("https://")
    || configuredIcon.startsWith("data:")
    || /\.(png|svg|ico)(\?.*)?$/i.test(configuredIcon);

  if (isOfficialImage) return configuredIcon;
  if (name.includes("python")) return "🐍";
  if (name.includes("rust")) return "🦀";
  if (name.includes("php")) return "🐘";
  if (name.includes("c#") || name.includes("csharp")) return "#️⃣";
  if (name.includes("antigravity")) return "🤖";
  if (name.includes("node.js") || name === "node" || name.includes("nodejs")) return "⚡";
  if (name.includes("powershell")) return "⌨️";
  if (name.includes("docker")) return "📦";
  if (name === "git" || name.startsWith("git ")) return "🌿";
  if (name.includes("ffprobe")) return "🔍";
  if (name.includes("ffmpeg")) return "🎬";
  if (name.includes("visual studio code") || name === "vscode") return "📝";
  if (name.includes("sdl")) return "🧩";
  if (name.includes("rufus") || name.includes("ventoy")) return "💾";
  if (name.includes("linux") || category === "linux") return "🖥️";

  if (category === "package managers") return "📦";
  if (category === "data languages") return "📄";
  if (category === "programming languages") return "💻";
  if (category === "compilers") return "⚙️";
  if (category === "tools") return "🛠️";
  if (category === "development tools") return "🛠️";
  if (category === "multimedia") return "🎬";
  if (category === "runtimes") return "⚡";
  if (category === "shells") return "⌨️";
  if (category === "system tools") return "🧰";
  if (category === "editors") return "📝";
  if (category === "libraries") return "🧩";
  if (category === "ai & development") return "🤖";
  return "📦";
}

function renderIconHtml(icon) {
  if (!icon) {
    return `<div class="pkg-icon-container"><img src="${DEFAULT_PACKAGE_ICON}" class="pkg-icon-img" alt="Packwire default icon"></div>`;
  }
  const isUrl = icon.startsWith("http://") || icon.startsWith("https://") || icon.startsWith("data:") || icon.endsWith(".png") || icon.endsWith(".svg") || icon.endsWith(".ico");
  if (isUrl) {
    const safeIcon = escapeHtml(icon);
    return `<div class="pkg-icon-container"><img src="${safeIcon}" class="pkg-icon-img" alt="package icon" onerror="this.onerror=null;this.src='${DEFAULT_PACKAGE_ICON}'"></div>`;
  }
  return `<div class="pkg-icon-container"><span class="pkg-icon-emoji">${icon}</span></div>`;
}

function formatPackageVersion(pkg) {
  const version = String((pkg && pkg.version) || "").trim();
  if (!version || version.toLowerCase() === "latest") return "latest";
  return version.toLowerCase().startsWith("v") ? version : `v${version}`;
}

function packageVersionOptions(pkg) {
  const family = String((pkg && pkg.name) || "").trim().toLowerCase();
  const variants = availablePackages.filter(candidate =>
    String(candidate.name || "").trim().toLowerCase() === family
  );
  const options = new Map();

  variants.forEach(candidate => {
    const version = String(candidate.version || "").trim();
    if (candidate.type === "stable" || !version || version.toLowerCase() === "latest") {
      options.set("", { value: "", label: "latest" });
    } else {
      options.set(version, { value: version, label: formatPackageVersion(candidate) });
    }
  });

  if (pkg.type === "fixed" && pkg.version) {
    const version = String(pkg.version).trim();
    options.set(version, { value: version, label: formatPackageVersion(pkg) });
  }

  return Array.from(options.values()).sort((left, right) => {
    if (left.value === "") return -1;
    if (right.value === "") return 1;
    return right.value.localeCompare(left.value, undefined, { numeric: true });
  });
}

function packageFamilyKey(pkg) {
  const name = String((pkg && pkg.name) || "").trim().toLowerCase();
  if (name) return name;
  return String((pkg && pkg.id) || "").split("@")[0].trim().toLowerCase();
}

function preferPackageVariant(candidate, current) {
  if (!current) return true;
  if (candidate.type === "stable" && current.type !== "stable") return true;
  if (candidate.type !== "stable" && current.type === "stable") return false;

  const candidateVersion = String(candidate.version || "");
  const currentVersion = String(current.version || "");
  return candidateVersion.localeCompare(currentVersion, undefined, { numeric: true }) > 0;
}

function collapsePackageVariants(packages) {
  const representatives = new Map();
  packages.forEach(pkg => {
    const key = packageFamilyKey(pkg);
    if (preferPackageVariant(pkg, representatives.get(key))) {
      representatives.set(key, pkg);
    }
  });
  return Array.from(representatives.values());
}

function escapeHtml(str) {
  if (!str) return "";
  return String(str).replace(/[&<>"']/g, function(m) {
    return {
      "&": "&amp;",
      "<": "&lt;",
      ">": "&gt;",
      '"': "&quot;",
      "'": "&#39;"
    }[m];
  });
}

function renderCatalog(section = "catalog") {
  const isTools = section === "tools";
  const isOperatingSystems = section === "operating-systems";
  const cardsEl = isTools ? toolsCardsEl : (isOperatingSystems ? operatingSystemsCardsEl : catalogCardsEl);
  const searchInput = isTools ? toolsSearchInput : (isOperatingSystems ? operatingSystemsSearchInput : catalogSearchInput);
  const clearSearch = isTools ? btnClearToolsSearch : (isOperatingSystems ? btnClearOperatingSystemsSearch : btnClearCatalogSearch);
  const searchInfo = isTools ? toolsSearchInfo : (isOperatingSystems ? operatingSystemsSearchInfo : catalogSearchInfo);
  const searchQueryValue = isTools ? toolsSearchQuery : (isOperatingSystems ? operatingSystemsSearchQuery : catalogSearchQuery);
  const resetSearchId = isTools ? "btn-reset-tools-search" : (isOperatingSystems ? "btn-reset-operating-systems-search" : "btn-reset-catalog-search");
  cardsEl.innerHTML = "";
  const query = searchQueryValue.trim().toLowerCase();

  const isDedicatedTool = pkg => pkg.id === "antigravity" || pkg.id === "vscode";
  const isOperatingSystem = pkg => (pkg.category || "").toLowerCase() === "linux";
  const sourcePackages = isTools
    ? availablePackages.filter(isDedicatedTool)
    : isOperatingSystems
      ? availablePackages.filter(isOperatingSystem)
      : availablePackages.filter(pkg => !isDedicatedTool(pkg) && !isOperatingSystem(pkg));
  const filtered = sourcePackages.filter(pkg => {
    // 1. Channel/catalog filter
    if (isTools) {
      if ((pkg.category || "").toLowerCase() !== "tools") return false;
    } else if (!isOperatingSystems && currentFilter !== "all" && pkg.type !== currentFilter) {
      return false;
    }

    // 2. Search query filter
    if (query) {
      const matchName = pkg.name && pkg.name.toLowerCase().includes(query);
      const matchId = pkg.id && pkg.id.toLowerCase().includes(query);
      const matchDesc = pkg.description && pkg.description.toLowerCase().includes(query);
      const matchCat = pkg.category && pkg.category.toLowerCase().includes(query);
      const matchAliases = Array.isArray(pkg.aliases) && pkg.aliases.some(a => a.toLowerCase().includes(query));
      return matchName || matchId || matchDesc || matchCat || matchAliases;
    }

    return true;
  });
  const visiblePackages = collapsePackageVariants(filtered);

  // Update search UI info
  if (searchInfo) {
    if (query) {
      searchInfo.style.display = "flex";
      searchInfo.innerHTML = `
        <span>Mostrando <strong>${visiblePackages.length}</strong> de <strong>${sourcePackages.length}</strong> paquetes para "<strong>${escapeHtml(searchQueryValue)}</strong>"</span>
        <button class="btn-reset-search" id="${resetSearchId}">Limpiar filtro</button>
      `;
      const btnReset = document.getElementById(resetSearchId);
      if (btnReset) {
        btnReset.addEventListener("click", () => {
          if (searchInput) searchInput.value = "";
          if (isTools) toolsSearchQuery = "";
          else if (isOperatingSystems) operatingSystemsSearchQuery = "";
          else catalogSearchQuery = "";
          if (clearSearch) clearSearch.style.display = "none";
          renderCatalog(section);
        });
      }
    } else {
      searchInfo.style.display = "none";
    }
  }

  if (clearSearch) {
    clearSearch.style.display = query ? "block" : "none";
  }

  if (visiblePackages.length === 0) {
    if (query) {
      cardsEl.innerHTML = `
        <div class="info-card" style="grid-column: 1 / -1; text-align: center; padding: 40px 20px;">
          <div style="display: flex; justify-content: center; margin-bottom: 10px; color: var(--text-muted);">
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
          </div>
          <span class="info-label" style="font-size: 14px;">No se encontraron paquetes</span>
          <p class="subtitle" style="margin-top: 6px;">No hay resultados para "<strong>${escapeHtml(searchQueryValue)}</strong>". Prueba con otra búsqueda o limpia el buscador.</p>
        </div>
      `;
    } else {
      cardsEl.innerHTML = `<p class="subtitle">No hay paquetes que coincidan con este filtro.</p>`;
    }
    return;
  }

  visiblePackages.forEach(pkg => {
    const card = document.createElement("div");
    card.className = "package-card";

    const isStable = pkg.type === "stable";
    const typeLabel = isStable ? "latest" : "fixed";
    const typeClass = isStable ? "stable compact" : "fixed";

    const isCommand = !!(pkg.install && (pkg.install.mode === "command" || pkg.install.command));
    const hasHere = !!(pkg.install && pkg.install.here && (pkg.install.here.url || pkg.type === "stable"));
    const hasSite = !!(pkg.install && pkg.install.site && pkg.install.site.url);
    const isSiteOnly = !hasHere && !isCommand && hasSite;
    const versionOptions = packageVersionOptions(pkg);
    const hasVersionSelector = versionOptions.length > 1;
    const installButtonClass = isSiteOnly ? "btn-install-site" : "btn-install-here";
    const installIcon = isSiteOnly ? ICONS.external : (isCommand ? ICONS.terminal : ICONS.download);
    const installButton = `
      <button class="btn btn-primary ${installButtonClass}" data-id="${pkg.id}">
        ${installIcon} Instalar
      </button>
    `;
    const versionMenu = hasVersionSelector ? `
      <button class="btn btn-primary btn-version-toggle" type="button" aria-label="Seleccionar versión" title="Seleccionar versión">
        ${ICONS.chevronDown}
      </button>
      <div class="install-version-menu">
        ${versionOptions.map(option => `
          <button class="install-version-option" type="button" data-version="${escapeHtml(option.value)}">
            ${escapeHtml(option.label)}
          </button>
        `).join("")}
      </div>
    ` : "";
    const installActionHtml = hasVersionSelector
      ? `<div class="install-split">${installButton}${versionMenu}</div>`
      : installButton;

    const actionsHtml = `
      ${installActionHtml}
      ${hasSite && !isSiteOnly ? `
        <button class="btn btn-outline btn-site-secondary btn-install-site" data-id="${pkg.id}" title="Abrir sitio oficial">
          ${ICONS.external} Sitio
        </button>` : ''}
    `;

    const isGccPkg = (pkg.id === 'c-gcc' || pkg.id === 'gcc' || (Array.isArray(pkg.aliases) && pkg.aliases.includes('gcc')));
    const isWindowsEnv = (currentSystemInfo.is_windows_11 || currentSystemInfo.is_windows || currentSystemInfo.os === 'windows' || navigator.userAgent.includes("Windows") || navigator.platform.includes("Win"));

    const gccBannerHtml = (isGccPkg && isWindowsEnv) ? `
      <div class="card-gcc-guide" data-guide="gcc_windows11.html" title="Windows 11: Solución para falso positivo en as.exe">
        <span class="guide-banner-icon">🛡️</span>
        <span class="guide-banner-text"><strong>Guía Windows 11:</strong> Falso positivo en <code>as.exe</code></span>
        <button class="btn-guide-pill" type="button">Ver Guía →</button>
      </div>
    ` : '';

    const gccQuestionBtn = (isGccPkg && isWindowsEnv) ? `
      <button class="btn-guide-help" data-guide="gcc_windows11.html" title="Windows 11: Guía para resolver falso positivo de Windows Defender en as.exe" aria-label="Ayuda para Windows 11">?</button>
    ` : '';

    card.innerHTML = `
      <div>
        <div class="card-top">
          <div class="pkg-title-area">
            ${renderIconHtml(semanticPackageIcon(pkg))}
            <div style="flex: 1; min-width: 0;">
              <div style="display: flex; align-items: center;">
                <div class="pkg-name">${pkg.name}</div>
                ${gccQuestionBtn}
              </div>
              <div class="pkg-version">${formatPackageVersion(pkg)}</div>
            </div>
          </div>
          <span class="pkg-type-pill ${typeClass}">${typeLabel}</span>
        </div>
        <p class="pkg-desc">${pkg.description || 'Paquete de desarrollo oficial listo para instalar.'}</p>
        ${gccBannerHtml}
      </div>
      <div class="card-actions">
        ${actionsHtml}
      </div>
    `;

    // Action listeners
    const btnHere = card.querySelector(".btn-install-here");
    if (btnHere) {
      // El botón principal siempre representa la instalación estable más reciente.
      btnHere.addEventListener("click", () => startInstall(pkg, "here", ""));
    }
    const btnSite = card.querySelector(".btn-install-site");
    if (btnSite) {
      btnSite.addEventListener("click", () => startInstall(pkg, "site"));
    }

    const versionToggle = card.querySelector(".btn-version-toggle");
    const versionMenuElement = card.querySelector(".install-version-menu");
    if (versionToggle && versionMenuElement) {
      versionToggle.addEventListener("click", (event) => {
        event.stopPropagation();
        document.querySelectorAll(".install-version-menu.open").forEach(menu => {
          if (menu !== versionMenuElement) menu.classList.remove("open");
        });
        versionMenuElement.classList.toggle("open");
      });
      versionMenuElement.querySelectorAll(".install-version-option").forEach(option => {
        option.addEventListener("click", () => {
          versionMenuElement.classList.remove("open");
          const selectedVersion = option.dataset.version;
          startInstall(pkg, isSiteOnly ? "site" : "here", selectedVersion);
        });
      });
    }

    const guideElements = card.querySelectorAll(".btn-guide-help, .card-gcc-guide, .btn-guide-pill");
    guideElements.forEach(el => {
      el.addEventListener("click", (e) => {
        e.stopPropagation();
        openGuideModal("gcc_windows11.html");
      });
    });

    cardsEl.appendChild(card);
  });
}

function renderTools() {
  renderCatalog("tools");
}

function renderOperatingSystems() {
  renderCatalog("operating-systems");
}

async function refreshInstalled() {
  const [packwireResult, systemResult] = await Promise.all([
    API.listInstalled(),
    API.listSystemApps()
  ]);
  installedPackages = packwireResult || {};
  systemApplications = systemResult && Array.isArray(systemResult.apps) ? systemResult.apps : [];
  systemAppsSupported = !!(systemResult && systemResult.supported);
  renderInstalledList();
}

function renderInstalledList() {
  const query = installedSearchQuery.trim().toLowerCase();
  const allKeys = Object.keys(installedPackages);
  installedCounterEl.textContent = allKeys.length;

  installedContainerEl.innerHTML = `
    <section class="application-zone">
      <div class="application-zone-header">
        <div>
          <h3>Instaladas en Packwire</h3>
          <p class="subtitle">Aplicaciones administradas por Packwire y que pueden desinstalarse desde aquí.</p>
        </div>
        <span class="counter-badge">${allKeys.length}</span>
      </div>
      <div class="installed-list" id="packwire-installed-list"></div>
    </section>
    <section class="application-zone">
      <div class="application-zone-header">
        <div>
          <h3>Aplicaciones del sistema</h3>
          <p class="subtitle">Inventario detectado mediante las fuentes de aplicaciones de Windows. Esta zona es informativa.</p>
        </div>
        <span class="counter-badge" id="system-applications-counter">0</span>
      </div>
      <div class="system-app-list" id="system-applications-list"></div>
    </section>
  `;
  const packwireContainerEl = document.getElementById("packwire-installed-list");
  const systemApplicationsContainerEl = document.getElementById("system-applications-list");

  let filteredKeys = allKeys;
  if (query) {
    filteredKeys = allKeys.filter(pkgId => {
      const item = installedPackages[pkgId];
      const matchName = item.name && item.name.toLowerCase().includes(query);
      const matchId = item.id && item.id.toLowerCase().includes(query);
      const matchShims = Array.isArray(item.shims) && item.shims.some(s => s.toLowerCase().includes(query));
      return matchName || matchId || matchShims;
    });
  }

  // Update installed search UI info
  if (installedSearchInfo) {
    if (query) {
      installedSearchInfo.style.display = "flex";
      installedSearchInfo.innerHTML = `
        <span>Mostrando <strong>${filteredKeys.length}</strong> de <strong>${allKeys.length}</strong> aplicaciones de Packwire para "<strong>${escapeHtml(installedSearchQuery)}</strong>"</span>
        <button class="btn-reset-search" id="btn-reset-installed-search">Limpiar filtro</button>
      `;
      const btnReset = document.getElementById("btn-reset-installed-search");
      if (btnReset) {
        btnReset.addEventListener("click", () => {
          if (installedSearchInput) installedSearchInput.value = "";
          installedSearchQuery = "";
          if (btnClearInstalledSearch) btnClearInstalledSearch.style.display = "none";
          renderInstalledList();
        });
      }
    } else {
      installedSearchInfo.style.display = "none";
    }
  }

  if (btnClearInstalledSearch) {
    btnClearInstalledSearch.style.display = query ? "block" : "none";
  }

  if (allKeys.length === 0) {
    packwireContainerEl.innerHTML = `
      <div class="info-card">
        <span class="info-label">Sin paquetes activos</span>
        <p class="subtitle">Aún no has instalado ninguna aplicación con Packwire. Explora Paquetes para comenzar.</p>
      </div>
    `;
    renderSystemApplications(systemApplicationsContainerEl, query);
    return;
  }

  if (filteredKeys.length === 0) {
    packwireContainerEl.innerHTML = `
      <div class="info-card" style="text-align: center; padding: 36px 20px;">
        <div style="display: flex; justify-content: center; margin-bottom: 8px; color: var(--text-muted);">
          <svg viewBox="0 0 24 24" width="26" height="26" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
        </div>
        <span class="info-label">Sin coincidencias</span>
        <p class="subtitle">No se encontró ningún paquete instalado que coincida con "<strong>${escapeHtml(installedSearchQuery)}</strong>".</p>
      </div>
    `;
    renderSystemApplications(systemApplicationsContainerEl, query);
    return;
  }

  filteredKeys.forEach(pkgId => {
    const item = installedPackages[pkgId];
    const itemEl = document.createElement("div");
    itemEl.className = "installed-item";

    const shimsHtml = (item.shims || []).map(s => `<span class="shim-tag">${s}</span>`).join(" ");
    const iconHtml = renderIconHtml(semanticPackageIcon(item));

    const updateInfo = availableUpdates[item.id];
    const hasUpdate = !!updateInfo;
    const updateBadgeHtml = hasUpdate
      ? `<span class="update-badge" title="Nueva versión disponible"><span class="update-dot-mini"></span> v${updateInfo.latest_version} disponible</span>`
      : '';

    // Non-fixed packages (e.g. stable) get the update button
    let updateBtnHtml = "";
    if (item.type !== "fixed") {
      if (hasUpdate) {
        updateBtnHtml = `<button class="btn btn-primary btn-sm btn-update" data-id="${item.id}" title="Actualizar a v${updateInfo.latest_version}">${ICONS.arrowUp} Actualizar</button>`;
      } else {
        updateBtnHtml = `<button class="btn btn-outline btn-sm btn-update" data-id="${item.id}" title="Buscar y aplicar última versión">${ICONS.refresh} Actualizar</button>`;
      }
    }

    itemEl.innerHTML = `
      <div class="installed-meta">
        ${iconHtml}
        <div class="installed-details">
          <div class="installed-title-row">
            <strong>${item.name}</strong>
            <span class="pkg-version">v${item.version}</span>
            <span class="pkg-type-pill ${item.type}">${item.type}</span>
            <span class="badge">${item.mode}</span>
            ${updateBadgeHtml}
          </div>
          <div class="installed-path">${item.install_path || 'Instalación manual (Site)'}</div>
          ${shimsHtml ? `<div class="installed-shims">${shimsHtml}</div>` : ''}
        </div>
      </div>
      <div style="display: flex; gap: 8px; align-items: center;">
        ${updateBtnHtml}
        <button class="btn btn-outline btn-sm btn-reinstall" data-id="${item.id}" title="Reinstalar paquete">
          ${ICONS.refresh} Reinstalar
        </button>
        <button class="btn btn-danger btn-sm btn-uninstall" data-id="${item.id}" title="Desinstalar">
          ${ICONS.trash} Desinstalar
        </button>
      </div>
    `;

    // Event listeners
    if (item.type !== "fixed") {
      const updateBtn = itemEl.querySelector(".btn-update");
      if (updateBtn) {
        updateBtn.addEventListener("click", () => startUpdate(item.id, item.name));
      }
    }

    itemEl.querySelector(".btn-reinstall").addEventListener("click", () => startReinstall(item.id, item.name));

    itemEl.querySelector(".btn-uninstall").addEventListener("click", async () => {
      if (confirm(`¿Desinstalar '${item.name}' (v${item.version})?`)) {
        const res = await API.uninstall(item.id);
        if (res.success) {
          showToast(res.message, "success");
          delete availableUpdates[item.id];
          await refreshInstalled();
          renderCatalog();
          checkUpdatesInBackground();
        } else {
          showToast(res.message, "error");
        }
      }
    });

    packwireContainerEl.appendChild(itemEl);
  });

  renderSystemApplications(systemApplicationsContainerEl, query);
}

function renderSystemApplications(container, query) {
  if (!container) return;

  const counter = document.getElementById("system-applications-counter");
  if (counter) counter.textContent = systemApplications.length;

  if (!systemAppsSupported) {
    container.innerHTML = `
      <div class="info-card">
        <span class="info-label">Inventario no disponible</span>
        <p class="subtitle">La detección de aplicaciones del sistema está disponible actualmente en Windows.</p>
      </div>
    `;
    return;
  }

  const filtered = query
    ? systemApplications.filter(app => {
        const values = [app.name, app.version, app.publisher, app.source];
        return values.some(value => String(value || "").toLowerCase().includes(query));
      })
    : systemApplications;

  if (filtered.length === 0) {
    container.innerHTML = `
      <div class="info-card">
        <span class="info-label">Sin coincidencias</span>
        <p class="subtitle">No se encontró ninguna aplicación del sistema para esta búsqueda.</p>
      </div>
    `;
    return;
  }

  container.innerHTML = filtered.map(app => {
    const icon = app.kind === "appx" ? "🪟" : "🖥️";
    const version = app.version ? `v${escapeHtml(app.version)}` : "Versión no indicada";
    const publisher = app.publisher ? escapeHtml(app.publisher) : "Proveedor no indicado";
    const location = app.install_location
      ? escapeHtml(app.install_location)
      : "Ubicación gestionada por Windows";
    return `
      <article class="system-app-item">
        <div class="system-app-icon" aria-hidden="true">${icon}</div>
        <div class="system-app-details">
          <div class="system-app-title-row">
            <strong>${escapeHtml(app.name || "Aplicación sin nombre")}</strong>
            <span class="pkg-version">${version}</span>
          </div>
          <div class="system-app-meta">${publisher} · ${escapeHtml(app.source || "Windows")}</div>
          <div class="installed-path">${location}</div>
        </div>
        <span class="badge">Gestionada por Windows</span>
      </article>
    `;
  }).join("");
}

function resetModalButtons() {
  if (modalProgressTrack) modalProgressTrack.style.display = "block";
  if (btnModalCancel) btnModalCancel.style.display = "none";
  if (btnModalConfirm) btnModalConfirm.style.display = "none";
  if (btnModalDone) btnModalDone.style.display = "inline-flex";
}

async function startUninstallAll() {
  modalTitle.textContent = "Desinstalar Todo Packwire";
  modalDesc.textContent = "¿Estás seguro de que deseas desinstalar completamente Packwire y todos sus componentes?";
  modalStatusText.textContent = "Se eliminarán todos los paquetes descargados, los shims generados, la caché temporal y se retirará Packwire del PATH del sistema.";

  if (modalProgressTrack) modalProgressTrack.style.display = "none";
  if (modalTerminalBox) modalTerminalBox.style.display = "none";

  if (btnModalCancel) btnModalCancel.style.display = "inline-flex";
  if (btnModalConfirm) {
    btnModalConfirm.style.display = "inline-flex";
    btnModalConfirm.textContent = "Confirmar Desinstalación Total";
  }
  if (btnModalDone) btnModalDone.style.display = "none";

  modalFooter.style.display = "flex";
  modalOverlay.style.display = "flex";

  const onConfirm = async () => {
    btnModalConfirm.removeEventListener("click", onConfirm);
    btnModalCancel.removeEventListener("click", onCancel);

    if (btnModalCancel) btnModalCancel.style.display = "none";
    if (btnModalConfirm) btnModalConfirm.style.display = "none";
    if (modalProgressTrack) modalProgressTrack.style.display = "block";

    modalTitle.textContent = "Desinstalando Packwire...";
    modalDesc.textContent = "Limpiando paquetes, shims, caché y variables de entorno del sistema...";
    modalStatusText.textContent = "Por favor espera mientras se ejecuta la desinstalación completa...";

    try {
      const res = await API.uninstallAll(true);
      if (res.success) {
        modalTitle.textContent = "¡Desinstalación Completada!";
        modalDesc.textContent = res.message;
        modalStatusText.textContent = "Todos los paquetes, shims y rutas han sido limpiados con éxito.";
        showToast("Todo Packwire ha sido desinstalado correctamente.", "success");
      } else {
        modalTitle.textContent = "Aviso de Desinstalación";
        modalDesc.textContent = res.message;
        modalStatusText.textContent = "La operación finalizó con algunos avisos.";
        showToast(res.message, "error");
      }
    } catch (err) {
      modalTitle.textContent = "Error";
      modalDesc.textContent = err.toString();
      modalStatusText.textContent = "Ocurrió un error inesperado al desinstalar.";
      showToast(err.toString(), "error");
    } finally {
      if (modalProgressTrack) modalProgressTrack.style.display = "none";
      if (btnModalDone) btnModalDone.style.display = "inline-flex";
      modalFooter.style.display = "flex";
      availableUpdates = {};
      await refreshInstalled();
      renderCatalog();
      await loadSystemInfo();
    }
  };

  const onCancel = () => {
    btnModalConfirm.removeEventListener("click", onConfirm);
    btnModalCancel.removeEventListener("click", onCancel);
    modalOverlay.style.display = "none";
    resetModalButtons();
  };

  btnModalConfirm.addEventListener("click", onConfirm);
  btnModalCancel.addEventListener("click", onCancel);
}

async function startReinstall(pkgId, pkgName) {
  resetModalButtons();
  modalTitle.textContent = `Reinstalando ${pkgName}`;
  modalDesc.textContent = `Reinstalando paquete '${pkgId}' limpiamente en segundo plano...`;
  modalStatusText.textContent = "Encolando tarea en segundo plano...";
  if (modalTerminalBox) modalTerminalBox.style.display = "block";
  if (modalTerminalLogs) modalTerminalLogs.textContent = "Enviando a la cola de trabajadores en segundo plano...\n";
  if (terminalLivePill) {
    terminalLivePill.textContent = "EN VIVO";
    terminalLivePill.style.color = "#10b981";
  }
  modalFooter.style.display = "none";
  modalOverlay.style.display = "flex";

  try {
    const queueRes = await API.queueInstall(pkgId, null, null, true);
    if (!queueRes || !queueRes.success) {
      modalTitle.textContent = "Aviso de Reinstalación";
      modalDesc.textContent = queueRes ? queueRes.message : "Error al encolar tarea.";
      modalStatusText.textContent = "No se pudo iniciar la tarea.";
      modalFooter.style.display = "flex";
      showToast(queueRes ? queueRes.message : "Error al encolar.", "error");
      return;
    }

    const jobId = queueRes.job_id;
    if (modalTerminalLogs) {
      modalTerminalLogs.textContent += `[INFO] Tarea encolada con ID: ${jobId}\n`;
    }

    await pollJobUntilDone(jobId);
  } catch (err) {
    modalTitle.textContent = "Error";
    modalDesc.textContent = err.toString();
    modalStatusText.textContent = "Ocurrió un error inesperado al reinstalar.";
    showToast(err.toString(), "error");
  } finally {
    modalFooter.style.display = "flex";
    await refreshInstalled();
    renderCatalog();
  }
}

async function startUpdate(pkgId, pkgName) {
  resetModalButtons();
  modalTitle.textContent = `Actualizando ${pkgName}`;
  modalDesc.textContent = `Buscando y aplicando la versión más reciente para '${pkgId}'...`;
  modalStatusText.textContent = "Descargando nueva versión y actualizando shims...";
  modalFooter.style.display = "none";
  modalOverlay.style.display = "flex";

  try {
    const results = await API.update(pkgId);
    const first = (Array.isArray(results) && results.length > 0) ? results[0] : results;
    if (first && first.success) {
      modalTitle.textContent = "¡Actualización Completada!";
      modalDesc.textContent = first.message;
      modalStatusText.textContent = "La versión más reciente ya está configurada.";
      showToast(first.message, "success");
    } else {
      modalTitle.textContent = "Estado de Actualización";
      modalDesc.textContent = first ? first.message : "El paquete ya se encuentra en la versión más reciente.";
      modalStatusText.textContent = "Información de versión procesada.";
      showToast(first ? first.message : "Actualizado.", "info");
    }
  } catch (err) {
    modalTitle.textContent = "Error";
    modalDesc.textContent = err.toString();
    modalStatusText.textContent = "Error al actualizar paquete.";
    showToast(err.toString(), "error");
  } finally {
    modalFooter.style.display = "flex";
    delete availableUpdates[pkgId];
    await refreshInstalled();
    renderCatalog();
    checkUpdatesInBackground();
  }
}

async function startInstall(pkg, mode, selectedVersion = undefined) {
  resetModalButtons();
  if (mode === "site") {
    modalTitle.textContent = "Abriendo sitio oficial";
    modalDesc.textContent = `Redirigiendo a la web oficial de ${pkg.name}...`;
    modalStatusText.textContent = "Se abrirá tu navegador predeterminado para instalación manual.";
    if (modalTerminalBox) modalTerminalBox.style.display = "none";
    modalOverlay.style.display = "flex";
    modalFooter.style.display = "flex";

    const res = await API.install(pkg.id, "site");
    if (res.success) {
      showToast("Navegador abierto correctamente.", "success");
      await refreshInstalled();
      renderCatalog();
    } else {
      showToast(res.message, "error");
    }
    return;
  }

  // Queue background thread installation (here or command)
  const effectiveMode = (pkg.install && pkg.install.mode) || mode || "here";
  modalTitle.textContent = `Instalando ${pkg.name}`;
  modalDesc.textContent = `Procesando '${pkg.id}' en modo '${effectiveMode}' en segundo plano...`;
  modalStatusText.textContent = "Encolando tarea en segundo plano...";
  if (modalTerminalBox) modalTerminalBox.style.display = "block";
  if (modalTerminalLogs) modalTerminalLogs.textContent = "Enviando a la cola de trabajadores en segundo plano...\n";
  if (terminalLivePill) {
    terminalLivePill.textContent = "EN VIVO";
    terminalLivePill.style.color = "#10b981";
  }
  modalFooter.style.display = "none";
  modalOverlay.style.display = "flex";

  try {
    const requestedVersion = selectedVersion !== undefined
      ? (selectedVersion || null)
      : (pkg.type === "fixed" && pkg.version && pkg.version.toLowerCase() !== "latest" ? pkg.version : null);
    const installPackageId = selectedVersion !== undefined || pkg.type !== "fixed"
      ? pkg.id.split("@")[0]
      : pkg.id;
    const queueRes = await API.queueInstall(installPackageId, effectiveMode, requestedVersion);
    if (!queueRes || !queueRes.success) {
      modalTitle.textContent = "Error al iniciar";
      modalDesc.textContent = queueRes ? queueRes.message : "No se pudo encolar la tarea.";
      modalStatusText.textContent = "No se pudo encolar la tarea.";
      modalFooter.style.display = "flex";
      showToast(queueRes ? queueRes.message : "Error al encolar.", "error");
      return;
    }

    const jobId = queueRes.job_id;
    if (modalTerminalLogs) {
      modalTerminalLogs.textContent += `[INFO] Tarea encolada con ID: ${jobId}\n`;
    }

    await pollJobUntilDone(jobId);
  } catch (err) {
    modalTitle.textContent = "Error";
    modalDesc.textContent = err.toString();
    modalStatusText.textContent = "Ocurrió un error inesperado.";
    showToast(err.toString(), "error");
  } finally {
    modalFooter.style.display = "flex";
    await refreshInstalled();
    renderCatalog();
  }
}

async function pollJobUntilDone(jobId) {
  return new Promise((resolve) => {
    const pollInterval = setInterval(async () => {
      try {
        const job = await API.getJob(jobId);
        if (!job) return;

        modalStatusText.textContent = job.status_text || "Procesando...";

        if (modalTerminalLogs && Array.isArray(job.logs)) {
          modalTerminalLogs.textContent = job.logs.join("\n");
          modalTerminalLogs.scrollTop = modalTerminalLogs.scrollHeight;
        }

        if (job.status === "completed") {
          clearInterval(pollInterval);
          modalTitle.textContent = "¡Instalación Completada!";
          modalDesc.textContent = job.result?.message || "Paquete instalado con éxito.";
          if (terminalLivePill) {
            terminalLivePill.textContent = "COMPLETADO";
            terminalLivePill.style.color = "#10b981";
          }
          showToast(job.result?.message || "Instalación completada con éxito.", "success");
          resolve();
        } else if (job.status === "failed") {
          clearInterval(pollInterval);
          modalTitle.textContent = "Aviso / Error en la Instalación";
          modalDesc.textContent = job.result?.message || "Ocurrió un error durante la instalación.";
          if (terminalLivePill) {
            terminalLivePill.textContent = "FALLIDO";
            terminalLivePill.style.color = "#ef4444";
          }
          showToast(job.result?.message || "Error al instalar paquete.", "error");
          resolve();
        }
      } catch (pollErr) {
        console.warn("Error consultando estado del trabajo:", pollErr);
      }
    }, 250);
  });
}

function showToast(message, type = "info") {
  const toast = document.createElement("div");
  toast.className = `toast ${type}`;
  let iconSvg = '';
  if (type === "success") {
    iconSvg = `<svg class="toast-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>`;
  } else if (type === "error") {
    iconSvg = `<svg class="toast-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>`;
  } else {
    iconSvg = `<svg class="toast-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line></svg>`;
  }
  toast.innerHTML = `<span class="toast-icon-wrap">${iconSvg}</span><span>${message}</span>`;
  toastContainer.appendChild(toast);

  setTimeout(() => {
    toast.style.opacity = "0";
    toast.style.transform = "translateX(100%)";
    toast.style.transition = "all 0.25s ease";
    setTimeout(() => toast.remove(), 250);
  }, 4000);
}

function showAppUpdateToast(update) {
  const toast = document.createElement("div");
  toast.className = "toast info update-toast";
  toast.innerHTML = `
    <span class="toast-icon-wrap">
      <svg class="toast-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 3v12"></path><path d="m7 10 5 5 5-5"></path><path d="M5 21h14"></path>
      </svg>
    </span>
    <span class="toast-update-copy">Se encontró una actualización de Packwire: <strong>v${escapeHtml(update.latest_version)}</strong></span>
    <button class="toast-update-action" type="button">Ver actualización</button>
  `;
  toastContainer.appendChild(toast);

  const action = toast.querySelector(".toast-update-action");
  action.addEventListener("click", async () => {
    await API.openSite(update.release_url);
    toast.remove();
  });
  setTimeout(() => {
    if (!toast.isConnected) return;
    toast.style.opacity = "0";
    toast.style.transform = "translateX(100%)";
    toast.style.transition = "all 0.25s ease";
    setTimeout(() => toast.remove(), 250);
  }, 10000);
}


function openGuideModal(guideName) {
  const modal = document.getElementById("guide-modal");
  if (modal) {
    modal.style.display = "flex";
  }
}

function setupGuideModalListeners() {
  const modal = document.getElementById("guide-modal");
  const btnClose = document.getElementById("btn-guide-modal-close");
  const btnDone = document.getElementById("btn-guide-modal-done");
  const btnBrowser = document.getElementById("btn-guide-open-browser");

  if (btnClose) btnClose.addEventListener("click", () => { if (modal) modal.style.display = "none"; });
  if (btnDone) btnDone.addEventListener("click", () => { if (modal) modal.style.display = "none"; });
  if (btnBrowser) btnBrowser.addEventListener("click", () => {
    API.openGuide("gcc_windows11.html");
  });
  if (modal) {
    modal.addEventListener("click", (e) => {
      if (e.target === modal) modal.style.display = "none";
    });
  }
}
