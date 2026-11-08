# HADOC Game Center and Hardware Benchmark Design

**Date:** 2026-11-08  
**Status:** Implementation in the feature branch.

## Product intent

The owner requested a one-pass delivery of Game Launcher, per-game performance profiles and a real hardware benchmark. This feature is a part of the Windows desktop shell, not a game compatibility layer or FPS booster.

## Game Center

- Local game library with name, existing Windows target path and HADOC profile.
- Game target types: .exe, .lnk and .url.
- Start Menu discovery is explicitly triggered by the user and limited to current-user and all-users Start Menu folders, maximum recursion depth five and maximum 1500 candidate items.
- Candidate filtering uses known game / launcher names; anything not auto-detected can be added manually by absolute path.
- Game library persists in app data as JSON.
- Removing an item removes only the HADOC library record; it never removes or edits the original game.
- Launch uses Windows shell association. Executable paths use their own parent folder as the working directory to support games that load assets through relative paths.
- Profile values: Balanced or Gaming. Gaming pauses HADOC dashboard monitoring and reduces HADOC visual effects; Balanced restores the standard shell. Neither changes Windows process priority, affinity, services, drivers, registry or anti-cheat state.

## Hardware benchmark

- Short integer-loop CPU sample: single worker and parallel sample (up to eight workers).
- Memory copy throughput from a bounded in-memory source/destination buffer.
- Sequential storage test using a 16 MiB temporary file in the system temp folder. Flush writes with sync_all, sequentially read it back, and remove the file even when an I/O operation fails.
- Report measured rates and elapsed time, with an explicit note that sequential read may use the Windows file cache.
- No GPU or game-FPS result is claimed.

## Performance constraints

- No new npm or Cargo dependency.
- Game discovery runs only on explicit user request; no whole-drive scanning.
- Benchmark runs in spawn_blocking and is activated only by the user.
- Existing low-resource and Gaming mode continue to pause dashboard polling.
- All file paths and labels are inserted as text, not HTML.
