# HADOC OS

**A red, dark and glass-inspired desktop experience for Windows.**  
Created by **KyZer_HADOC**.

HADOC OS is a Windows desktop environment built with **Rust + Tauri 2 + Vite**. It runs on top of Windows rather than replacing the Windows kernel, drivers, DirectX, OpenGL or Vulkan stack. Your normal Windows applications and games continue to use the existing Windows environment.

## Current development status

HADOC OS is an early preview. The custom desktop shell is in progress and the first Windows integrations are being added in small, testable stages.

### Available in this preview

- Custom red, black and glass desktop interface.
- Windowed and immersive fullscreen modes.
- HADOC app launcher and taskbar.
- Quick launch for Windows Settings, Terminal, Task Manager, Notepad and the default browser.
- Built-in **read-only File Manager** with Home, common user folders, available drive roots, folder navigation, back/up controls, path entry, filename filtering and double-click open.
- Live system overview with computer name, Windows version, CPU architecture/thread count, RAM use and available storage volumes.
- **Gaming / Low Resource mode** pauses HADOC dashboard monitoring and removes nonessential blur, ambient effects and animations.
- Automatic low-effects presentation on systems with two or fewer logical threads or 4 GiB RAM or less.
- A separate shortcut for the native Windows File Explorer.
- **Game Center** with a saved local game library, Start Menu game-shortcut discovery, per-game Balanced/Gaming presets (the Gaming preset only reduces HADOC activity; it does not change game process priority) and a one-click game launcher.
- **Quick Hardware Benchmark** for measured single/multi-thread CPU throughput, memory copy bandwidth and sequential disk read/write. Temporary disk test files are removed.

The built-in File Manager does not delete, move, rename, copy or edit files. It lists folders and files and opens selected items using their normal Windows application.

### Performance and game compatibility

Gaming / Low Resource mode reduces work performed by HADOC itself. It does not overclock hardware, create extra CPU/GPU capacity, alter game launch priority, close Windows services, kill other processes, change registry settings or guarantee extra game FPS. Game performance remains dependent on the PC's CPU, GPU, RAM, storage, drivers and each game's requirements.

The File Manager is paginated and does not index the entire disk. Game discovery scans only the Windows Start Menu when requested; games can also be added using an executable or shortcut path. The local game library stores only names, paths and HADOC profile choices under the app's data folder.

System monitoring reuses collectors, refreshes storage less frequently than memory and pauses while the app is hidden or Gaming mode is active. The hardware benchmark runs only when manually requested, uses a short CPU/memory test and a 16 MiB temporary disk file, and deletes the file afterwards. Disk read results may include the Windows file cache.

Core 2 Duo-class systems are a low-resource compatibility goal for the HADOC shell when their installed Windows and WebView2 runtime are supported. This does **not** mean every modern game or Windows 11 can run on them. Windows 11 requires a supported processor and additional hardware requirements; most Core 2 Duo PCs are not officially supported. Windows 10 support ended on October 14, 2025, so a Windows 10 host no longer receives free security updates from Microsoft.

## Compatibility

HADOC OS is not a bootable operating system and does not emulate Windows. App and game compatibility is inherited from the host Windows installation, hardware, graphics drivers, runtime libraries and anti-cheat systems. Compatibility with every game or application is not guaranteed.

The target is **Windows 10 (version 1803 or later) and Windows 11, x64**, where WebView2 is available. The app uses the WebView2 runtime supplied by Windows or installed by the Tauri setup.

## Development

Requirements:
- Rust stable toolchain
- Node.js 22 or later + npm
- Microsoft C++ Build Tools (MSVC)
- WebView2 Runtime
- Tauri CLI (installed through the project dependencies)

Install dependencies and run the desktop app:

```bash
npm install
npm run tauri dev
```

Run front-end utility tests:

```bash
npm test
```

Build the Windows installer:

```bash
npm run tauri build
```

The GitHub Actions workflow tests the front-end utilities, builds the Vite frontend, runs Rust tests, and packages the Windows x64 NSIS installer.

## Project structure

- `src/` — desktop interface and styles
- `src-tauri/src/` — Rust commands and Windows integration
- `src-tauri/icons/` — application icon resources
- `tests/` — front-end utility tests

## License

MIT
