# HADOC OS Ultra Performance and Gaming Profile Plan

**Goal:** Minimize HADOC's own CPU, RAM and GPU overhead and make the interface comfortable on old dual-core Windows PCs, while preserving Windows application/game compatibility.

**Spec:** `docs/superpowers/specs/2026-11-08-ultra-performance-gaming-profile-design.md`

## Constraints

- Tauri desktop shell on Windows 10/11 x64, not a bootable OS.
- Do not claim to increase hardware power or guarantee FPS.
- No process killing, service disabling, registry tweaks, CPU affinity/priority forcing, admin rights, game injection or anti-cheat interference.
- No extra front-end framework or large dependencies.

## Tasks

### Task 1 — Add and test a low-resource profile classifier

Files:
- Modify `src/ui-utils.js`
- Modify `tests/ui-utils.test.js`

- [ ] Add a test that 2 logical threads select low-effects mode.
- [ ] Add a test that <= 4 GiB RAM selects low-effects mode.
- [ ] Add a test that a newer device stays in balanced presentation mode.
- [ ] Add tests for dashboard refresh being suppressed while hidden or while Gaming mode is active.
- [ ] Run tests first and observe RED.
- [ ] Implement small pure helpers and observe GREEN.

### Task 2 — Add a reversible Gaming / Low Resource toggle

Files:
- Modify `index.html` if necessary
- Modify `src/main.js`
- Modify `src/style.css`

- [ ] Add accessible toggle button in the desktop interface.
- [ ] On activation, apply `performance-mode` class, hide nonessential dashboard/widgets, disable animated ambient effects and blur/shadows, and stop dashboard refresh.
- [ ] On deactivation, restore Balanced UI and immediately refresh system information if the window is visible.
- [ ] Show current mode and use `aria-pressed`.
- [ ] Keep app launch and File Manager available.
- [ ] Preserve user choice for the current app session without localStorage or other unapproved persistent storage.
- [ ] Respect `prefers-reduced-motion` too.

### Task 3 — Audit resource use

Files:
- Modify `src/main.js`, `src/style.css`, Rust code only when an observed hotspot warrants it.

- [ ] Remove redundant timers, polling or duplicate refresh paths.
- [ ] Ensure dashboard has at most one in-flight refresh.
- [ ] Avoid refreshing system info when document is hidden or Gaming mode is active.
- [ ] Ensure file listing remains paginated and does not render more than the received page.
- [ ] Review for full-disk scans, process enumeration and expensive visual effects.
- [ ] Do not optimize by disabling Windows OS services or security features.

### Task 4 — Document performance tiers and compatibility

Files:
- Modify `README.md`.

- [ ] Explain what Gaming / Low Resource mode does and does not do.
- [ ] Clarify that game performance depends on the host PC and game requirements.
- [ ] State the Windows 10/11 targets and minimum supported WebView2 platform.
- [ ] Explain that Core 2 Duo compatibility is a shell target only and Windows 11 is not officially supported on most Core 2 Duo machines.
- [ ] Keep installer lean; do not bundle graphics drivers, runtimes or a browser.

### Task 5 — Verify

- [ ] Run `npm test`.
- [ ] Run `npm run build`.
- [ ] Run Rust tests.
- [ ] Build Windows installer and verify artifact.
- [ ] Review actual CI logs before claiming success.
