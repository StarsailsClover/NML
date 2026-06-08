//! Java version detection

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{NMLError, Result};
use super::JavaInstance;

/// Detect Java at specific path
pub async fn detect_java_at(path: &Path) -> Result<Option<JavaInstance>> {
    let java_exe = if cfg!(target_os = "windows") {
        path.join("bin").join("java.exe")
    } else {
        path.join("bin").join("java")
    };
    
    if !java_exe.exists() {
        return Ok(None);
    }
    
    // Run java -version
    let output = Command::new(&java_exe)
        .arg("-version")
        .output()?;
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Parse version from output
    let version = parse_java_version(&stderr)?;
    let major_version = parse_major_version(&version)?;
    
    let is_64bit = stderr.contains("64-Bit") || stderr.contains("64-bit");
    
    Ok(Some(JavaInstance {
        version,
        major_version,
        path: path.to_path_buf(),
        distribution: infer_distribution(path),
        is_64bit,
        detected_at: chrono::Utc::now(),
    }))
}

/// Detect system Java
pub async fn detect_system_java() -> Result<Option<JavaInstance>> {
    // Check JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let path = PathBuf::from(java_home);
        if let Some(java) = detect_java_at(&path).await? {
            return Ok(Some(java));
        }
    }
    
    // Check PATH
    if let Ok(output) = Command::new("java").arg("-version").output() {
        if output.status.success() {
            // Find java executable location
            if let Ok(output) = Command::new("which").arg("java").output() {
                let path = String::from_utf8_lossy(&output.stdout);
                let path = PathBuf::from(path.trim());
                if let Some(parent) = path.parent() {
                    if let Some(bin) = parent.parent() {
                        return detect_java_at(bin).await;
                    }
                }
            }
        }
    }
    
    // Check common locations
    let common_paths = get_common_java_locations();
    for path in common_paths {
        if let Some(java) = detect_java_at(&path).await? {
            return Ok(Some(java));
        }
    }
    
    Ok(None)
}

/// Get common Java installation locations
pub fn get_common_java_locations() -> Vec<PathBuf> {
    let mut locations = vec![];
    
    #[cfg(target_os = "windows")]
    {
        // Windows common locations
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            locations.push(PathBuf::from(program_files).join("Java"));
        }
        if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
            locations.push(PathBuf::from(program_files_x86).join("Java"));
        }
        locations.push(PathBuf::from("C:\\Program Files\\Java"));
        locations.push(PathBuf::from("C:\\Program Files (x86)\\Java"));
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux common locations
        locations.push(PathBuf::from("/usr/lib/jvm"));
        locations.push(PathBuf::from("/usr/java"));
        locations.push(PathBuf::from("/opt/java"));
        locations.push(PathBuf::from("~/.sdkman/candidates/java"));
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS common locations
        locations.push(PathBuf::from("/Library/Java/JavaVirtualMachines"));
        locations.push(PathBuf::from("/System/Library/Java/JavaVirtualMachines"));
        locations.push(PathBuf::from("/usr/lib/java"));
    }
    
    locations
}

/// Parse Java version string
fn parse_java_version(output: &str) -> Result<String> {
    // Parse "java version \"1.8.0_312\"" or \"17.0.1\" or \"21.0.1\" etc
    if let Some(start) = output.find('"') {
        if let Some(end) = output[start + 1..].find('"') {
            return Ok(output[start + 1..start + 1 + end].to_string());
        }
    }
    
    Err(NMLError::Other("Failed to parse Java version".to_string()))
}

/// Parse major version from full version
fn parse_major_version(version: &str) -> Result<u8> {
    let parts: Vec<&str> = version.split('.').collect();
    
    if parts.len() >= 2 {
        if parts[0] == "1" {
            // Old format: 1.8.0_xxx -> 8
            parts[1].parse().map_err(|_| NMLError::Other("Invalid version".to_string()))
        } else {
            // New format: 17.0.1 -> 17
            parts[0].parse().map_err(|_| NMLError::Other("Invalid version".to_string()))
        }
    } else {
        Err(NMLError::Other("Invalid version format".to_string()))
    }
}

/// Infer distribution from path
fn infer_distribution(path: &Path) -> super::JavaDistribution {
    let path_str = path.to_string_lossy().to_lowercase();
    
    if path_str.contains("adopt") || path_str.contains("temurin") {
        super::JavaDistribution::Adoptium
    } else if path_str.contains("oracle") {
        super::JavaDistribution::Oracle
    } else if path_str.contains("microsoft") {
        super::JavaDistribution::Microsoft
    } else if path_str.contains("corretto") {
        super::JavaDistribution::Amazon
    } else if path_str.contains("zulu") {
        super::JavaDistribution::Azul
    } else if path_str.contains("dragonwell") {
        super::JavaDistribution::Alibaba
    } else if path_str.contains("kona") {
        super::JavaDistribution::Tencent
    } else if path_str.contains("bisheng") {
        super::JavaDistribution::BiSheng
    } else {
        super::JavaDistribution::Adoptium // Default
    }
}
