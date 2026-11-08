# HADOC OS File Manager and System Dashboard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an in-app read-only File Manager and live Windows system overview, while repairing CI so the project can build and test reliably.

**Architecture:** Add focused Rust modules for filesystem access and system telemetry, expose narrow Tauri commands, and render the File Manager as an overlay in the existing vanilla-JS desktop. Use the standard library for filesystem access and `sysinfo` for basic host information.

**Tech Stack:** Rust, Tauri 2, Vite, vanilla JavaScript/CSS, Node's built-in test runner, `sysinfo`.

**Spec:** `docs/superpowers/specs/2026-11-08-file-manager-system-dashboard-design.md`

## Global Constraints

- Target Windows 10 / Windows 11 x64.
- Keep Rust + Tauri 2 + Vite + vanilla JavaScript/CSS.
- Do not add a front-end framework.
- File browsing is read-only; do not add delete, edit, copy, move, rename, or format actions.
- Do not request administrator privileges or collect process names.
- Preserve the existing red, dark, glass visual style and window controls.

## Review Focus

1. A path that does not exist or points to a file must return a useful error, not crash the app. Test both conditions in the filesystem module.
2. Filenames containing HTML-sensitive characters must be rendered as plain text, not interpreted as markup. Test the escaping helper.
3. A quick location that is absent on a Windows profile must not make the File Manager fail. Test location filtering or handle it explicitly.
4. An empty directory and a directory with mixed folders/files must render stable empty/sorted states. Test directory listing.
5. A host returning missing system metadata must still yield a usable snapshot. Keep fields optional/defaultable and render “Unavailable”.

---

### Task 1: Repair the Windows build workflow and add front-end tests

**Files:**
- Modify: `.github/workflows/build-windows.yml`
- Modify: `package.json`
- Create: `tests/ui-utils.test.js`
- Create: `src/ui-utils.js`

**Interfaces:**
- Produces `escapeHtml(value: unknown) -> string` and `formatBytes(value: number | null | undefined) -> string`.
- The test command is `npm test`.

- [ ] **Step 1: Write failing tests** for HTML escaping of `<tag attr="x">` and for byte formatting at 0, 1024, and missing values.
- [ ] **Step 2: Run `npm test`** and confirm it fails because the utility module/script is not implemented.
- [ ] **Step 3: Implement** the two small utility functions in `src/ui-utils.js`; add `"test": "node --test"` to `package.json`.
- [ ] **Step 4: Run `npm test`** and confirm all utility tests pass.
- [ ] **Step 5: Update workflow** to remove npm caching while no lockfile exists, trigger on main and feature branches, and run `npm test`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1`, then `npm run tauri build`.
- [ ] **Step 6: Commit** as `test: add UI utilities and repair Windows CI`.

### Task 2: Add tested read-only filesystem commands

**Files:**
- Create: `src-tauri/src/file_manager.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- `list_directory(path: Option<String>) -> Result<DirectoryListing, String>`
- `get_quick_locations() -> Vec<QuickLocation>`
- `open_path(path: String) -> Result<(), String>`
- `DirectoryListing { path: String, parent: Option<String>, entries: Vec<FileEntry> }`
- `FileEntry { name: String, path: String, is_dir: bool, size: Option<u64>, modified: Option<u64> }`
- `QuickLocation { name: String, path: String, icon: String, group: String }`

- [ ] **Step 1: Write failing Rust tests** for folders-first sorting and file sizes in a temporary directory, plus rejection of a regular file as a directory.
- [ ] **Step 2: Run `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1`** and confirm the tests fail because the filesystem helper is not implemented.
- [ ] **Step 3: Implement** a private `read_directory(path: &Path) -> Result<DirectoryListing, String>` helper. Canonicalize the path, require a directory, skip entries with unreadable metadata, and sort directories before files (case-insensitive name order).
- [ ] **Step 4: Implement** the three Tauri commands. Default `None` to the current user's profile folder; include existing Home/Desktop/Documents/Downloads/Pictures/Music/Videos locations and existing Windows drive roots; open paths through Windows ShellExecute rather than a command string.
- [ ] **Step 5: Register commands** in `tauri::generate_handler!`.
- [ ] **Step 6: Run the Rust test command** and confirm it passes.
- [ ] **Step 7: Commit** as `feat: add safe read-only file manager commands`.

### Task 3: Add a live system snapshot command

**Files:**
- Create: `src-tauri/src/system_info.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- `get_system_snapshot() -> SystemSnapshot`
- `SystemSnapshot { computer_name: Option<String>, os_version: Option<String>, cpu_arch: Option<String>, logical_threads: usize, total_memory: u64, used_memory: u64, disks: Vec<DiskSnapshot> }`
- `DiskSnapshot { name: String, total_bytes: u64, available_bytes: u64 }`

- [ ] **Step 1: Add a Rust test** proving byte values in a snapshot formatter are not needed for UI correctness; instead test serialization shape and ensure missing optional host values are accepted.
- [ ] **Step 2: Run Rust tests** and confirm the new snapshot test fails before the type/command exists.
- [ ] **Step 3: Add `sysinfo = "0.39.6"`** and implement the snapshot using memory refresh plus `Disks::new_with_refreshed_list()`; do not enumerate processes.
- [ ] **Step 4: Register `get_system_snapshot`** and run Rust tests.
- [ ] **Step 5: Commit** as `feat: expose Windows system information`.

### Task 4: Integrate the File Manager and live system overview

**Files:**
- Modify: `src/main.js`
- Modify: `src/style.css`
- Modify: `README.md`

**Interfaces:**
- The UI invokes `list_directory`, `get_quick_locations`, `open_path`, and `get_system_snapshot`.
- The File Manager supports opening/closing, quick locations, drive roots, parent/back navigation, path entry, local filename filtering, and double-click open.
- The system card displays computer, OS, CPU threads/architecture, RAM usage, and volume space.

- [ ] **Step 1: Add File Manager navigation markup and event wiring** without changing the existing window controls or start launcher.
- [ ] **Step 2: Use `escapeHtml`** for all file names/paths inserted into markup and `formatBytes` for all byte values.
- [ ] **Step 3: Add system snapshot rendering** and refresh it every 10 seconds; show “Unavailable” for absent fields.
- [ ] **Step 4: Add responsive File Manager/dashboard styles** matching the red/dark/glass interface.
- [ ] **Step 5: Update README** to describe the Windows desktop application, current File Manager/system features, and the fact that HADOC OS runs on top of Windows.
- [ ] **Step 6: Run `npm test`, `npm run build`, and the Windows GitHub Actions workflow**; confirm the installer artifact exists.
- [ ] **Step 7: Commit** as `feat: integrate HADOC file manager and system dashboard`.

---
