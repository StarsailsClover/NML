//! NML - N0th1ngness Minecraft Launcher - CLI

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tokio::runtime::Runtime;

use nml_core::account::{AccountManager, AccountProvider};
use nml_core::launch::{LaunchConfig, LaunchEngine};
use nml_core::version::{DefaultVersionManager, ProgressCallback, VersionManager};

/// NML - Minecraft Launcher
#[derive(Parser)]
#[command(name = "nml", version, about)]
struct Cli {
    /// Data directory (default: ./nml-data)
    #[arg(short = 'd', long, default_value = "./nml-data")]
    data_dir: PathBuf,

    /// Game directory (.minecraft)
    #[arg(short = 'g', long, default_value = "./minecraft")]
    game_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List installed Minecraft versions
    List,
    /// List available remote versions
    ListRemote,
    /// Download and install a Minecraft version
    Install {
        /// Version ID (e.g. 1.20.4)
        version: String,
    },
    /// Launch Minecraft
    Launch {
        /// Version ID to launch
        version: String,
        /// Player name (for offline mode)
        #[arg(short = 'p', long, default_value = "Player")]
        player_name: String,
        /// Max memory in MB
        #[arg(long, default_value = "4096")]
        max_memory: u32,
    },
    /// List accounts
    AccountList,
    /// Add offline account
    AccountAdd {
        /// Player name
        username: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let data_dir = cli.data_dir.clone();
    let game_dir = cli.game_dir.clone();

    std::fs::create_dir_all(&data_dir).ok();
    std::fs::create_dir_all(&game_dir).ok();

    let rt = Runtime::new().expect("Failed to create tokio runtime");

    match cli.command {
        Commands::List => rt.block_on(cmd_list(&data_dir)),
        Commands::ListRemote => rt.block_on(cmd_list_remote(&data_dir)),
        Commands::Install { version } => rt.block_on(cmd_install(&version, &data_dir)),
        Commands::Launch { version, player_name, max_memory } => {
            rt.block_on(cmd_launch(&version, &player_name, max_memory, &data_dir, &game_dir))
        }
        Commands::AccountList => rt.block_on(cmd_account_list(&data_dir)),
        Commands::AccountAdd { username } => rt.block_on(cmd_account_add(&username, &data_dir)),
    }
}

async fn cmd_list(data_dir: &PathBuf) {
    let vm = DefaultVersionManager::new(data_dir.clone());

    match vm.get_installed_versions().await {
        Ok(versions) => {
            if versions.is_empty() {
                println!("No installed versions found.");
                println!("Use 'nml list-remote' to see available versions.");
                println!("Then: nml install <version>");
            } else {
                println!("Installed versions:");
                for v in &versions {
                    println!("  {}  (type: {})", v.id, v.info.version_type);
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn cmd_list_remote(data_dir: &PathBuf) {
    let vm = DefaultVersionManager::new(data_dir.clone());

    println!("Fetching version manifest...");
    match vm.get_remote_versions().await {
        Ok(manifest) => {
            println!("Available versions ({} total):", manifest.versions.len());

            let releases: Vec<_> = manifest.versions.iter()
                .filter(|v| v.version_type == nml_core::version::VersionType::Release)
                .take(20)
                .collect();
            let snapshots: Vec<_> = manifest.versions.iter()
                .filter(|v| v.version_type == nml_core::version::VersionType::Snapshot)
                .take(5)
                .collect();

            println!("\nLatest Releases:");
            for v in &releases {
                println!("  {} ({})", v.id, v.version_type);
            }
            if !snapshots.is_empty() {
                println!("\nLatest Snapshots:");
                for v in &snapshots {
                    println!("  {} ({})", v.id, v.version_type);
                }
            }
            println!("\nUse 'nml install <version>' to download a version.");
        }
        Err(e) => eprintln!("Error fetching manifest: {}", e),
    }
}

async fn cmd_install(version: &str, data_dir: &PathBuf) {
    let vm = DefaultVersionManager::new(data_dir.clone());

    // Check if already installed
    match vm.is_version_installed(version).await {
        Ok(true) => {
            println!("Version {} is already installed.", version);
            return;
        }
        Ok(false) => {}
        Err(e) => {
            eprintln!("Error checking version: {}", e);
            return;
        }
    }

    println!("Downloading Minecraft {}...", version);
    let progress: ProgressCallback = Box::new(|p: f32| {
        print!("\r  Progress: {:.1}%", p * 100.0);
        if p >= 1.0 {
            println!();
        }
    });

    match vm.install_version(version, progress).await {
        Ok(()) => {
            println!("\nVersion {} installed successfully!", version);
            println!("Launch with: nml launch {}", version);
        }
        Err(e) => eprintln!("\nError installing version: {}", e),
    }
}

async fn cmd_launch(version: &str, player_name: &str, max_memory: u32, data_dir: &PathBuf, game_dir: &PathBuf) {
    let vm = DefaultVersionManager::new(data_dir.clone());

    // Check if installed
    match vm.is_version_installed(version).await {
        Ok(false) => {
            eprintln!("Version {} is not installed.", version);
            eprintln!("Run: nml install {}", version);
            return;
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
        Ok(true) => {}
    }

    // Get or create offline account
    let mut account_mgr = AccountManager::new(data_dir.clone());
    account_mgr.load().await.ok();

    let account = if let Some(acc) = account_mgr.get_selected_account() {
        acc.clone()
    } else {
        let provider = nml_core::account::offline::OfflineProvider::new();
        let creds = nml_core::account::Credentials::offline(player_name.to_string());
        match provider.authenticate(creds).await {
            Ok(acc) => {
                println!("Created offline account: {}", acc.player_name);
                account_mgr.add_account(acc.clone()).await.ok();
                account_mgr.set_selected_account(&acc.id).await.ok();
                acc
            }
            Err(e) => {
                eprintln!("Failed to create account: {}", e);
                return;
            }
        }
    };

    println!("Launching Minecraft {} for player {}...", version, account.player_name);

    let launch_config = LaunchConfig {
        version_id: version.to_string(),
        player_name: account.player_name.clone(),
        access_token: account.access_token.clone(),
        uuid: Some(account.uuid.clone()),
        user_type: "mojang".to_string(),
        max_memory,
        min_memory: 512,
        jvm_args: Vec::new(),
        game_args: Vec::new(),
        window_width: None,
        window_height: None,
        fullscreen: false,
        server: None,
        game_dir: game_dir.clone(),
        java_path: None,
        enable_optimization: false,
    };

    let engine = LaunchEngine::new(data_dir.clone());
    match engine.launch(&launch_config).await {
        Ok(process) => {
            println!("Minecraft {} launched! (PID: {})", version, process.pid);
            let _ = process.wait().await;
            println!("Minecraft exited.");
        }
        Err(e) => {
            eprintln!("Failed to launch Minecraft: {}", e);
        }
    }
}

async fn cmd_account_list(data_dir: &PathBuf) {
    let mut account_mgr = AccountManager::new(data_dir.clone());
    account_mgr.load().await.ok();

    let accounts = account_mgr.get_accounts();
    if accounts.is_empty() {
        println!("No accounts configured.");
        println!("Accounts are auto-created on first launch.");
    } else {
        println!("Accounts:");
        for acc in accounts {
            let sel = if acc.is_selected { " [selected]" } else { "" };
            println!("  {} ({}){}", acc.player_name, acc.account_type.display_name(), sel);
        }
    }
}

async fn cmd_account_add(username: &str, data_dir: &PathBuf) {
    let mut account_mgr = AccountManager::new(data_dir.clone());
    account_mgr.load().await.ok();

    let provider = nml_core::account::offline::OfflineProvider::new();
    let creds = nml_core::account::Credentials::offline(username.to_string());

    match provider.authenticate(creds).await {
        Ok(account) => {
            account_mgr.add_account(account.clone()).await.ok();
            account_mgr.set_selected_account(&account.id).await.ok();
            println!("Offline account '{}' created and selected.", username);
        }
        Err(e) => eprintln!("Failed to create account: {}", e),
    }
}
