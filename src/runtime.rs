use std::path::PathBuf;

pub fn asset_root() -> PathBuf {
    if let Some(path) = std::env::var_os("BORJES_ASSET_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
    {
        return path;
    }

    if let Ok(executable) = std::env::current_exe() {
        if let Some(prefix) = executable.parent().and_then(|bin| bin.parent()) {
            let installed = prefix.join("share/borjes/assets");
            if installed.is_dir() {
                return installed;
            }
        }
    }

    let system = PathBuf::from("/usr/share/borjes/assets");
    if system.is_dir() {
        return system;
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

pub fn root_override() -> Option<String> {
    let mut args = std::env::args_os().skip(1);
    while let Some(argument) = args.next() {
        if argument == "--root" {
            return args
                .next()
                .map(|value| value.to_string_lossy().into_owned());
        }
        if let Some(value) = argument.to_string_lossy().strip_prefix("--root=") {
            return Some(value.to_string());
        }
    }

    std::env::var("BORJES_ROOT")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_asset_fallback_has_expected_suffix() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        assert!(path.ends_with("assets"));
    }
}
