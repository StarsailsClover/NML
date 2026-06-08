# NML Architecture

## Overview

NML is a next-generation Minecraft launcher built with Rust core and WinUI 3 interface.

## Architecture Layers

### 1. Core Layer (Rust)

Location: `nml-core/src/`

Modules:
- `version/`: Version management
- `download/`: Multi-source download engine
- `launch/`: Game launch system
- `account/`: Account management (Microsoft/Offline/Third-party)
- `p2p/`: P2P networking (FastLink/minecraftBC)
- `hook/`: API Hook for offline multiplayer fix
- `hot_swap/`: Identity hot-swap system
- `mcjebooster/`: MCJEBooster integration
- `minecraftbc/`: minecraftBC P2P integration
- `mnmcp/`: MnMCP Mini World integration
- `java/`: Java auto-detection and download
- `mod_manager/`: Mod management
- `server/`: Server management
- `error_analyzer/`: Crash analysis
- `sandbox/`: Sandbox launcher
- `upgrade/`: Version upgrade assistant
- `ffi.rs`: C ABI exports for C#

### 2. UI Layer (WinUI 3)

Location: `NML.Windows/`

Pages:
- `HomePage.xaml`: Version list and launch
- `DownloadPage.xaml`: Download management
- `MultiplayerPage.xaml`: P2P multiplayer
- `ModsPage.xaml`: Mod management
- `ServerPage.xaml`: Server management
- `SettingsPage.xaml`: Settings panel
- `AccountPage.xaml`: Account management

### 3. FFI Bridge

Location: `nml-core/src/ffi.rs` and `NML.Windows/Core/NMLCore.cs`

Binding between Rust and C# via C ABI.

## Technical Stack

| Layer | Technology |
|-------|------------|
| Core | Rust + Tokio |
| UI | WinUI 3 (.NET 9) |
| FFI | C ABI |
| Build | Cargo + MSBuild |

## Key Features

### 1.16-1.16.5 Offline Multiplayer Fix

Method: API Hook (Windows: InternetGetConnectedState)

Flow:
1. Hook network detection API
2. Return "offline" to Minecraft
3. Multiplayer button enabled
4. Restore network

### Identity Hot-Swap

Method: Java Agent injection + Reflection

Flow:
1. Inject Java Agent into running MC
2. Replace GameProfile via reflection
3. Update session service
4. Refresh player display

### P2P Multiplayer

Method: FastLink protocol (via minecraftBC)

Flow:
1. Create P2P node
2. Discover peers
3. Host world (local server + proxy)
4. Join world (P2P tunnel)
5. LAN injection to MC

## Data Flow

```
[WinUI 3] --FFI--> [Rust Core] --P2P--> [FastLink/minecraftBC]
                        |
                        +--> [MCJEBooster Agent]
                        |
                        +--> [Java Process]
```

## Version

v26.1-20240524-RC (Taipa)

Team: BlockConnect
Core Developer: StarsailsClover
