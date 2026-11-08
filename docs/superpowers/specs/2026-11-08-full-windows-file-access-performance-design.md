# HADOC OS — Full Windows File Access and Performance Design

**Date:** 2026-11-08  
**Status:** Approved by project owner request to continue implementation.

## Goal

Make HADOC OS File Manager browse the real Windows filesystem, not just a small set of default user folders, while keeping the desktop responsive on modest Windows 10/11 PCs.

## User-visible behavior

- Open File Manager at **This PC**, showing every currently assigned Windows drive letter (fixed, removable and mapped/network drives).
- Browse drive roots and every directory/file the current Windows user is permitted to read, including hidden/system folders when accessible.
- Quick locations include Home, Desktop, Documents, Downloads, Pictures, Music, Videos, OneDrive locations, AppData, Local AppData, Temp, Public, Program Files, Program Files (x86), ProgramData and Windows where those paths exist.
- Redirected or missing quick folders are handled gracefully; the list includes paths that actually exist rather than assuming the default profile layout.
- The folder path bar accepts normal Windows paths such as `C:\Users\<user>\AppData\Roaming`.
- Back, Forward, Up, Refresh and double-click open continue to work.
- Folder listing is read-only. Opening an item uses its Windows shell association.
- Hidden items are included and visibly identified. Permission-denied and disconnected locations show a readable error, not an application crash.
- Filtering searches the complete current directory, not only the currently rendered page.

## Performance requirements

- File listings are paginated. Load an initial page and fetch more only as the user scrolls.
- Sort directory names first and case-insensitively without gathering full file metadata for every entry.
- Avoid rebuilding sysinfo collectors on every dashboard refresh; reuse collectors and refresh only the needed data.
- Refresh the dashboard only while the window is visible; refresh disk data less frequently than memory.
- Use no new front-end framework and no heavy file-manager package.
- Avoid network font downloads and retain native Windows fonts.
- Filesystem scans and other potentially blocking I/O run off the UI event loop.
- Do not request administrator privileges or promise access to protected locations that Windows denies.

## Scope

Only Windows 10 22H2 / Windows 11 x64 are release targets. Existing launch, window controls, app launcher, fullscreen and red/dark/glass design remain supported.

## Implementation decisions

1. Use the Windows `GetLogicalDrives` API to retrieve assigned drive letters instead of probing all 26 drive letters one-by-one. Keep the drive mask conversion as a pure, tested function.
2. Represent This PC as a virtual root in the File Manager, with drive roots as directory entries.
3. Build quick locations from existing Windows environment paths and profile-relative fallback folders; normalize/deduplicate paths case-insensitively on Windows.
4. Use backend page parameters (`offset`, `limit`) and full-directory query filtering.
5. Include Windows hidden-attribute files and conventional dot-hidden names; never filter them out.
6. Keep the read-only boundary: no file editing, deletion, rename, move, copy, format or privilege elevation in this pass.
7. Keep one `sysinfo::System` and one `sysinfo::Disks` collector alive and refresh only the values needed by the dashboard.

## Success checks

- Rust tests cover drive-mask conversion, This PC virtual roots, existing quick-location filtering/deduplication, paging/order, hidden-file metadata and query filtering.
- Front-end tests cover pagination state and filtering/escaping helpers.
- Vite build and Rust tests pass.
- GitHub Actions produces the Windows x64 NSIS installer.
