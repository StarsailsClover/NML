//! Java detector for launch

use std::path::PathBuf;
use crate::error::{NMLError, Result};

pub struct JavaDetector;

impl JavaDetector {
    pub fn new() -> Self { Self }

    pub async fn find_java(&self, _required_version: i32) -> Result<PathBuf> {
        let exe_name = if cfg!(windows) { "java.exe" } else { "java" };

        // Try JAVA_HOME first
        if let Ok(home) = std::env::var("JAVA_HOME") {
            let p = PathBuf::from(&home);
            let exe = p.join("bin").join(exe_name);
            if exe.exists() {
                return Ok(exe);
            }
        }

        // Try PATH
        if let Ok(paths) = std::env::var("PATH") {
            for path in std::env::split_paths(&paths) {
                let exe = path.join(exe_name);
                if exe.exists() {
                    return Ok(exe);
                }
            }
        }

        Err(NMLError::JavaNotFound(8))
    }
}
