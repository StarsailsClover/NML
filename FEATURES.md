# NML Feature Specification

## Core Features

### Tier 0: Essential

#### Version Management
- Version manifest retrieval (Official/BMCLAPI/MCBBS)
- Version download and installation
- Version integrity verification
- Version deletion management
- Java auto-detection
- Java auto-download (Adoptium)
- JVM parameter optimization
- Game process management
- Launch configuration persistence

#### Account System
- Microsoft OAuth login
- Offline accounts (local)
- Third-party authentication (Authlib-Injector)
- Account cache system
- Server status monitoring
- Offline fallback login

#### Download System
- Multi-source parallel download
- Resume support
- Chunked multi-threading
- Download queue management
- Automatic mirror selection

#### Problem Solutions
- 1.16-1.16.5 offline multiplayer fix (API Hook)
- Microsoft server failure fallback (cache system)
- Crash log analysis

### Tier 1: Extended

#### ModLoader Support
- Forge installation
- Fabric installation
- NeoForge installation
- Quilt installation
- OptiFine installation
- ModLoader version detection

#### Mod Management
- CurseForge API integration
- Modrinth API integration
- Search and browse mods
- One-click installation
- Auto-update mods
- Conflict detection
- Dependency resolution
- Enable/disable mods

#### Resource Management
- Resource pack management
- Shader pack management
- Skin upload and preview

#### Integration Packs
- CurseForge pack import
- Modrinth pack import
- FTB pack support
- MultiMC format support
- Pack export

### Tier 2: Advanced

#### MCJEBooster Integration
- Automatic adapter selection
- JVM argument optimization injection
- Runtime performance monitoring
- TPS/FPS display
- Multi-core tick optimization
- Memory leak detection

#### minecraftBC Integration
- World creation/hosting
- World discovery/browsing
- Join P2P worlds
- LAN injection
- Friend system

#### MnMCP Integration
- Mini World protocol bridge
- Cross-game multiplayer
- Data mapping conversion

#### Server Management
- Local server creation
- Server download (Vanilla/Forge/Fabric)
- Server configuration editor
- Server start/stop
- Player management (OP/ban)
- Whitelist management
- Plugin management
- Server log viewing
- Performance monitoring
- Auto-backup

### Tier 3: Exclusive

#### Identity Hot-Swap
- In-game username change (offline)
- Offline <-> Microsoft switching
- Microsoft <-> Skin site switching
- Instant skin refresh
- Single-player/multiplayer support

#### Sandbox Launcher
- Isolated environment
- Error capture
- Intelligent error analysis
- Auto-fix suggestions
- Retry on failure
- Release on success

#### Version Upgrade Assistant
- Config migration detection
- Setting migration
- Mod compatibility check
- Resource pack conversion
- Save upgrade tool

### Tier 4: AI & Automation

#### AI Integration
- LLM API integration
- AI assistant chat
- Smart error diagnosis
- Mod recommendations
- JVM parameter AI optimization

#### Agent Control
- MCP protocol support
- AI-controlled launch
- Auto-configuration installation
- Self-diagnosis

#### Automation
- Scheduled backups
- Auto-update mods
- Config cloud sync
- Scheduled tasks

## UI Pages (WinUI 3)

1. HomePage - Version list and launch
2. DownloadPage - Download management
3. MultiplayerPage - P2P multiplayer
4. ModsPage - Mod management
5. ServerPage - Server management
6. SettingsPage - Settings
7. AccountPage - Account management

## FFI Functions

Version management, download, launch, account, P2P, MCJEBooster, hot-swap, server management.

## Version

v26.1-20240524-RC (Taipa)

Team: BlockConnect
Core Developer: StarsailsClover
