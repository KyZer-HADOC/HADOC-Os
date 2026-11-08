# HADOC OS Desktop Tools — Design

**Date:** 2026-11-08  
**Status:** Approved by the project owner's instruction to continue without a design pause.

## Intent

Move HADOC OS beyond its current landing-page prototype by adding useful desktop tools while preserving the existing Rust + Tauri 2 Windows desktop application, red/dark glass visual language, and small-installation goal.

HADOC OS remains a desktop environment running on top of Windows. It is not a bootable operating system and does not replace Windows drivers, application compatibility, or security boundaries.

## Goals

1. Add an in-app File Manager overlay with quick locations, drive roots, directory navigation, parent/back navigation, filename filtering, file metadata, and opening files/folders through the Windows shell.
2. Add a live system overview showing computer name, Windows version, CPU architecture/thread count, RAM usage, and available storage volumes.
3. Refresh the system overview periodically without blocking the UI.
4. Keep the existing start launcher, window controls, fullscreen mode, and quick-launch behavior working.
5. Fix the Windows CI workflow so it does not require a missing npm lockfile before installing dependencies, and run Rust tests before packaging.

## Non-goals

- Replacing Windows or booting independently.
- File deletion, editing, copying, moving, renaming, or formatting.
- Process termination, registry editing, privilege elevation, or command-shell execution.
- Guaranteeing support for every Windows game or anti-cheat system.

## Approaches considered

1. **Built-in Tauri File Manager and native Rust filesystem commands (chosen).** Gives HADOC a cohesive in-app experience and lets the UI control navigation and styling. The Rust layer handles filesystem access and opening paths; the front end remains lightweight.
2. **Launch Windows Explorer only.** Lowest implementation cost, but leaves the current app mostly a shortcut launcher and does not create the integrated HADOC experience.
3. **Embed a third-party file-manager component.** Adds dependencies and integration complexity without providing enough value for this first phase.

The built-in approach is chosen because it adds useful functionality without introducing a large framework or another native application dependency.

## Architecture

- Keep Tauri 2 as the desktop bridge and register narrowly scoped Rust commands.
- Put filesystem behavior in `src-tauri/src/file_manager.rs` and system information in `src-tauri/src/system_info.rs`.
- Use Rust standard-library filesystem APIs for read-only directory listing and Windows ShellExecute for opening a selected path. Canonicalize requested paths and return clear errors for missing paths or non-directories.
- Use the `sysinfo` crate for OS, CPU architecture, memory and disk information. Do not collect or display process names.
- Build the File Manager as an overlay inside the existing desktop window; preserve the top bar and taskbar.
- Render filenames as text, not trusted HTML. Directory names and paths may contain special characters.
- Keep the dashboard readable on smaller windows and use periodic refresh for live memory and disk values.

## Constraints and safety

- Target: Windows 10 and Windows 11 on x64.
- Continue using Rust + Tauri 2 + Vite + vanilla JavaScript/CSS.
- Keep runtime dependencies limited; do not add a UI framework.
- File browsing is read-only. No destructive filesystem command is exposed.
- Open paths through the operating system's default shell association without interpolating paths into a command string.
- Do not request administrator privileges.
- Keep the existing red, black, and glass UI treatment.

## Error handling

- Invalid, missing, inaccessible, or non-directory paths produce a readable in-app error.
- A directory entry whose metadata cannot be read is skipped rather than crashing the entire listing.
- If a standard quick location is absent, omit it.
- System-information values that Windows or the host cannot provide display as “Unavailable” rather than breaking the desktop.

## Success criteria

- The Windows workflow installs npm dependencies, runs the front-end test suite, builds the Vite frontend, runs Rust tests, and produces the NSIS installer.
- Rust tests cover directory listing order, file metadata, and rejection of non-directory paths.
- The desktop UI exposes the built-in File Manager and current system snapshot.
- No file modification or deletion feature is introduced in this phase.
