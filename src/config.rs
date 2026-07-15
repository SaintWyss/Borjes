use std::path::{Path, PathBuf};

const APP_DIR: &str = "borjes";
const LEGACY_APP_DIR: &str = "datatree-rs";

pub fn config_dir() -> Option<PathBuf> {
    base_config_dir().map(|base| base.join(APP_DIR))
}

pub fn config_file(name: &str) -> Option<PathBuf> {
    let current = config_dir()?.join(name);
    let legacy = base_config_dir()?.join(LEGACY_APP_DIR).join(name);
    if !current.exists() && legacy.exists() {
        if let Err(err) = migrate_file(&legacy, &current) {
            eprintln!(
                "borjes: could not migrate {} to {}: {err}",
                legacy.display(),
                current.display()
            );
        }
    }
    Some(current)
}

pub fn assets_dir() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("assets"))
}

pub fn asset_root() -> PathBuf {
    if let Some(path) = std::env::var_os("BORJES_ASSET_DIR").filter(|value| !value.is_empty()) {
        return PathBuf::from(path);
    }

    let user_install = std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".local/share/borjes"));
    if let Some(path) = user_install.filter(|path| path.is_dir()) {
        return path;
    }

    let system_install = PathBuf::from("/usr/share/borjes");
    if system_install.is_dir() {
        return system_install;
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn base_config_dir() -> Option<PathBuf> {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        if !config_home.is_empty() {
            return Some(PathBuf::from(config_home));
        }
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config"))
}

fn migrate_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    if destination.exists() || !source.exists() {
        return Ok(());
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match std::fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(_) => {
            std::fs::copy(source, destination)?;
            std::fs::remove_file(source)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "borjes-config-{name}-{}-{unique}",
            std::process::id()
        ))
    }
    #[test]
    fn migrates_legacy_file_without_losing_contents() {
        let root = temp_dir("migrate");
        let source = root.join("legacy/lenses.json");
        let destination = root.join("borjes/lenses.json");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, "important").unwrap();
        migrate_file(&source, &destination).unwrap();
        assert_eq!(std::fs::read_to_string(&destination).unwrap(), "important");
        assert!(!source.exists());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn migration_never_overwrites_current_data() {
        let root = temp_dir("preserve");
        let source = root.join("legacy/pins.json");
        let destination = root.join("borjes/pins.json");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(&source, "legacy").unwrap();
        std::fs::write(&destination, "current").unwrap();
        migrate_file(&source, &destination).unwrap();
        assert_eq!(std::fs::read_to_string(destination).unwrap(), "current");
        assert_eq!(std::fs::read_to_string(source).unwrap(), "legacy");
        std::fs::remove_dir_all(root).unwrap();
    }
}
