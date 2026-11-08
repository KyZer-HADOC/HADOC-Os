# HADOC OS — Ultra Performance and Gaming Profile Design

**Date:** 2026-11-08  
**Status:** Approved direction based on project owner's request for a very lightweight, smooth Windows desktop on old hardware.

## Product truth

HADOC OS is currently a desktop shell running on top of Windows, not a bootable replacement operating system. It cannot increase host CPU/GPU performance, bypass game requirements, provide missing graphics drivers, or guarantee a game's frame rate. Game compatibility and performance depend on Windows, hardware, graphics drivers, game APIs/runtime requirements, and anti-cheat.

A future bootable HADOC distribution would be a separate Linux/OS project with a different game compatibility story. Do not describe this Tauri desktop as a standalone OS.

## Target

- Windows 10 x64 (from Windows 10 1803 for the built-in WebView2 dependency) and Windows 11 x64.
- Keep the app lightweight enough to stay usable on dual-core / low-memory PCs where the host Windows itself can run it.
- Core 2 Duo class hardware is a compatibility goal for the HADOC shell, not a promise that modern Windows 11 or demanding games will run well.
- Windows 11 official eligibility depends on supported processor lists and additional requirements including TPM 2.0, Secure Boot/UEFI, memory, storage and GPU capabilities.

## Profile design

### Balanced
- Red/dark glass design remains enabled.
- Dashboard can refresh while visible.
- Smooth but restrained transitions.
- File Manager loads pages on demand.

### Gaming / Low Resource
- Disable ambient effects, large backdrop filters, animated glows, transitions and non-essential shadows.
- Hide the live dashboard and stop all periodic dashboard calls.
- Stop background File Manager preloading; do not scan libraries/processes.
- Retain a tiny launcher only when needed; never inject into games, manipulate game memory, or modify anti-cheat.
- Do not kill Windows services, terminate other user processes, change registry settings, force driver settings, change CPU priorities, or request admin rights.
- A manual exit returns to Balanced.
- On weak machines (<= 2 logical threads OR <= 4 GiB RAM), default to low-effects presentation; leave the user in control of full Gaming mode.

## Engineering constraints

- Preserve Tauri 2 + Rust + Vite + vanilla JS; no framework or large dependency.
- Keep the native Rust backend focused; no always-running monitor thread.
- Keep one dashboard collector and call it only when useful.
- Use event-driven state, no 1-second timers or polling when hidden.
- Avoid CSS backdrop filters and expensive infinite animations in low-resource mode.
- Avoid indexing the entire disk or reading process lists.
- Keep default app install small; no bundled browser, game runtime, emulator or graphics driver.

## Success criteria

- Front-end unit tests cover low-resource detection and dashboard refresh decisions.
- Gaming mode changes are reversible and have accessible toggle semantics.
- Performance mode can be activated and deactivated without reloading the app.
- Dashboard polling is suspended while hidden or in Gaming mode, and refreshed immediately on return to Balanced/visible.
- Low-resource mode removes expensive visual effects and reduces UI work.
- Existing File Manager navigation, file opening and Windows app shortcuts remain functional.
- Tests, Vite build, Rust tests and Windows installer all pass.
- No claims of extra FPS or guaranteed game compatibility.

## Future separate project: bootable edition

Only after the Windows desktop shell is stable:
- Evaluate a minimal Linux base rather than replacing Windows with a custom kernel.
- Keep hardware drivers and power management upstream.
- Offer a lightweight desktop/session and native game launchers.
- Use Proton/Wine where suitable, but clearly document anti-cheat and game compatibility limitations.
- Make a separate install image, recovery media and hardware test matrix. Do not overwrite a user's Windows installation.
