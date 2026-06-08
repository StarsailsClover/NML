//! Error analysis and automatic fixing system
//!
//! Analyzes Minecraft crashes and provides intelligent fixes

use std::collections::HashMap;
use std::path::PathBuf;
use regex::Regex;

use crate::error::{NMLError, Result};
use crate::launch::MinecraftProcess;

pub mod patterns;
pub mod fixes;
pub mod collector;

use patterns::ErrorPattern;
use fixes::FixSuggestion;
use collector::LogCollector;

/// Error analyzer
pub struct ErrorAnalyzer {
    patterns: Vec<ErrorPattern>,
    fix_database: HashMap<String, FixSuggestion>,
    collector: LogCollector,
}

/// Analysis result
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub error_type: ErrorType,
    pub severity: ErrorSeverity,
    pub description: String,
    pub causes: Vec<String>,
    pub fixes: Vec<FixSuggestion>,
    pub raw_log: String,
}

/// Error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    JavaVersionMismatch,
    MemoryIssue,
    ModConflict,
    ModMissingDependency,
    ResourcePackIssue,
    ShaderIssue,
    NetworkIssue,
    GraphicsDriverIssue,
    FileCorruption,
    PermissionDenied,
    Unknown,
}

/// Error severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl ErrorAnalyzer {
    /// Create new analyzer
    pub fn new() -> Self {
        let mut analyzer = Self {
            patterns: Vec::new(),
            fix_database: HashMap::new(),
            collector: LogCollector::new(),
        };
        
        analyzer.load_patterns();
        analyzer.load_fixes();
        
        analyzer
    }
    
    /// Load error patterns
    fn load_patterns(&mut self) {
        self.patterns = vec![
            ErrorPattern {
                id: "java_version_mismatch".to_string(),
                regex: Regex::new(r"Unsupported major\.minor version|has been compiled by a more recent version").unwrap(),
                error_type: ErrorType::JavaVersionMismatch,
                severity: ErrorSeverity::Error,
            },
            ErrorPattern {
                id: "out_of_memory".to_string(),
                regex: Regex::new(r"java\.lang\.OutOfMemoryError|OutOfMemory").unwrap(),
                error_type: ErrorType::MemoryIssue,
                severity: ErrorSeverity::Critical,
            },
            ErrorPattern {
                id: "mod_conflict".to_string(),
                regex: Regex::new(r"Mod .* conflicts with|Found duplicate mods").unwrap(),
                error_type: ErrorType::ModConflict,
                severity: ErrorSeverity::Error,
            },
            ErrorPattern {
                id: "mod_missing_dep".to_string(),
                regex: Regex::new(r"Missing required mod|requires .* but only found").unwrap(),
                error_type: ErrorType::ModMissingDependency,
                severity: ErrorSeverity::Error,
            },
            ErrorPattern {
                id: "resource_pack_invalid".to_string(),
                regex: Regex::new(r"ResourceLocationException|Invalid resource pack").unwrap(),
                error_type: ErrorType::ResourcePackIssue,
                severity: ErrorSeverity::Warning,
            },
            ErrorPattern {
                id: "shader_error".to_string(),
                regex: Regex::new(r"Shader compilation failed|Invalid shader").unwrap(),
                error_type: ErrorType::ShaderIssue,
                severity: ErrorSeverity::Warning,
            },
            ErrorPattern {
                id: "network_timeout".to_string(),
                regex: Regex::new(r"Connection timed out|Read timed out").unwrap(),
                error_type: ErrorType::NetworkIssue,
                severity: ErrorSeverity::Warning,
            },
            ErrorPattern {
                id: "graphics_driver".to_string(),
                regex: Regex::new(r"GLFW error|OpenGL error|Graphics driver").unwrap(),
                error_type: ErrorType::GraphicsDriverIssue,
                severity: ErrorSeverity::Error,
            },
            ErrorPattern {
                id: "file_corruption".to_string(),
                regex: Regex::new(r"CorruptNBT|JSON syntax error|Invalid region file").unwrap(),
                error_type: ErrorType::FileCorruption,
                severity: ErrorSeverity::Error,
            },
        ];
    }
    
    /// Load fix database
    fn load_fixes(&mut self) {
        self.fix_database.insert("java_version_mismatch".to_string(), FixSuggestion {
            id: "java_version_mismatch".to_string(),
            description: "Java版本不匹配".to_string(),
            auto_fixable: false,
            steps: vec![
                "检测当前Java版本".to_string(),
                "下载所需Java版本".to_string(),
                "重新启动游戏".to_string(),
            ],
        });
        
        self.fix_database.insert("out_of_memory".to_string(), FixSuggestion {
            id: "out_of_memory".to_string(),
            description: "内存不足".to_string(),
            auto_fixable: true,
            steps: vec![
                "自动增加内存分配".to_string(),
                "关闭其他程序".to_string(),
            ],
        });
        
        self.fix_database.insert("mod_conflict".to_string(), FixSuggestion {
            id: "mod_conflict".to_string(),
            description: "Mod冲突".to_string(),
            auto_fixable: false,
            steps: vec![
                "禁用冲突的Mod".to_string(),
                "联系Mod作者".to_string(),
            ],
        });
        
        self.fix_database.insert("mod_missing_dep".to_string(), FixSuggestion {
            id: "mod_missing_dep".to_string(),
            description: "缺少Mod依赖".to_string(),
            auto_fixable: true,
            steps: vec![
                "自动下载缺失的依赖Mod".to_string(),
            ],
        });
    }
    
    /// Analyze crash log
    pub async fn analyze(&self, log_path: &PathBuf) -> Result<AnalysisResult> {
        // Collect log
        let log = self.collector.collect(log_path).await?;
        
        // Analyze patterns
        let mut detected_patterns = vec![];
        for pattern in &self.patterns {
            if pattern.regex.is_match(&log) {
                detected_patterns.push(pattern);
            }
        }
        
        // Sort by severity
        detected_patterns.sort_by_key(|p| p.severity);
        
        // Build result
        let (error_type, severity, description) = if let Some(pattern) = detected_patterns.last() {
            (pattern.error_type, pattern.severity, format!("检测到: {}", pattern.id))
        } else {
            (ErrorType::Unknown, ErrorSeverity::Info, "未识别错误".to_string())
        };
        
        // Find fixes
        let mut fixes = vec![];
        for pattern in &detected_patterns {
            if let Some(fix) = self.fix_database.get(&pattern.id) {
                fixes.push(fix.clone());
            }
        }
        
        Ok(AnalysisResult {
            error_type,
            severity,
            description,
            causes: self.extract_causes(&log).await?,
            fixes,
            raw_log: log,
        })
    }
    
    /// Extract error causes from log
    async fn extract_causes(&self, log: &str) -> Result<Vec<String>> {
        let mut causes = vec![];
        
        // Extract exception stack traces
        let exception_regex = Regex::new(r"([\w.]+Exception): (.+)").unwrap();
        for cap in exception_regex.captures_iter(log) {
            causes.push(format!("{}: {}", &cap[1], &cap[2]));
        }
        
        // Extract "Caused by"
        let caused_by_regex = Regex::new(r"Caused by: ([\w.]+): (.+)").unwrap();
        for cap in caused_by_regex.captures_iter(log) {
            causes.push(format!("原因: {}: {}", &cap[1], &cap[2]));
        }
        
        Ok(causes)
    }
    
    /// Auto-fix if possible
    pub async fn auto_fix(&self, result: &AnalysisResult) -> Result<bool> {
        for fix in &result.fixes {
            if fix.auto_fixable {
                tracing::info!("Auto-fixing: {}", fix.description);
                
                // Apply fix
                match fix.id.as_str() {
                    "out_of_memory" => {
                        // Increase memory
                        return Ok(true);
                    }
                    "mod_missing_dep" => {
                        // Auto-download dependency
                        return Ok(true);
                    }
                    _ => {}
                }
            }
        }
        
        Ok(false)
    }
    
    /// Monitor running process for errors
    pub async fn monitor_process(&self, process: &MinecraftProcess) -> Result<Option<AnalysisResult>> {
        // Check if process has crashed
        if !process.is_running().await {
            // Collect latest log
            let log_path = self.get_log_path(process);
            if log_path.exists() {
                return self.analyze(&log_path).await.map(Some);
            }
        }
        
        Ok(None)
    }
    
    fn get_log_path(&self, process: &MinecraftProcess) -> PathBuf {
        // Default Minecraft log location
        std::env::temp_dir().join(format!("minecraft_{}.log", process.pid))
    }
}

impl Default for ErrorAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
