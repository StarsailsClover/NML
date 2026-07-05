# NML Version Specification

Full Name: N0th1ngness Minecraft Launcher
Current Major Version: Taipa (v26.0 - v26.49)

## Version Format

v{YY}.{N}-{YYYYMMDD}-{TYPE}

Components:
- YY: Last two digits of year (e.g., 26 for 2026)
- N: Commit count within major version (1-50)
- YYYYMMDD: 8-digit date
- TYPE: RC (Release Candidate) / Stable / Preview

## Major Version Rule

50 commits = 1 major version
Major versions are named after cities alphabetically.

Current: Taipa (v26.0 to v26.49)
Next: (v26.50 to v26.99) - next city name

## Version History

### v26.0-Alpha.1 (Current)

- Type: Alpha
- Date: 2026-07-05
- Commit: Flutter migration baseline
- Status: Alpha Preview

Core Features (32 Rust modules):
- Version management
- Multi-source download
- Account system (Microsoft/Offline/Third-party)
- 1.16-1.16.5 offline multiplayer fix (API Hook)
- Microsoft login cache with Grace Period
- Identity hot-swap (exclusive feature)
- MCJEBooster integration
- minecraftBC P2P multiplayer
- MnMCP Mini World cross-play
- Error analysis system
- Sandbox launcher
- Version upgrade assistant
- Server management
- Mod management
- Java auto-download

WinUI 3 Interface (7 pages):
- Home page
- Download management
- Multiplayer
- Mod management
- Server management
- Settings
- Account management

Integrated Projects:
- StarsailsClover/MCJEBooster
- StarsailsClover/minecraftBC
- StarsailsClover/MnMCP

## Build Requirements

- Rust 1.75+
- .NET 9 SDK
- Windows SDK 10.0.22621+
- Visual Studio 2026 Preview (optional)

## Build Instructions

```powershell
# Build Rust Core
cd nml-core
cargo build --release

# Build Windows UI
cd NML.Windows
dotnet publish -c Release -r win-x64 --self-contained
```

## License

MIT License

Team: BlockConnect
Core Developer: StarsailsClover
