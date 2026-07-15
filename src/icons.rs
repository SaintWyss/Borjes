use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::scan;

// Every icon id we ship in assets/icons/<id>.png.
pub const ICON_IDS: &[&str] = &[
    "folder",
    "file",
    "file-code",
    "file-text",
    "image",
    "music",
    "video",
    "archive",
    "terminal",
    "shield",
    "book-open",
    "graduation-cap",
    "briefcase",
    "inbox",
    "settings",
    "database",
    "gamepad-2",
    "hard-drive",
    "package",
    "cpu",
    "network",
    "globe",
    "wrench",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisualKind {
    Sphere,
    Png,
    Emoji,
    Model,
}

fn default_scale() -> f32 {
    1.0
}

fn default_halo() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VisualSpec {
    pub kind: VisualKind,
    #[serde(default)]
    pub asset: Option<String>,
    #[serde(default)]
    pub emoji: Option<String>,
    #[serde(default = "default_scale")]
    pub scale: f32,
    #[serde(default = "default_halo")]
    pub halo: bool,
}

impl VisualSpec {
    pub fn sphere() -> Self {
        Self {
            kind: VisualKind::Sphere,
            asset: None,
            emoji: None,
            scale: 1.0,
            halo: false,
        }
    }

    pub fn builtin(id: &str) -> Self {
        Self {
            kind: VisualKind::Png,
            asset: Some(format!("icons/{id}.png")),
            emoji: None,
            scale: 0.95,
            halo: true,
        }
    }

    pub fn png(asset: impl Into<String>, scale: f32, halo: bool) -> Self {
        Self {
            kind: VisualKind::Png,
            asset: Some(asset.into()),
            emoji: None,
            scale,
            halo,
        }
    }

    pub fn emoji(emoji: impl Into<String>, scale: f32, halo: bool) -> Self {
        Self {
            kind: VisualKind::Emoji,
            asset: None,
            emoji: Some(emoji.into()),
            scale,
            halo,
        }
    }

    pub fn model(asset: impl Into<String>, scale: f32, halo: bool) -> Self {
        Self {
            kind: VisualKind::Model,
            asset: Some(asset.into()),
            emoji: None,
            scale,
            halo,
        }
    }

    pub fn sane_scale(&self) -> f32 {
        if self.scale.is_finite() {
            self.scale.clamp(0.2, 4.0)
        } else {
            1.0
        }
    }

    pub fn is_structurally_valid(&self) -> bool {
        match self.kind {
            VisualKind::Sphere => true,
            VisualKind::Png | VisualKind::Model => self
                .asset
                .as_deref()
                .is_some_and(|asset| !asset.trim().is_empty()),
            VisualKind::Emoji => self
                .emoji
                .as_deref()
                .is_some_and(|emoji| !emoji.trim().is_empty()),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct IconConfigFile {
    #[serde(default)]
    pub folders: HashMap<String, VisualSpec>,
}

// User overrides: folder NAME or absolute PATH -> visual spec.
#[derive(Resource, Clone, Debug, Default)]
pub struct IconConfig {
    pub folders: HashMap<String, VisualSpec>,
}

// Preloaded built-in icon textures by id. Dynamic PNG/model paths are loaded on demand.
#[derive(Resource, Default)]
#[allow(dead_code)] // Built-in preload is kept for the icon picker/cache path.
pub struct IconAssets(pub HashMap<String, Handle<Image>>);

pub fn load_icon_assets(asset_server: Res<AssetServer>, mut commands: Commands) {
    let mut map = HashMap::new();
    for id in ICON_IDS {
        map.insert(
            (*id).to_string(),
            asset_server.load(format!("icons/{id}.png")),
        );
    }
    commands.insert_resource(IconAssets(map));
}

pub fn config_path() -> Option<PathBuf> {
    crate::config::file_path("icons.json")
}

pub fn user_assets_dir}

pub fn user_assets_dir() -> Option<PathBuf> {
    config_path().and_then(|path| path.parent().map(|parent| parent.join("assets")))
}

pub fn load_icon_config() -> IconConfig {
    let Some(path) = config_path() else {
        return IconConfig::default();
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return IconConfig::default();
    };
    parse_icon_config(&text)
}

pub fn save_icon_config(cfg: &IconConfig) {
    let Some(path) = config_path() else {
        warn!("could not save icon config: HOME/XDG_CONFIG_HOME not set");
        return;
    };
    if let Some(parent) = path.parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            warn!("could not create icon config dir: {err}");
            return;
        }
    }

    let file = IconConfigFile {
        folders: cfg.folders.clone(),
    };
    let Ok(text) = serde_json::to_string_pretty(&file) else {
        warn!("could not serialize icon config");
        return;
    };
    if let Err(err) = std::fs::write(path, format!("{text}\n")) {
        warn!("could not save icon config: {err}");
    }
}

pub fn set_override(cfg: &mut IconConfig, path: &Path, visual: VisualSpec) {
    cfg.folders
        .insert(path.to_string_lossy().to_string(), visual);
}

pub fn clear_override(cfg: &mut IconConfig, path: &Path, name: &str) {
    cfg.folders.remove(&path.to_string_lossy().to_string());
    cfg.folders.remove(name);
}

// Resolve the visual for a folder: override > name hint > content scan > folder.
pub fn visual_for(path: &Path, name: &str, cfg: &IconConfig) -> VisualSpec {
    if let Some(spec) = cfg.folders.get(&path.to_string_lossy().to_string()) {
        if spec.is_structurally_valid() {
            return spec.clone();
        }
        warn!("invalid icon override for {}", path.display());
    }
    if let Some(spec) = cfg.folders.get(name) {
        if spec.is_structurally_valid() {
            return spec.clone();
        }
        warn!("invalid icon override for {name}");
    }

    VisualSpec::builtin(default_icon_id(path, name))
}

pub fn resolve_asset_path(asset: &str) -> Option<String> {
    let expanded = expand_tilde(asset.trim());
    let candidate = PathBuf::from(&expanded);
    if candidate.is_absolute() {
        return candidate
            .exists()
            .then(|| candidate.display().to_string())
            .or_else(|| {
                warn!(
                    "custom visual asset does not exist: {}",
                    candidate.display()
                );
                None
            });
    }

    let repo_asset = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(&expanded);
    if repo_asset.exists() {
        return Some(expanded);
    }

    if let Some(user_asset) = user_assets_dir().map(|dir| dir.join(&expanded)) {
        if user_asset.exists() {
            return Some(user_asset.display().to_string());
        }
    }

    warn!("custom visual asset does not exist: {asset}");
    None
}

pub fn ensure_emoji_png(emoji: &str) -> Option<String> {
    let emoji = emoji.trim();
    if emoji.is_empty() {
        return None;
    }

    let dir = user_assets_dir()?.join("emoji");
    if let Err(err) = std::fs::create_dir_all(&dir) {
        warn!("could not create emoji asset dir: {err}");
        return None;
    }

    let file_name = format!("{}.png", emoji_slug(emoji));
    let path = dir.join(file_name);
    if path.exists() {
        return Some(path.display().to_string());
    }

    let status = Command::new("magick")
        .args([
            "-background",
            "none",
            "-font",
            "Noto Color Emoji",
            "-pointsize",
            "96",
            &format!("label:{emoji}"),
        ])
        .arg(&path)
        .status();

    match status {
        Ok(status) if status.success() && path.exists() => Some(path.display().to_string()),
        Ok(status) => {
            warn!("emoji rasterization failed with status {status}");
            None
        }
        Err(err) => {
            warn!("could not run magick for emoji rasterization: {err}");
            None
        }
    }
}

pub fn default_icon_id(path: &Path, name: &str) -> &'static str {
    let lname = name.to_lowercase();
    let hint = |needle: &str| lname.contains(needle);
    if hint("cyber") {
        "shield"
    } else if hint("knowledge") {
        "book-open"
    } else if hint("career") {
        "briefcase"
    } else if hint("vm") {
        "hard-drive"
    } else if hint("system") {
        "settings"
    } else if hint("game") {
        "gamepad-2"
    } else if hint("inbox") {
        "inbox"
    } else if hint("ops") {
        "wrench"
    } else if hint("archive") {
        "archive"
    } else if hint("workspace") {
        "terminal"
    } else if hint("media") {
        "image"
    } else {
        content_icon(path)
    }
}

fn content_icon(path: &Path) -> &'static str {
    let mut code = 0u32;
    let mut image = 0u32;
    let mut audio = 0u32;
    let mut video = 0u32;
    let mut doc = 0u32;
    let mut archive = 0u32;

    for entry in scan::scan_files(path) {
        let ext = entry
            .name
            .rsplit_once('.')
            .map(|(_, e)| e.to_lowercase())
            .unwrap_or_default();
        match ext.as_str() {
            "rs" | "py" | "js" | "ts" | "tsx" | "go" | "c" | "cpp" | "h" | "java" | "sh" | "rb"
            | "php" => code += 1,
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" => image += 1,
            "mp3" | "wav" | "flac" | "ogg" | "m4a" => audio += 1,
            "mp4" | "mkv" | "avi" | "mov" | "webm" => video += 1,
            "pdf" | "md" | "txt" | "doc" | "docx" | "odt" => doc += 1,
            "zip" | "tar" | "gz" | "7z" | "rar" | "xz" => archive += 1,
            _ => {}
        }
    }

    let cats = [
        (code, "file-code"),
        (image, "image"),
        (audio, "music"),
        (video, "video"),
        (doc, "file-text"),
        (archive, "archive"),
    ];
    cats.iter()
        .filter(|(n, _)| *n > 0)
        .max_by_key(|(n, _)| *n)
        .map(|(_, id)| *id)
        .unwrap_or("folder")
}

fn parse_icon_config(text: &str) -> IconConfig {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        warn!("icons.json is not valid json");
        return IconConfig::default();
    };

    if let Some(folders) = value.get("folders").and_then(Value::as_object) {
        let mut parsed = HashMap::new();
        for (key, value) in folders {
            if let Some(spec) = parse_visual_value(value) {
                parsed.insert(key.clone(), spec);
            }
        }
        return IconConfig { folders: parsed };
    }

    // Backward compatibility: {"FolderName":"shield"}.
    let Some(object) = value.as_object() else {
        return IconConfig::default();
    };
    let mut parsed = HashMap::new();
    for (key, value) in object {
        if let Some(id) = value.as_str() {
            parsed.insert(key.clone(), VisualSpec::builtin(id));
        }
    }
    IconConfig { folders: parsed }
}

fn parse_visual_value(value: &Value) -> Option<VisualSpec> {
    if let Some(id) = value.as_str() {
        return Some(VisualSpec::builtin(id));
    }
    match serde_json::from_value::<VisualSpec>(value.clone()) {
        Ok(spec) if spec.is_structurally_valid() => Some(spec),
        Ok(_) => None,
        Err(err) => {
            warn!("could not parse visual spec: {err}");
            None
        }
    }
}

fn expand_tilde(path: &str) -> String {
    if path == "~" {
        return std::env::var("HOME").unwrap_or_else(|_| path.to_string());
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return Path::new(&home).join(rest).display().to_string();
        }
    }
    path.to_string()
}

fn emoji_slug(emoji: &str) -> String {
    emoji
        .chars()
        .map(|ch| format!("{:x}", ch as u32))
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_path_wins_over_name_hint() {
        let mut cfg = IconConfig::default();
        cfg.folders.insert(
            "/x/30_Cybersecurity".to_string(),
            VisualSpec::builtin("globe"),
        );

        assert_eq!(
            visual_for(Path::new("/x/30_Cybersecurity"), "30_Cybersecurity", &cfg),
            VisualSpec::builtin("globe")
        );
    }

    #[test]
    fn name_hint_resolves_to_builtin_png() {
        let cfg = IconConfig::default();
        assert_eq!(
            visual_for(Path::new("/x/30_Cybersecurity"), "30_Cybersecurity", &cfg),
            VisualSpec::builtin("shield")
        );
    }

    #[test]
    fn empty_or_missing_dir_is_folder_png() {
        let cfg = IconConfig::default();
        assert_eq!(
            visual_for(Path::new("/no/such/dir-xyz"), "plain", &cfg),
            VisualSpec::builtin("folder")
        );
    }

    #[test]
    fn parses_new_format() {
        let cfg = parse_icon_config(
            r#"{
                "folders": {
                    "/x/A": {"kind":"png","asset":"icons/shield.png","scale":1.2},
                    "Books": {"kind":"emoji","emoji":"📚"},
                    "Robot": {"kind":"model","asset":"custom/robot.glb","halo":false}
                }
            }"#,
        );

        assert_eq!(
            cfg.folders.get("/x/A"),
            Some(&VisualSpec::png("icons/shield.png", 1.2, true))
        );
        assert_eq!(
            cfg.folders.get("Books"),
            Some(&VisualSpec::emoji("📚", 1.0, true))
        );
        assert_eq!(
            cfg.folders.get("Robot"),
            Some(&VisualSpec::model("custom/robot.glb", 1.0, false))
        );
    }

    #[test]
    fn parses_legacy_flat_format() {
        let cfg = parse_icon_config(r#"{"a":"folder","b":"shield"}"#);
        assert_eq!(cfg.folders.get("a"), Some(&VisualSpec::builtin("folder")));
        assert_eq!(cfg.folders.get("b"), Some(&VisualSpec::builtin("shield")));
    }

    #[test]
    fn clear_override_removes_path_and_name() {
        let mut cfg = IconConfig::default();
        cfg.folders
            .insert("/x/A".to_string(), VisualSpec::builtin("globe"));
        cfg.folders
            .insert("A".to_string(), VisualSpec::builtin("shield"));

        clear_override(&mut cfg, Path::new("/x/A"), "A");

        assert!(cfg.folders.is_empty());
    }

    #[test]
    fn invalid_override_falls_back() {
        let mut cfg = IconConfig::default();
        cfg.folders.insert(
            "plain".to_string(),
            VisualSpec {
                kind: VisualKind::Png,
                asset: None,
                emoji: None,
                scale: 1.0,
                halo: true,
            },
        );

        assert_eq!(
            visual_for(Path::new("/no/such/dir-xyz"), "plain", &cfg),
            VisualSpec::builtin("folder")
        );
    }
}
