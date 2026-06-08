# NML - N0th1ngness Minecraft Launcher

A next-generation Minecraft Launcher with P2P multiplayer, performance optimization, and hot-swap identity switching.

## Features

### Core
- Version management with multi-source download (Official/BMCLAPI/MCBBS)
- Account system (Microsoft/Offline/Third-party)
- Java auto-detection and download
- P2P multiplayer without public IP (via minecraftBC)
- Offline multiplayer fix for 1.16-1.16.5 (API Hook method)
- Identity hot-swap without game restart
- MCJEBooster performance optimization integration
- MnMCP Mini World cross-play support

### Technical Stack
- Core: Rust with Tokio async runtime
- Windows UI: WinUI 3 (native Windows experience)
- FFI: C ABI binding between Rust and C#

## Building

### Requirements
- Rust 1.75+
- .NET 9 SDK
- Windows SDK 10.0.22621+

### Build Commands

```powershell
# Build Rust Core
cd nml-core
cargo build --release

# Build Windows UI
cd ../NML.Windows
dotnet publish -c Release -r win-x64 --self-contained
```

## Version

Current: v26.1-20240524-RC (Taipa)

Version format: v{YY}.{N}-{YYYYMMDD}-{Type}
- YY: Last two digits of year (26 for 2026)
- N: Commit count within major version (1-50)
- YYYYMMDD: Date
- Type: RC/Stable/Preview

Major version "Taipa" (v26.0 to v26.49)

## License

MIT License - See LICENSE file

## Credits

Core Developer: StarsailsClover
Team: BlockConnect

Integrated Projects:
- MCJEBooster by StarsailsClover (Performance optimization)
- minecraftBC by StarsailsClover (P2P networking)
- MnMCP by StarsailsClover (Mini World bridge)
