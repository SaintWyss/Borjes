use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const HALF_LIFE_SECS: f64 = 1_209_600.0;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FrecencyEntry {
    pub score: f64,
    pub last: u64,
}

#[derive(Resource, Default)]
pub struct Frecency(pub HashMap<PathBuf, FrecencyEntry>);

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

pub fn decayed(entry: &FrecencyEntry, now: u64) -> f64 {
    let elapsed = now.saturating_sub(entry.last) as f64;
    entry.score * 0.5_f64.powf(elapsed / HALF_LIFE_SECS)
}

pub fn bump(map: &mut HashMap<PathBuf, FrecencyEntry>, path: &Path, now: u64) {
    let score = map.get(path).map(|entry| decayed(entry, now)).unwrap_or(0.0);
    map.insert(
        path.to_path_buf(),
        FrecencyEntry {
            score: score + 1.0,
            last: now,
        },
    );
}

pub fn load_frecency() -> HashMap<PathBuf, FrecencyEntry> {
    frecency_path()
        .and_then(|path| load_from(&path).ok())
        .unwrap_or_default()
}

pub fn save_frecency(map: &HashMap<PathBuf, FrecencyEntry>) {
    let Some(path) = frecency_path() else {
        return;
    };

    if let Err(err) = save_to(&path, map) {
        warn!("could not save frecency: {err}");
    }
}

fn frecency_path() -> Option<PathBuf> {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        if !config_home.is_empty() {
            return Some(PathBuf::from(config_home).join("datatree-rs/frecency.json"));
        }
    }

    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config/datatree-rs/frecency.json"))
}

fn load_from(path: &Path) -> std::io::Result<HashMap<PathBuf, FrecencyEntry>> {
    let text = std::fs::read_to_string(path)?;
    serde_json::from_str(&text)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
}

fn save_to(path: &Path, map: &HashMap<PathBuf, FrecencyEntry>) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let text = serde_json::to_string_pretty(map)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
    let tmp = path.with_extension("json.tmp");

    std::fs::write(&tmp, format!("{text}\n"))?;
    std::fs::rename(tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "borjes-frecency-{name}-{}-{}.json",
            std::process::id(),
            now_secs()
        ))
    }

    #[test]
    fn decay_halves_after_fourteen_days() {
        let entry = FrecencyEntry {
            score: 10.0,
            last: 1_000,
        };

        assert!((decayed(&entry, 1_000 + 1_209_600) - 5.0).abs() < 0.000_001);
    }

    #[test]
    fn bump_accumulates_decayed_score() {
        let mut map = HashMap::new();
        let path = Path::new("/tmp/work");

        bump(&mut map, path, 1_000);
        bump(&mut map, path, 1_000 + 1_209_600);

        let entry = map.get(path).expect("missing bumped entry");
        assert!((entry.score - 1.5).abs() < 0.000_001);
        assert_eq!(entry.last, 1_000 + 1_209_600);
    }

    #[test]
    fn save_and_load_round_trips() {
        let path = unique_path("round-trip");
        let mut map = HashMap::new();
        map.insert(
            PathBuf::from("/tmp/alpha"),
            FrecencyEntry {
                score: 2.5,
                last: 42,
            },
        );
        map.insert(
            PathBuf::from("/tmp/beta"),
            FrecencyEntry {
                score: 8.0,
                last: 84,
            },
        );

        save_to(&path, &map).expect("failed to save frecency");
        let loaded = load_from(&path).expect("failed to load frecency");

        assert_eq!(loaded.len(), map.len());
        for (key, entry) in map {
            let loaded_entry = loaded.get(&key).expect("missing loaded entry");
            assert_eq!(loaded_entry.score, entry.score);
            assert_eq!(loaded_entry.last, entry.last);
        }

        std::fs::remove_file(path).expect("failed to clean frecency file");
    }
}
