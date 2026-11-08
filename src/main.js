import "./style.css";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

const app = document.querySelector("#app");
const win = getCurrentWindow();
let startOpen = false;

const apps = [
  { name: "File Explorer", icon: "▤", desc: "Browse your files", action: "explorer" },
  { name: "Settings", icon: "⚙", desc: "Windows settings", action: "settings" },
  { name: "Terminal", icon: ">_", desc: "Command line", action: "terminal" },
  { name: "Task Manager", icon: "▥", desc: "System processes", action: "taskmgr" },
  { name: "Notepad", icon: "▧", desc: "Quick notes", action: "notepad" },
  { name: "Browser", icon: "◎", desc: "Open your default browser", action: "browser" }
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
        <div class="hero-actions"><button class="primary" id="open-start">Open launcher <b>↗</b></button><button class="ghost" id="full">Immersive mode ⛶</button></div>
      </div>
      <div class="orb-wrap"><div class="orb-ring ring-a"></div><div class="orb-ring ring-b"></div><div class="orb"><span>H</span></div><div class="orb-caption">DESIGNED TO FEEL<br><strong>LIKE YOURS.</strong></div></div>
    </section>
    <section class="widgets">
      <article class="widget system-widget"><div class="widget-head"><span>01 / SYSTEM</span><span class="status"><i></i> ONLINE</span></div><h2>Your PC, at a glance.</h2><div class="system-data"><div><small>PLATFORM</small><strong id="platform">Windows</strong></div><div><small>EDITION</small><strong>HADOC Preview</strong></div><div><small>ENGINE</small><strong>Rust + Tauri</strong></div></div></article>
      <article class="widget quick-widget"><div class="widget-head"><span>02 / QUICK ACCESS</span><span>OPEN</span></div><div class="quick-grid">${apps.slice(0,4).map(a=>`<button class="quick-app" data-action="${a.action}"><span>${a.icon}</span><b>${a.name}</b></button>`).join("")}</div></article>
    </section>
    <div class="launcher hidden" id="launcher"><div class="launcher-card"><div class="launcher-top"><div><small>HADOC OS / APPS</small><h2>What do you need?</h2></div><button id="launcher-close">×</button></div><div class="app-list">${apps.map(a=>`<button class="app-row" data-action="${a.action}"><span class="app-icon">${a.icon}</span><span><b>${a.name}</b><small>${a.desc}</small></span><i>↗</i></button>`).join("")}</div></div></div>
    <footer class="taskbar"><button class="start-button" id="task-start"><span class="mini-mark">H</span><span>HADOC</span></button><div class="taskbar-divider"></div><span class="taskbar-label">DESKTOP</span><div class="taskbar-right"><span class="host-label">RUNNING ON WINDOWS</span><span id="clock"></span></div></footer>
    <div class="toast" id="toast"></div>
  </main>`;

const toast = (message) => {
  const el = document.querySelector("#toast");
  el.textContent = message; el.classList.add("show");
  setTimeout(() => el.classList.remove("show"), 2600);
};

async function launch(action) {
  try {
    const result = await invoke("launch_windows_app", { app: action });
    toast(result || "Opened.");
  } catch (e) { toast(String(e)); }
}

function toggleLauncher(force) {
  startOpen = typeof force === "boolean" ? force : !startOpen;
  document.querySelector("#launcher").classList.toggle("hidden", !startOpen);
}
document.querySelector("#open-start").addEventListener("click", () => toggleLauncher(true));
document.querySelector("#task-start").addEventListener("click", () => toggleLauncher());
document.querySelector("#launcher-close").addEventListener("click", () => toggleLauncher(false));
document.querySelector("#launcher").addEventListener("click", e => { if (e.target.id === "launcher") toggleLauncher(false); });
document.querySelectorAll("[data-action]").forEach(el => el.addEventListener("click", () => { toggleLauncher(false); launch(el.dataset.action); }));
document.querySelector("#min").addEventListener("click", () => win.minimize());
document.querySelector("#max").addEventListener("click", () => win.toggleMaximize());
document.querySelector("#close").addEventListener("click", () => win.close());
document.querySelector("#full").addEventListener("click", async () => { await win.setFullscreen(!(await win.isFullscreen())); });
document.addEventListener("keydown", e => { if (e.key === "Escape") toggleLauncher(false); if (e.key === "Meta" || e.key === " ") { if (e.altKey) toggleLauncher(); } });

function updateClock() {
  document.querySelector("#clock").textContent = new Intl.DateTimeFormat(undefined, {hour:"2-digit",minute:"2-digit"}).format(new Date());
}
updateClock(); setInterval(updateClock, 15000);
invoke("get_platform_info").then(v => document.querySelector("#platform").textContent = v).catch(() => {});
