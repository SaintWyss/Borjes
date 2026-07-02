use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{nav::FolderTree, search::score_normalized, ui::normalize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Lens {
    pub name: String,
    pub keywords: Vec<String>,
    pub color: String,
}

#[derive(Resource, Default)]
pub struct Lenses {
    pub all: Vec<Lens>,
}

pub fn parse_color(hex: &str) -> Option<(u8, u8, u8)> {
    let text = hex.strip_prefix('#').unwrap_or(hex);
    if text.len() != 6 || !text.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }

    let red = u8::from_str_radix(&text[0..2], 16).ok()?;
    let green = u8::from_str_radix(&text[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&text[4..6], 16).ok()?;

    Some((red, green, blue))
}

pub fn load_lenses() -> Vec<Lens> {
    let Some(path) = lenses_path() else {
        return default_lenses();
    };

    match load_from(&path) {
        Ok(lenses) => lenses,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let lenses = default_lenses();
            if let Err(err) = save_to(&path, &lenses) {
                warn!("could not save default lenses: {err}");
            }
            lenses
        }
        Err(err) => {
            warn!("could not load lenses: {err}");
            default_lenses()
        }
    }
}

// Editor de lentes en la UI = pendiente; mientras, lenses.json se edita a mano.
#[allow(dead_code)]
pub fn save_lenses(lenses: &[Lens]) {
    let Some(path) = lenses_path() else {
        return;
    };

    if let Err(err) = save_to(&path, lenses) {
        warn!("could not save lenses: {err}");
    }
}

pub fn lens_matches(tree: &FolderTree, lens: &Lens) -> Vec<usize> {
    let keywords: Vec<String> = lens
        .keywords
        .iter()
        .map(|keyword| normalize(keyword))
        .filter(|keyword| !keyword.is_empty())
        .collect();
    if keywords.is_empty() {
        return Vec::new();
    }

    let raw_matches: HashSet<usize> = (0..tree.len())
        .filter(|id| {
            let Some(name) = tree.normalized(*id) else {
                return false;
            };

            keywords
                .iter()
                .any(|keyword| score_normalized(name, keyword) > 0)
        })
        .collect();

    let mut deduped: Vec<usize> = raw_matches
        .iter()
        .copied()
        .filter(|id| {
            let mut parent = tree.parent(*id);
            while let Some(parent_id) = parent {
                if raw_matches.contains(&parent_id) {
                    return false;
                }
                parent = tree.parent(parent_id);
            }
            true
        })
        .collect();

    deduped.sort_by(|a, b| {
        let a_node = tree.node(*a);
        let b_node = tree.node(*b);

        a_node
            .map(|node| node.depth)
            .cmp(&b_node.map(|node| node.depth))
            .then_with(|| {
                a_node
                    .map(|node| node.name.as_str())
                    .cmp(&b_node.map(|node| node.name.as_str()))
            })
    });
    deduped
}

pub fn find_lens<'a>(lenses: &'a [Lens], prefix: &str) -> Option<&'a Lens> {
    let norm_prefix = normalize(prefix);
    if norm_prefix.is_empty() {
        return None;
    }

    lenses
        .iter()
        .find(|lens| normalize(&lens.name).starts_with(&norm_prefix))
}

fn lenses_path() -> Option<PathBuf> {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        if !config_home.is_empty() {
            return Some(PathBuf::from(config_home).join("datatree-rs/lenses.json"));
        }
    }

    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config/datatree-rs/lenses.json"))
}

fn default_lenses() -> Vec<Lens> {
    vec![
        Lens {
            name: "Seguridad".to_string(),
            keywords: vec![
                "seguridad".to_string(),
                "cyber".to_string(),
                "sec".to_string(),
                "kali".to_string(),
                "pentest".to_string(),
                "hack".to_string(),
            ],
            color: "#ff6b6b".to_string(),
        },
        Lens {
            name: "Recursos".to_string(),
            keywords: vec![
                "recursos".to_string(),
                "adsi".to_string(),
                "administracion".to_string(),
            ],
            color: "#8aff6b".to_string(),
        },
    ]
}

fn load_from(path: &Path) -> std::io::Result<Vec<Lens>> {
    let text = std::fs::read_to_string(path)?;
    serde_json::from_str(&text)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
}

fn save_to(path: &Path, lenses: &[Lens]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let text = serde_json::to_string_pretty(lenses)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
    let tmp = path.with_extension("json.tmp");

    std::fs::write(&tmp, format!("{text}\n"))?;
    std::fs::rename(tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::FolderNode;

    fn unique_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "borjes-lenses-{name}-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default()
        ))
    }

    fn folder_node(id: usize, path: &str, parent_id: Option<usize>, depth: usize) -> FolderNode {
        FolderNode {
            id,
            path: PathBuf::from(path),
            name: Path::new(path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(path)
                .to_string(),
            parent_id,
            root_group: 0,
            depth,
            child_count: 0,
            is_symlink: false,
            pos: [0.0; 3],
        }
    }

    #[test]
    fn parse_color_accepts_hash_and_plain_hex() {
        assert_eq!(parse_color("#ff6b6b"), Some((255, 107, 107)));
        assert_eq!(parse_color("8aff6b"), Some((138, 255, 107)));
    }

    #[test]
    fn parse_color_rejects_malformed_input() {
        assert_eq!(parse_color("#ff6b"), None);
        assert_eq!(parse_color("#ff6b6bx"), None);
        assert_eq!(parse_color("#ff6bzz"), None);
        assert_eq!(parse_color(""), None);
    }

    #[test]
    fn default_lenses_save_and_load_round_trip() {
        let path = unique_path("round-trip");
        let lenses = default_lenses();

        save_to(&path, &lenses).expect("failed to save lenses");
        let loaded = load_from(&path).expect("failed to load lenses");

        assert_eq!(loaded.len(), lenses.len());
        for (loaded, expected) in loaded.iter().zip(lenses.iter()) {
            assert_eq!(loaded.name, expected.name);
            assert_eq!(loaded.keywords, expected.keywords);
            assert_eq!(loaded.color, expected.color);
        }

        std::fs::remove_file(path).expect("failed to clean lenses file");
    }

    #[test]
    fn lens_matches_across_branches_and_drops_descendants() {
        let tree = FolderTree::from_nodes(vec![
            folder_node(0, "/data", None, 0),
            folder_node(1, "/data/seguridad", Some(0), 1),
            folder_node(2, "/data/seguridad/kali", Some(1), 2),
            folder_node(3, "/data/work", Some(0), 1),
            folder_node(4, "/data/work/cyber-notes", Some(3), 2),
            folder_node(5, "/data/other", Some(0), 1),
        ]);
        let lens = Lens {
            name: "Seguridad".to_string(),
            keywords: vec![
                "seguridad".to_string(),
                "kali".to_string(),
                "cyber".to_string(),
            ],
            color: "#ff6b6b".to_string(),
        };

        assert_eq!(lens_matches(&tree, &lens), vec![1, 4]);
    }

    #[test]
    fn find_lens_matches_normalized_prefix() {
        let lenses = default_lenses();

        assert_eq!(
            find_lens(&lenses, "segu").map(|lens| lens.name.as_str()),
            Some("Seguridad")
        );
        assert_eq!(
            find_lens(&lenses, "SEGU").map(|lens| lens.name.as_str()),
            Some("Seguridad")
        );
        assert_eq!(
            find_lens(&lenses, "ség").map(|lens| lens.name.as_str()),
            Some("Seguridad")
        );
        assert!(find_lens(&lenses, "").is_none());
    }
}
