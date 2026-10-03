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
  install(id, mode, version) { return this.call("install", id, mode, version); },
  reinstall(id) { return this.call("reinstall", id); },
  uninstall(id) { return this.call("uninstall", id); },
  update(id) { return this.call("update", id); },
  checkUpdates() { return this.call("check_updates"); },
  listThemes() { return this.call("list_themes"); },
  getConfig(key) { return this.call("get_config", key); },
  setConfig(key, value) { return this.call("set_config", key, value); },
  queueInstall(id, mode, version, force = false) { return this.call("queue_install", id, mode, version, force); },
  getJob(jobId) { return this.call("get_job", jobId); },
  listJobs() { return this.call("list_jobs"); },
  addPath() { return this.call("add_path"); },
  uninstallAll(removePath = true) { return this.call("uninstall_all", removePath); },
  openSite(url) { return this.call("open_site", url); }
};

// Application State
let currentFilter = "all";
let catalogSearchQuery = "";
let installedSearchQuery = "";
let availablePackages = [];
let installedPackages = {};
let availableUpdates = {};

// Clean SVG Icons for Buttons (No cartoon emojis)
const ICONS = {
  download: `<svg class="btn-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>`,
  terminal: `<svg class="btn-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"></polyline><line x1="12" y1="19" x2="20" y2="19"></line></svg>`,
  refresh: `<svg class="btn-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/></svg>`,
  external: `<svg class="btn-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>`,
  trash: `<svg class="btn-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>`,
  arrowUp: `<svg class="btn-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="19" x2="12" y2="5"></line><polyline points="5 12 12 5 19 12"></polyline></svg>`
};

// DOM Elements
const themeSelectorEl = document.getElementById("theme-selector");
const catalogCardsEl = document.getElementById("catalog-cards");
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
});

async function initApp() {
  await setupTheme();
  await loadSystemInfo();
  await refreshCatalog();
  await refreshInstalled();
  
  // Detect updates in background as soon as app starts
  checkUpdatesInBackground();
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

  // Modal close buttons
  btnModalDone.addEventListener("click", () => {
    modalOverlay.style.display = "none";
    resetModalButtons();
  });
  btnModalClose.addEventListener("click", () => {
    modalOverlay.style.display = "none";
    resetModalButtons();
  });

  // Uninstall all buttons
  const btnUninstallAll = document.getElementById("btn-uninstall-all");
  if (btnUninstallAll) {
    btnUninstallAll.addEventListener("click", startUninstallAll);
  }
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
    if (e.key === "/" && document.activeElement !== catalogSearchInput && document.activeElement !== installedSearchInput) {
      const activeTab = document.querySelector(".nav-tab.active");
      if (activeTab && activeTab.dataset.tab === "catalog" && catalogSearchInput) {
        e.preventDefault();
        catalogSearchInput.focus();
      }
    }
  });
}

async function loadSystemInfo() {
  const info = await API.getSystemInfo();
  if (!info) return;

  const osMap = { windows: "Windows", darwin: "macOS", linux: "Linux" };
  const osName = osMap[info.os] || info.os;
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
}

async function refreshCatalog() {
  catalogCardsEl.innerHTML = `<div class="skeleton-card">Consultando catálogo oficial...</div>`;
  availablePackages = await API.listAvailable() || [];
  renderCatalog();
}

function renderIconHtml(icon) {
  if (!icon) return '<div class="pkg-icon-container"><span class="pkg-icon-emoji">📦</span></div>';
  const isUrl = icon.startsWith("http://") || icon.startsWith("https://") || icon.startsWith("data:") || icon.endsWith(".png") || icon.endsWith(".svg") || icon.endsWith(".ico");
  if (isUrl) {
    return `<div class="pkg-icon-container"><img src="${icon}" class="pkg-icon-img" alt="icon"></div>`;
  }
  return `<div class="pkg-icon-container"><span class="pkg-icon-emoji">${icon}</span></div>`;
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

function renderCatalog() {
  catalogCardsEl.innerHTML = "";
  const query = catalogSearchQuery.trim().toLowerCase();

  const filtered = availablePackages.filter(pkg => {
    // 1. Channel filter
    if (currentFilter !== "all" && pkg.type !== currentFilter) return false;

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

  // Update search UI info
  if (catalogSearchInfo) {
    if (query) {
      catalogSearchInfo.style.display = "flex";
      catalogSearchInfo.innerHTML = `
        <span>Mostrando <strong>${filtered.length}</strong> de <strong>${availablePackages.length}</strong> paquetes para "<strong>${escapeHtml(catalogSearchQuery)}</strong>"</span>
        <button class="btn-reset-search" id="btn-reset-catalog-search">Limpiar filtro</button>
      `;
      const btnReset = document.getElementById("btn-reset-catalog-search");
      if (btnReset) {
        btnReset.addEventListener("click", () => {
          if (catalogSearchInput) catalogSearchInput.value = "";
          catalogSearchQuery = "";
          if (btnClearCatalogSearch) btnClearCatalogSearch.style.display = "none";
          renderCatalog();
        });
      }
    } else {
      catalogSearchInfo.style.display = "none";
    }
  }

  if (btnClearCatalogSearch) {
    btnClearCatalogSearch.style.display = query ? "block" : "none";
  }

  if (filtered.length === 0) {
    if (query) {
      catalogCardsEl.innerHTML = `
        <div class="info-card" style="grid-column: 1 / -1; text-align: center; padding: 40px 20px;">
          <div style="display: flex; justify-content: center; margin-bottom: 10px; color: var(--text-muted);">
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
          </div>
          <span class="info-label" style="font-size: 14px;">No se encontraron paquetes</span>
          <p class="subtitle" style="margin-top: 6px;">No hay resultados para "<strong>${escapeHtml(catalogSearchQuery)}</strong>". Prueba con 'gcc', 'ffmpeg', 'node', 'python' o limpia el buscador.</p>
        </div>
      `;
    } else {
      catalogCardsEl.innerHTML = `<p class="subtitle">No hay paquetes que coincidan con este filtro.</p>`;
    }
    return;
  }

  filtered.forEach(pkg => {
    const card = document.createElement("div");
    card.className = "package-card";

    const isInstalled = !!installedPackages[pkg.id];
    const isStable = pkg.type === "stable";
    const typeLabel = isStable ? "Latest Stable" : "Fixed";
    const typeClass = isStable ? "stable" : "fixed";

    const isCommand = !!(pkg.install && (pkg.install.mode === "command" || pkg.install.command));
    const hasHere = !!(pkg.install && pkg.install.here && (pkg.install.here.url || pkg.type === "stable"));
    const hasSite = !!(pkg.install && pkg.install.site && pkg.install.site.url);
    const isSiteOnly = !hasHere && !isCommand && hasSite;

    let actionsHtml = "";
    if (isSiteOnly) {
      actionsHtml = `
        <button class="btn btn-primary btn-install-site" data-id="${pkg.id}" style="flex: 1;">
          ${ICONS.external} Abrir Sitio Oficial
        </button>
      `;
    } else if (isCommand) {
      actionsHtml = `
        <button class="btn btn-primary btn-install-here" data-id="${pkg.id}">
          ${isInstalled ? ICONS.refresh + ' Reinstalar' : ICONS.terminal + ' Instalar por Consola'}
        </button>
        ${hasSite ? `
        <button class="btn btn-outline btn-install-site" data-id="${pkg.id}">
          ${ICONS.external} Sitio
        </button>` : ''}
      `;
    } else {
      actionsHtml = `
        <button class="btn btn-primary btn-install-here" data-id="${pkg.id}">
          ${isInstalled ? ICONS.refresh + ' Reinstalar' : ICONS.download + ' Descargar Aquí'}
        </button>
        ${hasSite ? `
        <button class="btn btn-outline btn-install-site" data-id="${pkg.id}">
          ${ICONS.external} Sitio
        </button>` : ''}
      `;
    }

    card.innerHTML = `
      <div>
        <div class="card-top">
          <div class="pkg-title-area">
            ${renderIconHtml(pkg.icon)}
            <div>
              <div class="pkg-name">${pkg.name}</div>
              <div class="pkg-version">v${pkg.version || 'auto'}</div>
            </div>
          </div>
          <span class="pkg-type-pill ${typeClass}">${typeLabel}</span>
        </div>
        <p class="pkg-desc">${pkg.description || 'Paquete de desarrollo oficial listo para instalar.'}</p>
      </div>
      <div class="card-actions">
        ${actionsHtml}
      </div>
    `;

    // Action listeners
    const btnHere = card.querySelector(".btn-install-here");
    if (btnHere) {
      btnHere.addEventListener("click", () => startInstall(pkg, "here"));
    }
    const btnSite = card.querySelector(".btn-install-site");
    if (btnSite) {
      btnSite.addEventListener("click", () => startInstall(pkg, "site"));
    }

    catalogCardsEl.appendChild(card);
  });
}

async function refreshInstalled() {
  installedPackages = await API.listInstalled() || {};
  renderInstalledList();
}

function renderInstalledList() {
  const query = installedSearchQuery.trim().toLowerCase();
  const allKeys = Object.keys(installedPackages);
  installedCounterEl.textContent = allKeys.length;

  installedContainerEl.innerHTML = "";

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
        <span>Mostrando <strong>${filteredKeys.length}</strong> de <strong>${allKeys.length}</strong> paquetes instalados para "<strong>${escapeHtml(installedSearchQuery)}</strong>"</span>
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
    installedContainerEl.innerHTML = `
      <div class="info-card">
        <span class="info-label">Sin paquetes activos</span>
        <p class="subtitle">Aún no has instalado ningún paquete con Packwire. Explora el Catálogo para comenzar.</p>
      </div>
    `;
    return;
  }

  if (filteredKeys.length === 0) {
    installedContainerEl.innerHTML = `
      <div class="info-card" style="text-align: center; padding: 36px 20px;">
        <div style="display: flex; justify-content: center; margin-bottom: 8px; color: var(--text-muted);">
          <svg viewBox="0 0 24 24" width="26" height="26" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
        </div>
        <span class="info-label">Sin coincidencias</span>
        <p class="subtitle">No se encontró ningún paquete instalado que coincida con "<strong>${escapeHtml(installedSearchQuery)}</strong>".</p>
      </div>
    `;
    return;
  }

  filteredKeys.forEach(pkgId => {
    const item = installedPackages[pkgId];
    const itemEl = document.createElement("div");
    itemEl.className = "installed-item";

    const shimsHtml = (item.shims || []).map(s => `<span class="shim-tag">${s}</span>`).join(" ");
    const iconHtml = renderIconHtml(item.icon || (item.name.toLowerCase().includes("python") ? "🐍" : "📦"));

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

    installedContainerEl.appendChild(itemEl);
  });
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

async function startInstall(pkg, mode) {
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
    const queueRes = await API.queueInstall(pkg.id, effectiveMode, pkg.version);
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
