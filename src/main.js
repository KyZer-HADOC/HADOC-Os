import "./style.css";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { formatBytes, filterEntries, mergeDirectoryPage, isLowResourceDevice, shouldRefreshDashboard } from "./ui-utils.js";

const app = document.querySelector("#app");
const win = getCurrentWindow();

let startOpen = false;
let fileManagerOpen = false;
let gameCenterOpen = false;
let gameEntries = [];
let discoveredGames = [];
let selectedGameId = null;
let gamingMode = false;
let lowResourcePresentation = false;
let systemRefreshInFlight = false;
let currentDirectory = null;
let currentParent = null;
let currentEntries = [];
let currentQuery = "";
let currentOffset = 0;
let currentTotalEntries = 0;
let currentHasMore = false;
let loadingMoreEntries = false;
let directoryRequestId = 0;
let filterTimer = null;
let quickLocations = [];
let directoryHistory = [];
let historyIndex = -1;

const apps = [
  { name: "File Manager", icon: "▤", desc: "Browse folders and drives", action: "filemanager" },
  { name: "Settings", icon: "⚙", desc: "Windows settings", action: "settings" },
  { name: "Terminal", icon: ">_", desc: "Command line", action: "terminal" },
  { name: "Task Manager", icon: "▥", desc: "System processes", action: "taskmgr" },
  { name: "Notepad", icon: "▧", desc: "Quick notes", action: "notepad" },
  { name: "Browser", icon: "◎", desc: "Open your default browser", action: "browser" },
  { name: "Windows Explorer", icon: "▣", desc: "Open the Windows file explorer", action: "explorer" },
  { name: "Game Center", icon: "🎮", desc: "Game library and hardware benchmarks", action: "gamecenter" }
];

app.innerHTML = `
  <main class="desktop">
    <div class="ambient ambient-one"></div><div class="ambient ambient-two"></div>
    <header class="topbar" data-tauri-drag-region>
      <div class="brand"><span class="brand-mark">H</span><span>HADOC<span class="brand-light"> OS</span></span><i class="live-dot"></i></div>
      <div class="top-center">DESKTOP ENVIRONMENT <span>•</span> PREVIEW 0.1</div>
      <div class="window-actions">
        <button id="min" aria-label="Minimize">−</button><button id="max" aria-label="Maximize">□</button><button id="close" class="close" aria-label="Close">×</button>
      </div>
    </header>
    <section class="hero">
      <div class="hero-copy">
        <div class="eyebrow"><span></span> YOUR WINDOWS. REIMAGINED.</div>
        <h1>Welcome to<br><em>HADOC OS.</em></h1>
        <p>A cleaner space for everything you do.<br>Fast, focused and built around your Windows PC.</p>
        <div class="hero-actions"><button class="primary" id="open-start">Open launcher <b>↗</b></button><button class="ghost" id="full">Immersive mode ⛶</button><button class="ghost" id="open-game-center">Game Center 🎮</button><button class="ghost gaming-toggle" id="gaming-mode" type="button" aria-pressed="false">Gaming mode: Off</button></div>
      </div>
      <div class="orb-wrap"><div class="orb-ring ring-a"></div><div class="orb-ring ring-b"></div><div class="orb"><span>H</span></div><div class="orb-caption">DESIGNED TO FEEL<br><strong>LIKE YOURS.</strong></div></div>
    </section>
    <section class="widgets">
      <article class="widget system-widget">
        <div class="widget-head"><span>01 / LIVE SYSTEM</span><span class="status"><i></i> MONITORING</span></div>
        <div class="system-heading"><div><small>COMPUTER</small><h2 id="computer-name">Loading system...</h2></div><span class="system-badge">WINDOWS</span></div>
        <div class="system-data">
          <div><small>OPERATING SYSTEM</small><strong id="platform">Detecting...</strong></div>
          <div><small>CPU ARCHITECTURE</small><strong id="cpu-arch">Unavailable</strong><span id="cpu-threads">—</span></div>
        </div>
        <div class="memory-block">
          <div class="metric-title"><small>MEMORY USAGE</small><strong id="memory-label">Loading...</strong></div>
          <div class="metric-track"><span id="memory-bar"></span></div>
        </div>
        <div class="storage-block"><small class="storage-label">STORAGE VOLUMES</small><div id="storage-list" class="storage-list"><span class="muted">Reading available drives...</span></div></div>
      </article>
      <article class="widget quick-widget"><div class="widget-head"><span>02 / QUICK ACCESS</span><span>OPEN</span></div><div class="quick-grid">${apps.slice(0,4).map(a=>`<button class="quick-app" data-action="${a.action}"><span>${a.icon}</span><b>${a.name}</b></button>`).join("")}</div></article>
    </section>
    <div class="launcher hidden" id="launcher"><div class="launcher-card"><div class="launcher-top"><div><small>HADOC OS / APPS</small><h2>What do you need?</h2></div><button id="launcher-close" aria-label="Close launcher">×</button></div><div class="app-list">${apps.map(a=>`<button class="app-row" data-action="${a.action}"><span class="app-icon">${a.icon}</span><span><b>${a.name}</b><small>${a.desc}</small></span><i>↗</i></button>`).join("")}</div></div></div>
    <section class="file-manager hidden" id="file-manager" aria-label="HADOC File Manager">
      <div class="file-manager-backdrop" id="fm-backdrop"></div>
      <div class="file-window" role="dialog" aria-modal="true" aria-labelledby="fm-title">
        <header class="fm-header">
          <div class="fm-app-mark">▤</div>
          <div class="fm-title-wrap"><strong id="fm-title">File Manager</strong><small>Browse this PC</small></div>
          <span class="fm-readonly">READ ONLY</span>
          <button class="fm-close" id="fm-close" aria-label="Close File Manager">×</button>
        </header>
        <div class="fm-toolbar">
          <div class="fm-nav">
            <button id="fm-back" aria-label="Go back" title="Back" disabled>←</button>
            <button id="fm-forward" aria-label="Go forward" title="Forward" disabled>→</button>
            <button id="fm-up" aria-label="Go to parent folder" title="Up" disabled>↑</button>
            <button id="fm-refresh" aria-label="Refresh folder" title="Refresh">↻</button>
          </div>
          <form class="fm-address" id="fm-path-form">
            <span>⌕</span>
            <input id="fm-path" type="text" autocomplete="off" spellcheck="false" placeholder="Enter a folder path">
            <button type="submit">Go</button>
          </form>
        </div>
        <div class="fm-body">
          <aside class="fm-sidebar">
            <div class="fm-sidebar-label">QUICK ACCESS</div>
            <div id="fm-locations" class="fm-locations"><span class="muted">Loading locations...</span></div>
          </aside>
          <section class="fm-main">
            <div class="fm-content-head">
              <div class="fm-folder-heading"><strong id="fm-folder-title">Home</strong><small id="fm-folder-subtitle">Choose a location to browse</small></div>
              <label class="fm-filter"><span>⌕</span><input id="fm-filter" type="search" placeholder="Filter files"></label>
            </div>
            <div class="fm-column-head"><span>NAME</span><span>SIZE</span><span>MODIFIED</span></div>
            <div id="fm-entries" class="fm-entries"><div class="fm-empty">Loading your folders...</div></div>
          </section>
        </div>
        <footer class="fm-footer"><span id="fm-status">Connecting to Windows...</span><span>HADOC OS · Read-only browsing</span></footer>
      </div>
    </section>

    <section class="game-center hidden" id="game-center" aria-label="HADOC Game Center">
      <div class="gc-backdrop" id="gc-backdrop"></div>
      <div class="gc-window" role="dialog" aria-modal="true" aria-labelledby="gc-title">
        <header class="gc-header">
          <div class="gc-mark">H</div>
          <div class="gc-heading"><small>HADOC OS / PERFORMANCE SUITE</small><strong id="gc-title">Game Center</strong></div>
          <span class="gc-safe-label">NO OS OR DRIVER TWEAKS</span>
          <button class="fm-close" id="gc-close" aria-label="Close Game Center">×</button>
        </header>
        <nav class="gc-tabs" aria-label="Game Center sections">
          <button class="gc-tab active" data-gc-tab="library" type="button">Game Library</button>
          <button class="gc-tab" data-gc-tab="benchmark" type="button">Hardware Benchmark</button>
        </nav>
        <section class="gc-panel" id="gc-library-panel">
          <div class="gc-library-layout">
            <aside class="gc-game-sidebar">
              <div class="gc-sidebar-head"><div><small>YOUR COLLECTION</small><strong id="gc-game-count">0 games</strong></div><button id="gc-scan" type="button">Detect</button></div>
              <div id="gc-games-list" class="gc-games-list"><div class="gc-empty">Loading saved games…</div></div>
            </aside>
            <div class="gc-content">
              <div id="gc-selected-empty" class="gc-selected-empty">
                <span class="gc-big-icon">🎮</span>
                <strong>Select a game</strong>
                <p>Add a game executable or detect a game shortcut from your Windows Start Menu.</p>
              </div>
              <div id="gc-game-detail" class="gc-game-detail hidden">
                <div class="gc-selected-heading"><div><small>SELECTED GAME</small><h2 id="gc-selected-name">Game</h2><p id="gc-selected-path"></p></div><span class="gc-selected-chip">LOCAL ENTRY</span></div>
                <div class="gc-profile-card">
                  <label for="gc-game-profile">PER-GAME HADOC PROFILE</label>
                  <select id="gc-game-profile">
                    <option value="gaming">Gaming / Low Resource</option>
                    <option value="balanced">Balanced</option>
                  </select>
                  <p id="gc-profile-description">Pauses HADOC dashboard work and removes expensive effects while the game is running.</p>
                </div>
                <div class="gc-game-actions">
                  <button id="gc-launch-game" class="gc-primary" type="button">Launch game ↗</button>
                  <button id="gc-save-profile" class="gc-secondary" type="button">Save profile</button>
                  <button id="gc-remove-game" class="gc-danger" type="button">Remove entry</button>
                </div>
              </div>
              <div class="gc-add-card">
                <div class="gc-add-title"><strong>Add a game manually</strong><small>Use the full path to a .exe, .lnk or .url file</small></div>
                <form id="gc-add-form" class="gc-add-form">
                  <input id="gc-add-name" type="text" maxlength="80" placeholder="Game name (optional)" aria-label="Game name">
                  <input id="gc-add-path" type="text" required placeholder="C:\Games\Example\Game.exe" aria-label="Game executable or shortcut path">
                  <select id="gc-add-profile" aria-label="Default game profile">
                    <option value="gaming">Gaming / Low Resource</option>
                    <option value="balanced">Balanced</option>
                  </select>
                  <button class="gc-primary" type="submit">Add to library</button>
                </form>
              </div>
              <div class="gc-discovery-card">
                <div class="gc-add-title"><strong>Detected shortcuts</strong><small>Only scans Start Menu shortcuts when you ask</small></div>
                <div id="gc-discovered-list" class="gc-discovered-list"><span class="gc-muted">Select Detect to look for known game launchers and titles.</span></div>
              </div>
            </div>
          </div>
        </section>
        <section class="gc-panel hidden" id="gc-benchmark-panel">
          <div class="gc-benchmark-intro"><div><small>HARDWARE PERFORMANCE TEST</small><h2>Quick Benchmark</h2><p>Measures CPU throughput, memory copy bandwidth and sequential storage I/O. Temporary test data is removed afterwards.</p></div><button id="gc-run-benchmark" class="gc-primary" type="button">Run benchmark ▶</button></div>
          <div id="gc-benchmark-status" class="gc-benchmark-status">Ready. Close other heavy apps for more consistent results.</div>
          <div id="gc-benchmark-results" class="gc-benchmark-results">
            <div class="gc-benchmark-placeholder">No benchmark results yet.</div>
          </div>
          <p class="gc-benchmark-footnote">Results are measurements, not an FPS prediction. Disk read results may include the Windows file cache. This test does not change drivers, services, registry settings or game processes.</p>
        </section>
      </div>
    </section>
    <footer class="taskbar"><button class="start-button" id="task-start"><span class="mini-mark">H</span><span>HADOC</span></button><div class="taskbar-divider"></div><span class="taskbar-label">DESKTOP</span><div class="taskbar-right"><span class="host-label">RUNNING ON WINDOWS</span><span id="clock"></span></div></footer>
    <div class="toast" id="toast" role="status" aria-live="polite"></div>
  </main>`;

const toast = (message) => {
  const el = document.querySelector("#toast");
  el.textContent = String(message);
  el.classList.add("show");
  setTimeout(() => el.classList.remove("show"), 2800);
};

const setText = (selector, value) => {
  const element = document.querySelector(selector);
  if (element) element.textContent = value ?? "Unavailable";
};

function toggleLauncher(force) {
  startOpen = typeof force === "boolean" ? force : !startOpen;
  document.querySelector("#launcher").classList.toggle("hidden", !startOpen);
}

async function launch(action) {
  if (action === "gamecenter") {
    toggleLauncher(false);
    await openGameCenter();
    return;
  }
  if (action === "filemanager") {
    toggleLauncher(false);
    await openFileManager();
    return;
  }

  try {
    const result = await invoke("launch_windows_app", { app: action });
    toast(result || "Opened.");
  } catch (error) {
    toast(String(error));
  }
}


async function openGameCenter() {
  if (fileManagerOpen) closeFileManager();
  gameCenterOpen = true;
  document.querySelector("#game-center").classList.remove("hidden");
  await refreshGameLibrary();
}

function closeGameCenter() {
  gameCenterOpen = false;
  document.querySelector("#game-center").classList.add("hidden");
}

function selectedGame() {
  return gameEntries.find(game => game.id === selectedGameId) ?? null;
}

function renderGameLibrary() {
  const list = document.querySelector("#gc-games-list");
  list.replaceChildren();
  setText("#gc-game-count", `${gameEntries.length} ${gameEntries.length === 1 ? "game" : "games"}`);

  if (!gameEntries.length) {
    const empty = document.createElement("div");
    empty.className = "gc-empty";
    empty.textContent = "No games saved yet.";
    list.append(empty);
  } else {
    for (const game of gameEntries) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = `gc-game-item${game.id === selectedGameId ? " active" : ""}`;
      const icon = document.createElement("span");
      icon.className = "gc-game-icon";
      icon.textContent = "🎮";
      const details = document.createElement("span");
      details.className = "gc-game-item-copy";
      const name = document.createElement("b");
      name.textContent = game.name;
      const profile = document.createElement("small");
      profile.textContent = gameProfileLabel(game.profile);
      details.append(name, profile);
      button.append(icon, details);
      button.addEventListener("click", () => {
        selectedGameId = game.id;
        renderGameLibrary();
      });
      list.append(button);
    }
  }

  const selected = selectedGame();
  document.querySelector("#gc-selected-empty").classList.toggle("hidden", Boolean(selected));
  document.querySelector("#gc-game-detail").classList.toggle("hidden", !selected);
  if (!selected) return;

  setText("#gc-selected-name", selected.name);
  setText("#gc-selected-path", selected.path);
  document.querySelector("#gc-game-profile").value = selected.profile;
  updateGameProfileDescription(selected.profile);
}

function updateGameProfileDescription(profile) {
  setText(
    "#gc-profile-description",
    profile === "gaming"
      ? "Pauses HADOC dashboard monitoring and removes HADOC visual effects. Does not change Windows services, drivers or game priority."
      : "Keeps the normal HADOC desktop experience while the game is running."
  );
}

async function refreshGameLibrary() {
  try {
    gameEntries = await invoke("get_games");
    if (!gameEntries.some(game => game.id === selectedGameId)) {
      selectedGameId = gameEntries[0]?.id ?? null;
    }
    renderGameLibrary();
  } catch (error) {
    setText("#gc-game-count", "Unavailable");
    const list = document.querySelector("#gc-games-list");
    list.textContent = String(error);
  }
}

async function addGameToLibrary(name, path, profile = "gaming") {
  try {
    gameEntries = await invoke("add_game", { name, path, profile });
    const added = gameEntries.find(game => game.path.toLowerCase() === path.toLowerCase());
    selectedGameId = added?.id ?? gameEntries[0]?.id ?? null;
    renderGameLibrary();
    renderDiscoveredGames();
    document.querySelector("#gc-add-name").value = "";
    document.querySelector("#gc-add-path").value = "";
    toast("Game added to HADOC Game Center.");
  } catch (error) {
    toast(String(error));
  }
}

function renderDiscoveredGames() {
  const container = document.querySelector("#gc-discovered-list");
  container.replaceChildren();
  const savedPaths = new Set(gameEntries.map(game => game.path.toLowerCase()));
  const candidates = discoveredGames.filter(game => !savedPaths.has(game.path.toLowerCase()));

  if (!candidates.length) {
    const empty = document.createElement("span");
    empty.className = "gc-muted";
    empty.textContent = discoveredGames.length
      ? "All detected games are already in your library."
      : "No known game shortcuts found. You can add an executable path manually.";
    container.append(empty);
    return;
  }

  for (const candidate of candidates) {
    const row = document.createElement("div");
    row.className = "gc-discovered-row";
    const name = document.createElement("span");
    name.textContent = candidate.name;
    const add = document.createElement("button");
    add.type = "button";
    add.textContent = "Add";
    add.addEventListener("click", () => addGameToLibrary(candidate.name, candidate.path, "gaming"));
    row.append(name, add);
    container.append(row);
  }
}

async function scanForGames() {
  const button = document.querySelector("#gc-scan");
  button.disabled = true;
  button.textContent = "Scanning…";
  setText("#gc-discovered-list", "Checking Start Menu shortcuts…");
  try {
    discoveredGames = await invoke("discover_games");
    renderDiscoveredGames();
  } catch (error) {
    setText("#gc-discovered-list", String(error));
  } finally {
    button.disabled = false;
    button.textContent = "Detect";
  }
}

async function launchSelectedGame() {
  const game = selectedGame();
  if (!game) return;

  try {
    const result = await invoke("launch_game", { id: game.id });
    closeGameCenter();
    setGamingMode(game.profile === "gaming");
    toast(result || `Launching ${game.name}…`);
  } catch (error) {
    toast(String(error));
  }
}

function setGameCenterTab(tab) {
  const library = tab === "library";
  document.querySelector("#gc-library-panel").classList.toggle("hidden", !library);
  document.querySelector("#gc-benchmark-panel").classList.toggle("hidden", library);
  document.querySelectorAll("[data-gc-tab]").forEach(button => {
    button.classList.toggle("active", button.dataset.gcTab === tab);
  });
}

function renderBenchmarkResults(result) {
  const metrics = [
    ["CPU · Single thread", result.cpu_single_m_iterations_s, "M iterations/s"],
    ["CPU · Multi thread", result.cpu_multi_m_iterations_s, "M iterations/s"],
    ["Memory copy", result.memory_copy_mb_s, "MB/s"],
    ["Disk write", result.disk_write_mb_s, "MB/s"],
    ["Disk read", result.disk_read_mb_s, "MB/s"]
  ];
  const container = document.querySelector("#gc-benchmark-results");
  container.replaceChildren();

  for (const [label, value, unit] of metrics) {
    const card = document.createElement("article");
    card.className = "gc-metric-card";
    const heading = document.createElement("small");
    heading.textContent = label;
    const number = document.createElement("strong");
    number.textContent = formatBenchmarkMetric(value, unit);
    card.append(heading, number);
    container.append(card);
  }

  const summary = document.createElement("div");
  summary.className = "gc-benchmark-summary";
  summary.textContent = `${result.logical_threads} logical threads · ${result.duration_ms} ms total`;
  container.append(summary);
  setText("#gc-benchmark-status", result.disk_read_note || "Benchmark completed.");
}

async function runHardwareBenchmark() {
  const button = document.querySelector("#gc-run-benchmark");
  button.disabled = true;
  button.textContent = "Testing…";
  setText("#gc-benchmark-status", "Measuring CPU, memory and storage. Please keep HADOC open…");
  document.querySelector("#gc-benchmark-results").replaceChildren();

  try {
    const result = await invoke("run_quick_benchmark");
    renderBenchmarkResults(result);
  } catch (error) {
    setText("#gc-benchmark-status", String(error));
  } finally {
    button.disabled = false;
    button.textContent = "Run again ↻";
  }
}

function applyPerformancePresentation() {
  const desktop = document.querySelector(".desktop");
  desktop.classList.toggle("low-resource-mode", lowResourcePresentation || gamingMode);
  desktop.classList.toggle("gaming-mode", gamingMode);

  const button = document.querySelector("#gaming-mode");
  button.setAttribute("aria-pressed", String(gamingMode));
  button.textContent = gamingMode ? "Gaming mode: On" : "Gaming mode: Off";
  button.classList.toggle("gaming-active", gamingMode);
}

function setGamingMode(enabled) {
  gamingMode = Boolean(enabled);
  if (gamingMode) {
    toggleLauncher(false);
    if (fileManagerOpen) closeFileManager();
  }
  applyPerformancePresentation();
  toast(gamingMode
    ? "Gaming mode enabled — HADOC effects and monitoring paused."
    : "Balanced mode restored.");
  if (!gamingMode && shouldRefreshDashboard({ documentHidden: document.hidden, gamingMode })) {
    refreshSystemDashboard();
  }
}

function updateFileNavigation() {
  document.querySelector("#fm-back").disabled = historyIndex <= 0;
  document.querySelector("#fm-forward").disabled = historyIndex >= directoryHistory.length - 1;
  document.querySelector("#fm-up").disabled = !currentParent;
}

function renderQuickLocations() {
  const container = document.querySelector("#fm-locations");
  container.replaceChildren();

  const groups = [...new Set(quickLocations.map(location => location.group))];
  for (const group of groups) {
    const heading = document.createElement("div");
    heading.className = "fm-location-group";
    heading.textContent = group;
    container.append(heading);

    for (const location of quickLocations.filter(item => item.group === group)) {
      const button = document.createElement("button");
      button.className = "fm-location";
      button.type = "button";
      button.dataset.path = location.path;
      const icon = document.createElement("span");
      icon.textContent = location.icon || "▤";
      const label = document.createElement("b");
      label.textContent = location.name;
      button.append(icon, label);
      button.addEventListener("click", () => loadDirectory(location.path));
      container.append(button);
    }
  }

  if (!quickLocations.length) {
    const empty = document.createElement("span");
    empty.className = "muted";
    empty.textContent = "No quick locations found.";
    container.append(empty);
  }
}

function formatModified(timestamp) {
  if (!Number.isFinite(timestamp)) return "—";
  const date = new Date(timestamp * 1000);
  return Number.isNaN(date.getTime()) ? "—" : date.toLocaleString();
}

function renderEntries({ preserveScroll = false } = {}) {
  const container = document.querySelector("#fm-entries");
  const previousScrollTop = container.scrollTop;
  const visibleEntries = filterEntries(
    currentEntries,
    document.querySelector("#fm-filter").value
  );
  container.replaceChildren();

  if (!visibleEntries.length) {
    const empty = document.createElement("div");
    empty.className = "fm-empty";
    empty.textContent = document.querySelector("#fm-filter").value.trim()
      ? "No items match your filter."
      : "This folder is empty.";
    container.append(empty);
  } else {
    for (const entry of visibleEntries) {
      const row = document.createElement("button");
      row.className = "fm-entry";
      row.type = "button";
      row.dataset.path = entry.path;
      row.dataset.directory = String(Boolean(entry.is_dir));

      const nameCell = document.createElement("span");
      nameCell.className = "fm-entry-name";
      const icon = document.createElement("i");
      icon.className = entry.is_dir ? "fm-entry-icon folder" : "fm-entry-icon";
      icon.textContent = entry.is_dir ? "▰" : "▤";
      const name = document.createElement("b");
      name.textContent = entry.name;
      nameCell.append(icon, name);

      if (entry.is_hidden) {
        const hidden = document.createElement("small");
        hidden.className = "fm-hidden-label";
        hidden.textContent = "HIDDEN";
        nameCell.append(hidden);
      }

      const size = document.createElement("span");
      size.className = "fm-entry-size";
      size.textContent = entry.is_dir ? "—" : formatBytes(entry.size);

      const modified = document.createElement("span");
      modified.className = "fm-entry-modified";
      modified.textContent = formatModified(entry.modified);

      row.append(nameCell, size, modified);
      row.addEventListener("click", () => {
        container.querySelectorAll(".fm-entry.selected").forEach(item => item.classList.remove("selected"));
        row.classList.add("selected");
      });
      row.addEventListener("dblclick", () => openEntry(entry));
      row.addEventListener("keydown", event => {
        if (event.key === "Enter") openEntry(entry);
      });
      container.append(row);
    }
  }

  if (currentHasMore) {
    const more = document.createElement("div");
    more.className = "fm-load-more";
    more.textContent = loadingMoreEntries ? "Loading more items…" : "Scroll down to load more";
    container.append(more);
  }

  if (preserveScroll) container.scrollTop = previousScrollTop;
  const shown = currentEntries.length;
  const total = currentTotalEntries;
  const status = currentHasMore
    ? `Showing ${shown} of ${total} items · scroll for more`
    : `${total} item${total === 1 ? "" : "s"}`;
  setText("#fm-status", status);
}

async function openEntry(entry) {
  if (entry.is_dir) {
    await loadDirectory(entry.path);
    return;
  }

  try {
    await invoke("open_path", { path: entry.path });
  } catch (error) {
    toast(String(error));
  }
}

async function loadDirectory(path = null, options = {}) {
  const { recordHistory = true, query = "" } = options;
  const requestId = ++directoryRequestId;
  const container = document.querySelector("#fm-entries");
  container.innerHTML = '<div class="fm-empty">Loading folder…</div>';
  currentQuery = query;

  try {
    const listing = await invoke("list_directory", {
      path,
      offset: 0,
      limit: 200,
      query: query || null
    });
    if (requestId !== directoryRequestId) return false;

    currentDirectory = listing.path;
    currentParent = listing.parent;
    const page = mergeDirectoryPage([], listing);
    currentEntries = page.entries;
    currentOffset = page.nextOffset;
    currentHasMore = page.hasMore;
    currentTotalEntries = Number(listing.total_entries) || currentEntries.length;
    loadingMoreEntries = false;

    if (recordHistory) {
      directoryHistory = directoryHistory.slice(0, historyIndex + 1);
      directoryHistory.push(currentDirectory);
      historyIndex = directoryHistory.length - 1;
    }

    document.querySelector("#fm-path").value =
      currentDirectory === "__HADOC_THIS_PC__" ? "This PC" : currentDirectory;
    document.querySelector("#fm-filter").value = query;
    const folderName = currentDirectory === "__HADOC_THIS_PC__"
      ? "This PC"
      : currentDirectory.split(/[\\/]/).filter(Boolean).at(-1) || currentDirectory;
    setText("#fm-folder-title", folderName);
    setText("#fm-folder-subtitle", currentDirectory === "__HADOC_THIS_PC__"
      ? "Your Windows drives and storage locations"
      : currentDirectory);
    renderEntries();
    updateFileNavigation();

    document.querySelectorAll(".fm-location").forEach(button => {
      button.classList.toggle("active", button.dataset.path === currentDirectory);
    });
    return true;
  } catch (error) {
    if (requestId !== directoryRequestId) return false;
    currentEntries = [];
    currentHasMore = false;
    currentTotalEntries = 0;
    container.replaceChildren();
    const empty = document.createElement("div");
    empty.className = "fm-empty fm-error";
    empty.textContent = String(error);
    container.append(empty);
    setText("#fm-status", "Could not open folder");
    toast(String(error));
    updateFileNavigation();
    return false;
  }
}

async function loadMoreDirectoryEntries() {
  if (!fileManagerOpen || !currentHasMore || loadingMoreEntries || !currentDirectory) return;
  const requestId = directoryRequestId;
  loadingMoreEntries = true;
  renderEntries({ preserveScroll: true });

  try {
    const listing = await invoke("list_directory", {
      path: currentDirectory,
      offset: currentOffset,
      limit: 200,
      query: currentQuery || null
    });
    if (requestId !== directoryRequestId) return;

    const page = mergeDirectoryPage(currentEntries, listing);
    currentEntries = page.entries;
    currentOffset = page.nextOffset;
    currentHasMore = page.hasMore;
    currentTotalEntries = Number(listing.total_entries) || currentTotalEntries;
  } catch (error) {
    toast(String(error));
    currentHasMore = false;
  } finally {
    loadingMoreEntries = false;
    if (requestId === directoryRequestId) renderEntries({ preserveScroll: true });
  }
}

async function openFileManager() {
  if (gameCenterOpen) closeGameCenter();
  fileManagerOpen = true;
  document.querySelector("#file-manager").classList.remove("hidden");
  document.querySelector("#fm-filter").value = "";

  if (!quickLocations.length) {
    try {
      quickLocations = await invoke("get_quick_locations");
      renderQuickLocations();
    } catch (error) {
      quickLocations = [];
      renderQuickLocations();
      toast(String(error));
    }
  }

  await loadDirectory(currentDirectory, { recordHistory: !currentDirectory });
}

function closeFileManager() {
  fileManagerOpen = false;
  document.querySelector("#file-manager").classList.add("hidden");
}

async function refreshSystemDashboard() {
  if (!shouldRefreshDashboard({ documentHidden: document.hidden, gamingMode }) || systemRefreshInFlight) return;
  systemRefreshInFlight = true;

  try {
    const snapshot = await invoke("get_system_snapshot");
    if (gamingMode || document.hidden) return;
    lowResourcePresentation = isLowResourceDevice(snapshot);
    applyPerformancePresentation();
    setText("#computer-name", snapshot.computer_name || "Unavailable");
    setText("#platform", snapshot.os_version || "Unavailable");
    setText("#cpu-arch", snapshot.cpu_arch || "Unavailable");
    setText("#cpu-threads", snapshot.logical_threads ? `${snapshot.logical_threads} logical threads` : "Thread count unavailable");

    const totalMemory = Number(snapshot.total_memory) || 0;
    const usedMemory = Number(snapshot.used_memory) || 0;
    const memoryPercent = totalMemory > 0 ? Math.min(100, Math.max(0, usedMemory / totalMemory * 100)) : 0;
    setText("#memory-label", totalMemory ? `${formatBytes(usedMemory)} / ${formatBytes(totalMemory)}` : "Unavailable");
    document.querySelector("#memory-bar").style.width = `${memoryPercent.toFixed(1)}%`;

    const storage = document.querySelector("#storage-list");
    storage.replaceChildren();
    const disks = Array.isArray(snapshot.disks) ? snapshot.disks : [];
    if (!disks.length) {
      const empty = document.createElement("span");
      empty.className = "muted";
      empty.textContent = "No storage volumes detected.";
      storage.append(empty);
      return;
    }

    for (const disk of disks) {
      const total = Number(disk.total_bytes) || 0;
      const available = Number(disk.available_bytes) || 0;
      const used = Math.max(0, total - available);
      const percent = total > 0 ? Math.min(100, used / total * 100) : 0;

      const row = document.createElement("div");
      row.className = "storage-row";
      const head = document.createElement("div");
      head.className = "storage-row-head";
      const name = document.createElement("b");
      name.textContent = disk.name || "Volume";
      const capacity = document.createElement("span");
      capacity.textContent = total ? `${formatBytes(available)} free of ${formatBytes(total)}` : "Unavailable";
      head.append(name, capacity);
      const track = document.createElement("div");
      track.className = "metric-track storage-track";
      const bar = document.createElement("span");
      bar.style.width = `${percent.toFixed(1)}%`;
      track.append(bar);
      row.append(head, track);
      storage.append(row);
    }
  } catch (error) {
    setText("#computer-name", "Unavailable");
    setText("#platform", "Unavailable");
    setText("#cpu-arch", "Unavailable");
    setText("#cpu-threads", "Thread count unavailable");
    setText("#memory-label", "Unavailable");
    document.querySelector("#memory-bar").style.width = "0%";
    const storage = document.querySelector("#storage-list");
    storage.textContent = "System information unavailable.";
  } finally {
    systemRefreshInFlight = false;
  }
}

document.querySelector("#open-start").addEventListener("click", () => toggleLauncher(true));
document.querySelector("#task-start").addEventListener("click", () => toggleLauncher());
document.querySelector("#launcher-close").addEventListener("click", () => toggleLauncher(false));
document.querySelector("#launcher").addEventListener("click", event => {
  if (event.target.id === "launcher") toggleLauncher(false);
});
document.querySelectorAll("[data-action]").forEach(element => {
  element.addEventListener("click", () => {
    const action = element.dataset.action;
    toggleLauncher(false);
    launch(action);
  });
});

document.querySelector("#gc-close").addEventListener("click", closeGameCenter);
document.querySelector("#gc-backdrop").addEventListener("click", closeGameCenter);
document.querySelector("#open-game-center").addEventListener("click", openGameCenter);
document.querySelectorAll("[data-gc-tab]").forEach(button => {
  button.addEventListener("click", () => setGameCenterTab(button.dataset.gcTab));
});
document.querySelector("#gc-scan").addEventListener("click", scanForGames);
document.querySelector("#gc-add-form").addEventListener("submit", event => {
  event.preventDefault();
  const name = document.querySelector("#gc-add-name").value.trim();
  const path = document.querySelector("#gc-add-path").value.trim();
  const profile = document.querySelector("#gc-add-profile").value;
  if (path) addGameToLibrary(name, path, profile);
});
document.querySelector("#gc-game-profile").addEventListener("change", event => {
  updateGameProfileDescription(event.target.value);
});
document.querySelector("#gc-save-profile").addEventListener("click", async () => {
  const game = selectedGame();
  if (!game) return;
  try {
    gameEntries = await invoke("set_game_profile", {
      id: game.id,
      profile: document.querySelector("#gc-game-profile").value
    });
    renderGameLibrary();
    toast("Game profile saved.");
  } catch (error) {
    toast(String(error));
  }
});
document.querySelector("#gc-launch-game").addEventListener("click", launchSelectedGame);
document.querySelector("#gc-remove-game").addEventListener("click", async () => {
  const game = selectedGame();
  if (!game) return;
  try {
    gameEntries = await invoke("remove_game", { id: game.id });
    selectedGameId = gameEntries[0]?.id ?? null;
    renderGameLibrary();
    toast("Removed from HADOC only. Original game files were not changed.");
  } catch (error) {
    toast(String(error));
  }
});
document.querySelector("#gc-run-benchmark").addEventListener("click", runHardwareBenchmark);
document.querySelector("#fm-close").addEventListener("click", closeFileManager);
document.querySelector("#fm-backdrop").addEventListener("click", closeFileManager);
document.querySelector("#fm-back").addEventListener("click", async () => {
  if (historyIndex <= 0) return;
  const targetIndex = historyIndex - 1;
  const success = await loadDirectory(directoryHistory[targetIndex], { recordHistory: false });
  if (success) {
    historyIndex = targetIndex;
    updateFileNavigation();
  }
});
document.querySelector("#fm-forward").addEventListener("click", async () => {
  if (historyIndex >= directoryHistory.length - 1) return;
  const targetIndex = historyIndex + 1;
  const success = await loadDirectory(directoryHistory[targetIndex], { recordHistory: false });
  if (success) {
    historyIndex = targetIndex;
    updateFileNavigation();
  }
});
document.querySelector("#fm-up").addEventListener("click", () => {
  if (currentParent) loadDirectory(currentParent);
});
document.querySelector("#fm-refresh").addEventListener("click", () => {
  if (currentDirectory) loadDirectory(currentDirectory, { recordHistory: false, query: currentQuery });
});
document.querySelector("#fm-path-form").addEventListener("submit", event => {
  event.preventDefault();
  const path = document.querySelector("#fm-path").value.trim();
  if (path) loadDirectory(path === "This PC" ? "__HADOC_THIS_PC__" : path);
});
document.querySelector("#fm-filter").addEventListener("input", () => {
  const query = document.querySelector("#fm-filter").value.trim();
  clearTimeout(filterTimer);
  filterTimer = setTimeout(() => {
    if (currentDirectory) loadDirectory(currentDirectory, { recordHistory: false, query });
  }, 180);
});
document.querySelector("#fm-entries").addEventListener("scroll", event => {
  const element = event.currentTarget;
  if (element.scrollTop + element.clientHeight >= element.scrollHeight - 120) {
    loadMoreDirectoryEntries();
  }
});

document.querySelector("#min").addEventListener("click", () => win.minimize());
document.querySelector("#max").addEventListener("click", () => win.toggleMaximize());
document.querySelector("#close").addEventListener("click", () => win.close());
document.querySelector("#gaming-mode").addEventListener("click", () => setGamingMode(!gamingMode));
document.querySelector("#full").addEventListener("click", async () => {
  await win.setFullscreen(!(await win.isFullscreen()));
});
document.addEventListener("keydown", event => {
  const target = event.target;
  const typing = target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement;
  if (fileManagerOpen && !typing && event.altKey && event.key === "ArrowLeft") {
    document.querySelector("#fm-back").click();
  } else if (fileManagerOpen && !typing && event.altKey && event.key === "ArrowRight") {
    document.querySelector("#fm-forward").click();
  } else if (fileManagerOpen && !typing && event.key === "Backspace" && currentParent) {
    document.querySelector("#fm-up").click();
  } else if (fileManagerOpen && !typing && (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "l") {
    event.preventDefault();
    const path = document.querySelector("#fm-path");
    path.focus();
    path.select();
  }
  if (event.key === "Escape") {
    if (gameCenterOpen) closeGameCenter();
    else if (fileManagerOpen) closeFileManager();
    else toggleLauncher(false);
  }
  if ((event.key === "Meta" || event.key === " ") && event.altKey) toggleLauncher();
});

function updateClock() {
  document.querySelector("#clock").textContent = new Intl.DateTimeFormat(undefined, {
    hour: "2-digit",
    minute: "2-digit"
  }).format(new Date());
}

updateClock();
setInterval(updateClock, 15000);
refreshSystemDashboard();
setInterval(() => {
  if (shouldRefreshDashboard({ documentHidden: document.hidden, gamingMode })) {
    refreshSystemDashboard();
  }
}, 20000);
document.addEventListener("visibilitychange", () => {
  if (shouldRefreshDashboard({ documentHidden: document.hidden, gamingMode })) {
    refreshSystemDashboard();
  }
});

