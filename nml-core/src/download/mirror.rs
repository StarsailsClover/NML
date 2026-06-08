//! Mirror configuration for downloads

/// Mirror definition
#[derive(Debug, Clone)]
pub struct Mirror {
    /// Mirror name
    pub name: String,
    /// Base URL
    pub base_url: String,
    /// URL mappings (original -> mirror path)
    pub mappings: Vec<(String, String)>,
}

/// Mirror status
#[derive(Debug, Clone)]
pub enum MirrorStatus {
    /// Healthy with latency in ms
    Healthy { latency: u64 },
    /// Unhealthy/unreachable
    Unhealthy,
}

impl Mirror {
    /// Convert original URL to mirror URL
    pub fn mirror_url(&self, original: &str) -> Option<String> {
        for (prefix, replacement) in &self.mappings {
            if original.starts_with(prefix) {
                let path = original.strip_prefix(prefix)?;
                return Some(format!("{}{}", self.base_url, replacement.to_string() + path));
            }
        }
        None
    }

    /// Check if this mirror can handle a URL
    pub fn can_handle(&self, url: &str) -> bool {
        self.mappings.iter().any(|(prefix, _)| url.starts_with(prefix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror_url() {
        let mirror = Mirror {
            name: "BMCLAPI".to_string(),
            base_url: "https://bmclapi2.bangbang93.com".to_string(),
            mappings: vec![
                ("https://piston-meta.mojang.com".to_string(), "".to_string()),
            ],
        };

        let result = mirror.mirror_url("https://piston-meta.mojang.com/mc/game/version_manifest.json");
        assert_eq!(
            result,
            Some("https://bmclapi2.bangbang93.com/mc/game/version_manifest.json".to_string())
        );
    }

    #[test]
    fn test_can_handle() {
        let mirror = Mirror {
            name: "BMCLAPI".to_string(),
            base_url: "https://bmclapi2.bangbang93.com".to_string(),
            mappings: vec![
                ("https://piston-meta.mojang.com".to_string(), "".to_string()),
            ],
        };

        assert!(mirror.can_handle("https://piston-meta.mojang.com/test"));
        assert!(!mirror.can_handle("https://example.com/test"));
    }
}
