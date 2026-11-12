use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use bevy::prelude::*;

#[derive(Resource, Default)]
#[allow(dead_code)] // Store contents are read by 08-02/08-03 pin systems.
pub struct PinStore(pub HashSet<PathBuf>);

#[allow(dead_code)] // Wired by 08-02 after Pin/Unpin mutates the store.
pub fn save_pins(pins: &HashSet<PathBuf>) {
    let Some(path) = pins_path() else {
        return;
    };

    if let Err(err) = save_to(&path, pins) {
        warn!("could not save pins: {err}");
    }
}

#[allow(dead_code)] // Shared config path is consumed by 08-02/08-03 pin workflows.
pub fn pins_path() -> Option<PathBuf> {
    crate::config::config_file("pins.json")
}

pub fn load_pins() -> HashSet<PathBuf> {
    pins_path().map(|path| load_from(&path)).unwrap_or_default()
}

fn load_from(path: &Path) -> HashSet<PathBuf> {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return HashSet::new();
    };

    parse_json_array(&contents)
        .unwrap_or_default()
        .into_iter()
        .map(PathBuf::from)
        .collect()
}

fn save_to(path: &Path, pins: &HashSet<PathBuf>) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut items: Vec<String> = pins
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    items.sort();

    let bytes = to_json_array(&items);
    let tmp = path.with_extension("json.tmp");

    std::fs::write(&tmp, bytes)?;
    std::fs::rename(tmp, path)
}

fn to_json_array(items: &[String]) -> String {
    let mut out = String::from("[");

    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&json_escape(item));
        out.push('"');
    }

    out.push(']');
    out
}

fn json_escape(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

fn parse_json_array(input: &str) -> Option<Vec<String>> {
    let mut chars = input.chars().peekable();

    skip_ws(&mut chars);
    if chars.next()? != '[' {
        return None;
    }

    let mut items = Vec::new();

    loop {
        skip_ws(&mut chars);

        match chars.peek() {
            Some(']') => {
                chars.next();
                break;
            }
            Some('"') => {}
            _ => return None,
        }

        items.push(parse_json_string(&mut chars)?);
        skip_ws(&mut chars);

        match chars.peek() {
            Some(',') => {
                chars.next();
                skip_ws(&mut chars);
                if matches!(chars.peek(), Some(']')) {
                    return None;
                }
            }
            Some(']') => {
                chars.next();
                break;
            }
            _ => return None,
        }
    }

    skip_ws(&mut chars);
    if chars.peek().is_none() {
        Some(items)
    } else {
        None
    }
}

fn parse_json_string(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<String> {
    if chars.next()? != '"' {
        return None;
    }

    let mut out = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => return Some(out),
            '\\' => match chars.next() {
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                _ => return None,
            },
            _ => out.push(ch),
        }
    }

    None
}

fn skip_ws(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while matches!(chars.peek(), Some(' ' | '\n' | '\r' | '\t')) {
        chars.next();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_nanos();

        std::env::temp_dir().join(format!(
            "borjes-pins-{name}-{}-{unique}.json",
            std::process::id()
        ))
    }

    #[test]
    fn json_array_round_trips_paths_with_quotes_and_backslashes() {
        let items = vec![
            "/tmp/plain".to_string(),
            "/tmp/with\"quote".to_string(),
            "/tmp/with\\backslash".to_string(),
        ];

        let json = to_json_array(&items);

        assert_eq!(parse_json_array(&json), Some(items));
    }

    #[test]
    fn json_array_empty_slice_is_empty_array() {
        assert_eq!(to_json_array(&[]), "[]");
        assert_eq!(parse_json_array("[]"), Some(Vec::new()));
        assert_eq!(parse_json_array(" [  ] "), Some(Vec::new()));
    }

    #[test]
    fn malformed_json_array_returns_none() {
        for input in [
            "",
            "garbage",
            "\"/tmp\"",
            "[\"/tmp\"",
            "\"/tmp\"]",
            "[/tmp]",
            "[\"/tmp\",]",
            "[\"/tmp\" \"other\"]",
            "[\"bad\\n\"]",
            "[\"bad\\\"]",
        ] {
            assert_eq!(parse_json_array(input), None, "{input:?}");
        }
    }

    #[test]
    fn save_and_load_round_trips_pin_set() {
        let path = unique_path("round-trip");
        let mut pins = HashSet::new();
        pins.insert(PathBuf::from("/tmp/alpha"));
        pins.insert(PathBuf::from("/tmp/with\"quote"));
        pins.insert(PathBuf::from("/tmp/with\\backslash"));

        save_to(&path, &pins).expect("failed to save pins");

        let bytes = std::fs::read_to_string(&path).expect("failed to read saved pins");
        assert!(bytes.starts_with('['));
        assert!(bytes.ends_with(']'));
        assert_eq!(load_from(&path), pins);

        std::fs::remove_file(path).expect("failed to clean pins file");
    }

    #[test]
    fn missing_file_loads_empty_set() {
        let path = unique_path("missing");

        assert!(!path.exists());
        assert!(load_from(&path).is_empty());
    }

    #[test]
    fn corrupt_file_loads_empty_set() {
        let path = unique_path("corrupt");

        std::fs::write(&path, "not a json array").expect("failed to write corrupt pins file");

        assert!(load_from(&path).is_empty());

        std::fs::remove_file(path).expect("failed to clean pins file");
    }
}
