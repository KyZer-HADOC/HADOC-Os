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
- A separate shortcut for the native Windows File Explorer.

The built-in File Manager does not delete, move, rename, copy or edit files. It lists folders and files and opens selected items using their normal Windows application.

## Compatibility

HADOC OS is not a bootable operating system and does not emulate Windows. App and game compatibility is inherited from the host Windows installation, hardware, graphics drivers, runtime libraries and anti-cheat systems. Compatibility with every game or application is not guaranteed.

The initial target is **Windows 10 22H2 and Windows 11, x64**. The app uses the WebView2 runtime supplied by Windows or installed by the Tauri setup.

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
