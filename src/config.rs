use std::path::{Path, PathBuf};

const APP_DIR: &str = "borjes";
const LEGACY_DIR: &str = "datatree-rs";

pub fn app_dir() -> Option<PathBuf> {
    config_home().map(|root| root.join(APP_DIR))
}

pub fn file_path(name: &str) -> Option<PathBuf> {
    app_dir().map(|dir| dir.join(name))
}

/// Moves v1-v5 data from the former datatree-rs identity without overwriting
/// newer Borjes state. The operation is idempotent and safe to run every start.
pub fn migrate_legacy() {
    let Some(root) = config_home() else {
        return;
    };

    let current = root.join(APP_DIR);
    let legacy = root.join(LEGACY_DIR);
    if !legacy.exists() {
        return;
    }

    if let Err(err) = std::fs::create_dir_all(&current) {
        eprintln!("borjes: could not create config directory: {err}");
        return;
    }

    for name in ["pins.json", "frecency.json", "lenses.json", "icons.json"] {
        if let Err(err) = migrate_file(&legacy.join(name), &current.join(name)) {
            eprintln!("borjes: could not migrate {name}: {err}");
        }
    }

    let old_assets = legacy.join("assets");
    let new_assets = current.join("assets");
    if old_assets.exists() && !new_assets.exists() {
        if let Err(err) = move_or_copy_dir(&old_assets, &new_assets) {
            eprintln!("borjes: could not migrate custom assets: {err}");
        }
    }

    let _ = std::fs::remove_dir(&legacy);
}

fn config_home() -> Option<PathBuf> {
    if let Some(value) = std::env::var_os("XDG_CONFIG_HOME") {
        if !value.is_empty() {
            return Some(PathBuf::from(value));
        }
    }

    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config"))
}

fn migrate_file(old: &Path, new: &Path) -> std::io::Result<()> {
    if new.exists() || !old.exists() {
        return Ok(());
    }

    if let Some(parent) = new.parent() {
        std::fs::create_dir_all(parent)?;
    }

    match std::fs::rename(old, new) {
        Ok(()) => Ok(()),
        Err(_) => {
            std::fs::copy(old, new)?;
            std::fs::remove_file(old)
        }
    }
}

fn move_or_copy_dir(old: &Path, new: &Path) -> std::io::Result<()> {
    match std::fs::rename(old, new) {
        Ok(()) => Ok(()),
        Err(_) => {
            copy_dir_recursive(old, new)?;
            std::fs::remove_dir_all(old)
        }
    }
}

fn copy_dir_recursive(source: &Path, target: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(target)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let destination = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &destination)?;
        } else {
            std::fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("borjes-config-{name}-{}-{stamp}", std::process::id()))
    }

    #[test]
    fn migration_preserves_newer_destination() {
        let root = temp("preserve");
        let old = root.join("old.json");
        let new = root.join("new.json");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&old, "old").unwrap();
        std::fs::write(&new, "new").unwrap();

        migrate_file(&old, &new).unwrap();

        assert_eq!(std::fs::read_to_string(&new).unwrap(), "new");
        assert_eq!(std::fs::read_to_string(&old).unwrap(), "old");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn migration_moves_missing_destination() {
        let root = temp("move");
        let old = root.join("old.json");
        let new = root.join("nested/new.json");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&old, "state").unwrap();

        migrate_file(&old, &new).unwrap();

        assert_eq!(std::fs::read_to_string(&new).unwrap(), "state");
        assert!(!old.exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
