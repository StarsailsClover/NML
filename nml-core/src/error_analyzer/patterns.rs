//! Error patterns (stub)

use serde::{Deserialize, Serialize};
use super::{ErrorType, ErrorSeverity};

#[derive(Debug, Clone)]
pub struct ErrorPattern {
    pub id: String,
    pub regex: regex::Regex,
    pub error_type: ErrorType,
    pub severity: ErrorSeverity,
}
