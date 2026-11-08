# HADOC Game Center and Hardware Benchmark Plan

**Goal:** Build one lightweight, local-first Game Center with a saved game library, per-game HADOC profiles, on-demand game shortcut discovery and real CPU/memory/storage measurements.

**Architecture:** Rust manages a small JSON game library in the app data directory, discovers known game shortcuts only under the two Windows Start Menu folders when requested, and launches existing local EXE/LNK/URL targets via Windows ShellExecute. A separate Rust benchmark runs on a blocking worker and returns measured rates. Vanilla JS renders the library and benchmark results; no extra package dependencies.

## Scope and safety

- No whole-drive game search or automatic indexing.
- No game downloads, installers, cracks or key/license handling.
- No process injection, game memory access, anti-cheat modification, service termination, registry edits, priority/affinity forcing, overclocking or driver tweaks.
- Per-game presets control HADOC appearance and dashboard polling only.
- The benchmark measures CPU integer-loop throughput, memory copy bandwidth and sequential storage I/O; it is not a GPU benchmark or FPS predictor.
- A 16 MiB temporary disk test file is flushed and removed after the test. Sequential read measurements may include the OS file cache.

## Tasks

### Task 1 — Game library model and persistence

- [x] Validate EXE, LNK and URL targets.
- [x] Restrict profile values to Balanced and Gaming.
- [x] De-duplicate library items by normalized path.
- [x] Save game name, path and preset choice in the HADOC app data folder.
- [x] Add, list, update-profile, remove-entry and launch commands.
- [x] Add Start Menu shortcut discovery with a bounded depth and item count.

### Task 2 — Hardware benchmark engine

- [x] Add single-thread CPU throughput measurement.
- [x] Add multi-thread CPU throughput measurement with a bounded worker count.
- [x] Add memory copy bandwidth measurement.
- [x] Add sequential disk write and read measurements.
- [x] Run the benchmark on Tauri's blocking worker, not the UI thread.
- [x] Remove temporary benchmark data.
- [x] Document cached-read limitations and explicitly exclude GPU/FPS claims.

### Task 3 — Game Center UI

- [x] Add Game Center to the HADOC launcher and main desktop.
- [x] Add game library list, manual path entry and on-demand Start Menu detection.
- [x] Add per-game Balanced/Gaming profile selection and persistence.
- [x] Launch saved entries and switch HADOC's own Gaming Mode according to the selected profile.
- [x] Remove a library entry without touching the original game file.
- [x] Add Hardware Benchmark tab and measured result cards.
- [x] Keep visual effects low in low-resource and Gaming modes.

### Task 4 — Verification and docs

- [x] Add front-end tests for profile labels and benchmark metric formatting.
- [x] Add Rust tests for path validation, shortcut candidate filtering, profile validation, path de-duplication and persistence.
- [x] Add Rust tests for CPU, memory and disk benchmark operations and temporary file cleanup.
- [x] Update README with actual capability boundaries.
- [ ] Verify full Windows CI: utility tests, Vite build, Rust tests, NSIS build and artifact upload.
- [ ] Review latest CI logs and report any remaining issue honestly.
