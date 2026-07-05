//! Fix suggestion types (stub)

#[derive(Debug, Clone)]
pub struct FixSuggestion {
    pub id: String,
    pub description: String,
    pub auto_fixable: bool,
    pub steps: Vec<String>,
}
