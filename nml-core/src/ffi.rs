//! FFI exports for C# interop
//!
//! Provides C-compatible interface for WinUI 3

use std::ffi::{c_char, CStr, CString};
use std::ptr;

use crate::config::NMLConfig;
use crate::error::NMLError;

/// Opaque handle to NML Core
pub struct NMLCoreHandle {
    config: NMLConfig,
}

/// Initialize NML Core
/// 
/// # Safety
/// - config_path: null-terminated UTF-8 string or null for default
/// - Returns null on error
#[no_mangle]
pub unsafe extern "C" fn nml_init(config_path: *const c_char) -> *mut NMLCoreHandle {
    let result = if config_path.is_null() {
        NMLConfig::load()
    } else {
        match CStr::from_ptr(config_path).to_str() {
            Ok(s) => NMLConfig::load_from(std::path::PathBuf::from(s)),
            Err(_) => return ptr::null_mut(),
        }
    };
    
    match result {
        Ok(config) => {
            let handle = NMLCoreHandle { config };
            Box::into_raw(Box::new(handle))
        }
        Err(e) => {
            eprintln!("NML init failed: {}", e);
            ptr::null_mut()
        }
    }
}

/// Shutdown NML Core
/// 
/// # Safety
/// - handle must be valid pointer from nml_init
#[no_mangle]
pub unsafe extern "C" fn nml_shutdown(handle: *mut NMLCoreHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// Get last error message
/// 
/// # Safety
/// - Returns null-terminated string, must be freed with nml_free_string
#[no_mangle]
pub unsafe extern "C" fn nml_get_last_error() -> *mut c_char {
    // Static error storage
    static mut LAST_ERROR: Option<String> = None;
    
    if let Some(ref msg) = LAST_ERROR {
        match CString::new(msg.as_str()) {
            Ok(s) => s.into_raw(),
            Err(_) => ptr::null_mut(),
        }
    } else {
        ptr::null_mut()
    }
}

/// Free string returned by NML
/// 
/// # Safety
/// - s must be string returned by NML function
#[no_mangle]
pub unsafe extern "C" fn nml_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

// ============================================================================
// Version Management
// ============================================================================

/// Get installed versions as JSON array
/// 
/// # Safety
/// - handle: valid NMLCoreHandle
/// - callback: receives JSON string, null on error
#[no_mangle]
pub unsafe extern "C" fn nml_version_get_installed(
    handle: *mut NMLCoreHandle,
    callback: extern "C" fn(*const c_char),
) {
    if handle.is_null() {
        callback(ptr::null());
        return;
    }
    
    // Mock data for now
    let versions = vec!["1.20.1", "1.19.4", "1.18.2"];
    let json = serde_json::to_string(&versions).unwrap_or_default();
    
    match CString::new(json) {
        Ok(s) => callback(s.into_raw()),
        Err(_) => callback(ptr::null()),
    }
}

/// Get remote versions from manifest
#[no_mangle]
pub unsafe extern "C" fn nml_version_get_remote(
    handle: *mut NMLCoreHandle,
    callback: extern "C" fn(*const c_char),
) {
    if handle.is_null() {
        callback(ptr::null());
        return;
    }
    
    // Mock remote versions
    let versions = vec![
        ("1.20.4", "release"),
        ("24w05a", "snapshot"),
    ];
    let json = serde_json::to_string(&versions).unwrap_or_default();
    
    match CString::new(json) {
        Ok(s) => callback(s.into_raw()),
        Err(_) => callback(ptr::null()),
    }
}

/// Install a version
/// 
/// # Safety
/// - version_id: null-terminated UTF-8 string
/// - progress_callback: receives 0.0-1.0
#[no_mangle]
pub unsafe extern "C" fn nml_version_install(
    handle: *mut NMLCoreHandle,
    version_id: *const c_char,
    progress_callback: extern "C" fn(f32),
) -> i32 {
    if handle.is_null() || version_id.is_null() {
        return -1;
    }
    
    let version = match CStr::from_ptr(version_id).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    // Simulate installation
    for i in 0..=10 {
        progress_callback(i as f32 / 10.0);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    0 // Success
}

/// Uninstall a version
#[no_mangle]
pub unsafe extern "C" fn nml_version_uninstall(
    handle: *mut NMLCoreHandle,
    version_id: *const c_char,
) -> i32 {
    if handle.is_null() || version_id.is_null() {
        return -1;
    }
    
    0 // Success
}

// ============================================================================
// Launch Management
// ============================================================================

/// Launch Minecraft
/// 
/// # Safety
/// - version_id: null-terminated string
/// - player_name: null-terminated string
#[no_mangle]
pub unsafe extern "C" fn nml_launch(
    handle: *mut NMLCoreHandle,
    version_id: *const c_char,
    player_name: *const c_char,
    is_offline: bool,
) -> i32 {
    if handle.is_null() || version_id.is_null() || player_name.is_null() {
        return -1;
    }
    
    let version = match CStr::from_ptr(version_id).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    let player = match CStr::from_ptr(player_name).to_str() {
        Ok(s) => s,
        Err(_) => return -3,
    };
    
    println!("Launching {} for player {} (offline={})", version, player, is_offline);
    
    0 // Success
}

/// Kill running Minecraft
#[no_mangle]
pub unsafe extern "C" fn nml_kill_minecraft(handle: *mut NMLCoreHandle) -> i32 {
    if handle.is_null() {
        return -1;
    }
    
    0 // Success
}

// ============================================================================
// Account Management
// ============================================================================

/// Add offline account
/// 
/// # Safety
/// - username: null-terminated string
/// - Returns JSON account info
#[no_mangle]
pub unsafe extern "C" fn nml_account_add_offline(
    handle: *mut NMLCoreHandle,
    username: *const c_char,
    callback: extern "C" fn(*const c_char),
) {
    if handle.is_null() || username.is_null() {
        callback(ptr::null());
        return;
    }
    
    let name = match CStr::from_ptr(username).to_str() {
        Ok(s) => s,
        Err(_) => {
            callback(ptr::null());
            return;
        }
    };
    
    // Generate offline UUID
    let uuid = format!("offline-{}-uuid", name);
    
    let account = serde_json::json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "type": "offline",
        "username": name,
        "uuid": uuid,
    });
    
    match CString::new(account.to_string()) {
        Ok(s) => callback(s.into_raw()),
        Err(_) => callback(ptr::null()),
    }
}

/// Get all accounts
#[no_mangle]
pub unsafe extern "C" fn nml_account_get_all(
    handle: *mut NMLCoreHandle,
    callback: extern "C" fn(*const c_char),
) {
    if handle.is_null() {
        callback(ptr::null());
        return;
    }
    
    let accounts: Vec<serde_json::Value> = vec![];
    let json = serde_json::to_string(&accounts).unwrap_or_default();
    
    match CString::new(json) {
        Ok(s) => callback(s.into_raw()),
        Err(_) => callback(ptr::null()),
    }
}

// ============================================================================
// P2P Multiplayer
// ============================================================================

/// Start P2P node
#[no_mangle]
pub unsafe extern "C" fn nml_p2p_start(
    handle: *mut NMLCoreHandle,
    config_json: *const c_char,
) -> i32 {
    if handle.is_null() {
        return -1;
    }
    
    println!("P2P node started");
    0
}

/// Stop P2P node
#[no_mangle]
pub unsafe extern "C" fn nml_p2p_stop(handle: *mut NMLCoreHandle) -> i32 {
    if handle.is_null() {
        return -1;
    }
    
    println!("P2P node stopped");
    0
}

/// Discover P2P worlds
#[no_mangle]
pub unsafe extern "C" fn nml_p2p_discover(
    handle: *mut NMLCoreHandle,
    callback: extern "C" fn(*const c_char),
) {
    if handle.is_null() {
        callback(ptr::null());
        return;
    }
    
    let worlds: Vec<serde_json::Value> = vec![];
    let json = serde_json::to_string(&worlds).unwrap_or_default();
    
    match CString::new(json) {
        Ok(s) => callback(s.into_raw()),
        Err(_) => callback(ptr::null()),
    }
}

/// Host a world
#[no_mangle]
pub unsafe extern "C" fn nml_p2p_host(
    handle: *mut NMLCoreHandle,
    world_name: *const c_char,
    local_port: u16,
    callback: extern "C" fn(*const c_char),
) {
    if handle.is_null() || world_name.is_null() {
        callback(ptr::null());
        return;
    }
    
    let world_id = uuid::Uuid::new_v4().to_string();
    
    match CString::new(world_id) {
        Ok(s) => callback(s.into_raw()),
        Err(_) => callback(ptr::null()),
    }
}

// ============================================================================
// Download
// ============================================================================

/// Download file
#[no_mangle]
pub unsafe extern "C" fn nml_download_file(
    handle: *mut NMLCoreHandle,
    url: *const c_char,
    destination: *const c_char,
    progress_callback: extern "C" fn(f32),
) -> i32 {
    if handle.is_null() || url.is_null() || destination.is_null() {
        return -1;
    }
    
    // Simulate download
    for i in 0..=10 {
        progress_callback(i as f32 / 10.0);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    
    0
}

// ============================================================================
// MCJEBooster
// ============================================================================

/// Enable MCJEBooster optimization
#[no_mangle]
pub unsafe extern "C" fn nml_mcje_enable(
    handle: *mut NMLCoreHandle,
    mc_version: *const c_char,
) -> i32 {
    if handle.is_null() || mc_version.is_null() {
        return -1;
    }
    
    let version = match CStr::from_ptr(mc_version).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    println!("MCJEBooster enabled for {}", version);
    0
}

/// Get performance stats
#[no_mangle]
pub unsafe extern "C" fn nml_mcje_get_stats(
    handle: *mut NMLCoreHandle,
    callback: extern "C" fn(*const c_char),
) {
    if handle.is_null() {
        callback(ptr::null());
        return;
    }
    
    let stats = serde_json::json!({
        "tps": 20.0,
        "mspt": 50.0,
        "entity_count": 1000,
        "optimized": true,
    });
    
    match CString::new(stats.to_string()) {
        Ok(s) => callback(s.into_raw()),
        Err(_) => callback(ptr::null()),
    }
}

// ============================================================================
// Version Info
// ============================================================================

/// Get NML version
#[no_mangle]
pub extern "C" fn nml_version() -> *const c_char {
    const VERSION: &str = "0.1.0\0";
    VERSION.as_ptr() as *const c_char
}
