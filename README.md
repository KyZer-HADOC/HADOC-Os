# HADOC OS

**A red, dark and glass-inspired desktop experience for Windows.**  
Created by **KyZer_HADOC**.

HADOC OS is a Windows desktop environment built with Rust + Tauri 2. It runs *on top of Windows* rather than replacing the Windows kernel, drivers, DirectX, OpenGL or Vulkan stack. This keeps normal Windows applications and games running through the existing Windows environment.

## Current status
Early development — the initial desktop shell and Windows integration foundation.

## Goals
- Custom dark/red glass desktop
- Windowed and immersive fullscreen modes
- Start menu, taskbar, desktop shortcuts and app launcher
- Lightweight native Rust backend
- Keep the installer and runtime footprint as small as practical
- Windows 10 22H2 and Windows 11 x64

## Important compatibility note
HADOC OS does not promise that every game or application will work. Compatibility is inherited from the host Windows installation, drivers, runtime libraries, anti-cheat systems and hardware. HADOC OS does not emulate or replace Windows.

## Development
Requirements:
- Rust stable toolchain
- Node.js LTS + npm
- Microsoft C++ Build Tools (MSVC)
- WebView2 Runtime
- Tauri CLI

Install dependencies:
```bash
npm install
npm run tauri dev
```

Build:
```bash
npm run tauri build
```

The release workflow also builds a Windows x64 installer on GitHub Actions.

## License
MIT
