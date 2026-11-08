# HADOC OS Full Windows File Access and Performance Plan

> **For agentic workers:** Continue implementation inline with test-driven development. Steps use checkbox syntax.

**Goal:** Make HADOC File Manager show and browse the real Windows filesystem, including assigned drives and hidden/system locations, without stalling the desktop on large folders.

**Architecture:** Introduce a virtual This PC root and real Windows drive discovery, build a curated list of existing Windows shell/system locations, and return folder listings in sorted pages. Keep system collectors alive and refresh selectively; make the UI load more pages only when needed.

**Tech Stack:** Rust, Tauri 2, Windows Kernel32 API, sysinfo 0.39.6, vanilla JavaScript/CSS, Node built-in tests.

**Spec:** `docs/superpowers/specs/2026-11-08-full-windows-file-access-performance-design.md`

## Global Constraints

- Windows 10 22H2 and Windows 11 x64.
- No admin rights, no full OS replacement, no file mutation features.
- No UI framework or heavy File Manager dependency.
- Maintain red/dark/glass visuals and low background overhead.

## Review Focus

1. Disconnected mapped/removable drives must not hang discovery; test pure drive-mask mapping and make inaccessible entries fail gracefully on navigation.
2. Very large directories must not create a huge DOM at once; test pagination and load-more state.
3. Hidden/system paths are discoverable but access-denied locations must show clear errors; test hidden flag and error handling.
4. Folder names and paths with Unicode, ampersands and angle brackets must remain plain text; test plain-text rendering helpers.
5. Hidden or redirected user profile folders must not break quick access; test location filtering/deduplication.

---

### Task 1: Add Windows drive and shell-location discovery

**Files:**
- Modify: `src-tauri/src/file_manager.rs`

**Interfaces:**
- `this_pc_path() -> &'static str`
- `logical_drive_roots(mask: u32) -> Vec<PathBuf>`
- `get_quick_locations() -> Vec<QuickLocation>`
- `this_pc_listing() -> DirectoryListing`

- [ ] **Step 1: Write Rust tests** for drive-mask conversion (C and E assigned, D absent), virtual This PC listing, and quick-location inclusion/deduplication using temporary directories.
- [ ] **Step 2: Run the Rust test suite** and observe the new tests fail because the new functions/fields are not implemented.
- [ ] **Step 3: Implement** the pure helpers and Windows `GetLogicalDrives` call. Map only set mask bits to drive roots; do not probe unused letters.
- [ ] **Step 4: Build quick locations** from existing profile/system environment paths, include OneDrive/AppData/Temp/Windows/Program Files/Public paths when present, and deduplicate case-insensitively.
- [ ] **Step 5: Change** `list_directory(None)` to open the virtual This PC root; selecting a drive opens the actual root.
- [ ] **Step 6: Run Rust tests** and commit as `feat: discover Windows drives and shell folders`.

### Task 2: Add hidden-file metadata, sorting and paged directory reads

**Files:**
- Modify: `src-tauri/src/file_manager.rs`

**Interfaces:**
- `list_directory(path, offset, limit, query)`
- `DirectoryListing { path, parent, entries, total_entries, next_offset, has_more }`
- `FileEntry { name, path, is_dir, is_hidden, size, modified, extension }`

- [ ] **Step 1: Write Rust tests** for hidden flags, directory-first global ordering across pages, file sizes, extension, full-directory query filtering and empty directories.
- [ ] **Step 2: Run Rust tests** and observe the new assertions fail before implementation.
- [ ] **Step 3: Implement** a two-pass listing: collect names and directory flags, sort them, filter the full name set, then read metadata only for the requested page.
- [ ] **Step 4: Enforce** a bounded page size (default 200, maximum 500) and return total/next-offset/has-more values.
- [ ] **Step 5: Make filesystem listing asynchronous** through Tauri's blocking-task executor so slow folder I/O does not block window interaction.
- [ ] **Step 6: Run Rust tests** and commit as `perf: paginate Windows directory listings`.

### Task 3: Make system monitoring lightweight

**Files:**
- Modify: `src-tauri/src/system_info.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Managed `SystemInfoState` retains a single `System` and `Disks` collector.
- `get_system_snapshot(state) -> Result<SystemSnapshot, String>`

- [ ] **Step 1: Write Rust tests** for refresh-throttle decisions and snapshot values with missing optional metadata.
- [ ] **Step 2: Run Rust tests** and observe the throttle test fail before implementation.
- [ ] **Step 3: Keep** collectors in Tauri managed state; refresh memory on request and storage data at most once every 30 seconds.
- [ ] **Step 4: Register state** during setup and return safe errors if collector locks are poisoned.
- [ ] **Step 5: Run Rust tests** and commit as `perf: reuse system collectors`.

### Task 4: Connect This PC, paging and full-folder filtering in the UI

**Files:**
- Modify: `src/main.js`
- Modify: `src/style.css`
- Modify: `src/ui-utils.js`
- Modify: `tests/ui-utils.test.js`

**Interfaces:**
- File Manager opens at This PC.
- Sidebar groups Computer, User, Windows and Drives.
- Toolbar supports Back, Forward, Up, Refresh and path entry.
- Backend query filters the entire current folder.
- Infinite scrolling requests the next page only near the list bottom.
- Hidden entries carry a visible marker.

- [ ] **Step 1: Add front-end tests** for page accumulator state (no duplicates, correct next offset, stop at end) and safe label formatting.
- [ ] **Step 2: Run `npm test`** and observe the new tests fail before implementation.
- [ ] **Step 3: Implement** page state helpers and wire backend paging/search into File Manager.
- [ ] **Step 4: Add** Forward navigation and keyboard shortcuts (Ctrl+L, Alt+Left/Right, Backspace outside inputs).
- [ ] **Step 5: Ensure** virtual This PC, drive roots, hidden/system quick locations, readable errors and responsive layouts.
- [ ] **Step 6: Refresh dashboard every 20 seconds only while visible**; re-fetch immediately when the window becomes visible.
- [ ] **Step 7: Run `npm test`, `npm run build` and the Windows workflow**; commit as `feat: browse the complete Windows filesystem`.

### Task 5: Review compatibility and document the feature

**Files:**
- Modify: `README.md`
- Modify: `.github/workflows/build-windows.yml` only if verification finds a real CI issue.

- [ ] **Step 1: Verify** the latest Windows workflow's front-end tests, Vite build, Rust tests, NSIS build and uploaded artifact.
- [ ] **Step 2: Review** for unbounded DOM updates, repeated system collector allocation, synchronous folder reads, missing-path crashes and unsafe HTML insertion.
- [ ] **Step 3: Update README** with actual supported File Manager behavior and limitations.
- [ ] **Step 4: Commit** as `docs: document complete Windows file access`.

---
