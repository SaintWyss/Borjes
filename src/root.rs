use std::path::PathBuf;

use bevy::prelude::*;

#[derive(Resource)]
pub struct RootConfig {
    pub path: String,
    pub error: Option<String>,
}

impl Default for RootConfig {
    fn default() -> Self {
        let configured = crate::runtime::root_override().unwrap_or_else(|| "~/Data".to_string());
        Self {
            path: expand_tilde(&configured).display().to_string(),
            error: None,
        }
    }
}

#[derive(Resource, Default)]
pub struct PendingRescan(pub Option<PathBuf>);

pub fn expand_tilde(input: &str) -> PathBuf {
    let Some(home) = std::env::var_os("HOME") else {
        return PathBuf::from(input);
    };

    if input == "~" {
        return PathBuf::from(home);
    }

    input
        .strip_prefix("~/")
        .map(|rest| PathBuf::from(home).join(rest))
        .unwrap_or_else(|| PathBuf::from(input))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::scan_folder;

    #[test]
    fn expand_tilde_expands_home_prefix() {
        let home = PathBuf::from(std::env::var_os("HOME").expect("HOME not set"));

        assert_eq!(expand_tilde("~/Data"), home.join("Data"));
    }

    #[test]
    fn expand_tilde_leaves_absolute_paths_unchanged() {
        let path = PathBuf::from("/tmp/borjes");

        assert_eq!(expand_tilde(path.to_str().unwrap()), path);
    }

    #[test]
    fn scan_folder_missing_path_returns_empty_without_panic() {
        let missing =
            std::env::temp_dir().join(format!("borjes-missing-{}-{}", std::process::id(), "root"));

        assert!(!missing.exists());
        assert!(scan_folder(&missing).is_empty());
    }
}
