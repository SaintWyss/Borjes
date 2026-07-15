use ignore::WalkBuilder;
use std::{
    collections::{HashMap, HashSet},
    fmt,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

const EXCLUDED: &[&str] = &[
    "node_modules",
    ".git",
    ".cache",
    "Steam",
    "target",
    "__pycache__",
    ".Trash",
];

pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
}

pub struct FolderNode {
    pub id: usize,
    pub path: PathBuf,
    pub name: String,
    pub parent_id: Option<usize>,
    pub root_group: usize,
    pub depth: usize,
    pub child_count: usize,
    pub is_symlink: bool,
    pub pos: [f32; 3],
}

impl fmt::Display for FolderNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _phase2_placeholders = (&self.path, self.pos);

        write!(
            f,
            "id={} depth={} root_group={} parent_id={:?} child_count={} is_symlink={} name={}",
            self.id,
            self.depth,
            self.root_group,
            self.parent_id,
            self.child_count,
            self.is_symlink,
            self.name
        )
    }
}

pub fn scan_folder(root: &Path) -> Vec<FolderNode> {
    let walk_root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let walker = WalkBuilder::new(&walk_root)
        .follow_links(false)
        .hidden(true) // saltear dotdirs (.config, .stversions, …) = ruido para navegar
        // git_ignore se deja en default (true): saltea dist/build/vendor que git ignora = más ruido fuera
        .filter_entry(|entry| {
            let name = entry.file_name().to_str().unwrap_or("");
            !EXCLUDED.contains(&name)
        })
        .build();

    let mut nodes: Vec<FolderNode> = Vec::new();
    let mut path_to_id: HashMap<PathBuf, usize> = HashMap::new();
    let mut visited_inodes: HashSet<u64> = HashSet::new();
    let mut visited_symlink_targets: HashSet<PathBuf> = HashSet::new();

    for result in walker {
        let entry = match result {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let is_dir = entry
            .file_type()
            .map(|file_type| file_type.is_dir())
            .unwrap_or(false);
        let is_symlink = entry.path_is_symlink();

        if !is_dir && !is_symlink {
            continue;
        }

        if is_symlink {
            let target_metadata = match std::fs::metadata(entry.path()) {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };

            if !target_metadata.is_dir() {
                continue;
            }

            let canonical_target = match std::fs::canonicalize(entry.path()) {
                Ok(path) => path,
                Err(_) => continue,
            };

            let symlink_inode = match std::fs::symlink_metadata(entry.path()) {
                Ok(metadata) => metadata.ino(),
                Err(_) => continue,
            };

            if !visited_inodes.insert(symlink_inode) {
                continue;
            }

            if !visited_symlink_targets.insert(canonical_target) {
                continue;
            }
        }

        let id = nodes.len();
        let depth = entry.depth();
        let path = entry.path().to_path_buf();
        let name = entry.file_name().to_str().unwrap_or("").to_string();
        let parent_id = path
            .parent()
            .and_then(|parent| path_to_id.get(parent))
            .copied();

        path_to_id.insert(path.clone(), id);
        nodes.push(FolderNode {
            id,
            path,
            name,
            parent_id,
            root_group: 0,
            depth,
            child_count: 0,
            is_symlink,
            pos: [0.0_f32; 3],
        });
    }

    // Pre-orden: el padre siempre se procesa antes que el hijo, así root_groups
    // ya tiene el grupo del padre cuando llega un nodo de depth > 1.
    let mut root_groups: HashMap<usize, usize> = HashMap::new();
    for node in &mut nodes {
        let root_group = match node.depth {
            0 => 0,
            1 => node.id,
            _ => node
                .parent_id
                .and_then(|parent_id| root_groups.get(&parent_id).copied())
                .unwrap_or(0),
        };

        node.root_group = root_group;
        root_groups.insert(node.id, root_group);
    }

    let mut counts: HashMap<usize, usize> = HashMap::new();
    for node in &nodes {
        if let Some(parent_id) = node.parent_id {
            *counts.entry(parent_id).or_insert(0) += 1;
        }
    }

    for node in &mut nodes {
        node.child_count = *counts.get(&node.id).unwrap_or(&0);
    }

    nodes
}

pub fn scan_files(dir: &Path) -> Vec<FileEntry> {
    let scan_root = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    let Ok(entries) = std::fs::read_dir(scan_root) else {
        return Vec::new();
    };

    let mut files: Vec<FileEntry> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_str()?.to_string();

            if name.starts_with('.') || EXCLUDED.contains(&name.as_str()) {
                return None;
            }

            let file_type = entry.file_type().ok()?;
            if !file_type.is_file() {
                return None;
            }

            Some(FileEntry {
                path: entry.path(),
                name,
            })
        })
        .collect();

    // read_dir order is filesystem-dependent. Sort before applying the visual
    // cap so the same directory always produces the same burst.
    files.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.name.cmp(&right.name))
    });
    files.truncate(200);
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::os::unix::fs::symlink;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_scan_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("borjes-{name}-{}-{unique}", std::process::id()));

        fs::create_dir(&path).expect("failed to create temp scan dir");
        path
    }

    #[test]
    fn scan_files_returns_files_only_one_level() {
        let dir = temp_scan_dir("files-only");
        File::create(dir.join("alpha.txt")).expect("failed to create alpha.txt");
        fs::create_dir(dir.join("nested")).expect("failed to create nested dir");
        File::create(dir.join("nested").join("beta.txt")).expect("failed to create beta.txt");

        let files = scan_files(&dir);
        let names: Vec<&str> = files.iter().map(|file| file.name.as_str()).collect();

        assert_eq!(files.len(), 1);
        assert_eq!(names, ["alpha.txt"]);
        assert!(files[0].path.is_absolute());

        fs::remove_dir_all(dir).expect("failed to clean temp scan dir");
    }

    #[test]
    fn scan_files_excludes_dotfiles_and_excluded_names() {
        let dir = temp_scan_dir("hidden-excluded");
        File::create(dir.join("visible.txt")).expect("failed to create visible.txt");
        File::create(dir.join(".secret")).expect("failed to create dotfile");
        File::create(dir.join("target")).expect("failed to create excluded file");

        let files = scan_files(&dir);
        let names: Vec<&str> = files.iter().map(|file| file.name.as_str()).collect();

        assert_eq!(files.len(), 1);
        assert_eq!(names, ["visible.txt"]);

        fs::remove_dir_all(dir).expect("failed to clean temp scan dir");
    }

    #[test]
    fn scan_files_caps_results_at_200() {
        let dir = temp_scan_dir("cap");
        for index in 0..250 {
            File::create(dir.join(format!("file-{index:03}.txt"))).expect("failed to create file");
        }

        let files = scan_files(&dir);

        assert_eq!(files.len(), 200);

        fs::remove_dir_all(dir).expect("failed to clean temp scan dir");
    }

    #[test]
    fn scan_files_returns_empty_for_missing_dir() {
        let dir = std::env::temp_dir().join(format!(
            "borjes-missing-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock before UNIX_EPOCH")
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);

        assert!(scan_files(&dir).is_empty());
    }

    #[test]
    fn scan_files_is_deterministic_and_sorted() {
        let dir = temp_scan_dir("sorted");
        File::create(dir.join("zeta.txt")).unwrap();
        File::create(dir.join("Alpha.txt")).unwrap();
        File::create(dir.join("beta.txt")).unwrap();

        let names: Vec<String> = scan_files(&dir)
            .into_iter()
            .map(|entry| entry.name)
            .collect();

        assert_eq!(names, ["Alpha.txt", "beta.txt", "zeta.txt"]);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn scan_folder_builds_consistent_tree_from_fixture() {
        let dir = temp_scan_dir("tree");
        fs::create_dir_all(dir.join("alpha/child")).unwrap();
        fs::create_dir_all(dir.join("beta/child/grandchild")).unwrap();
        fs::create_dir_all(dir.join("node_modules/noise")).unwrap();
        fs::create_dir_all(dir.join(".hidden/noise")).unwrap();
        symlink(dir.join("alpha"), dir.join("alpha-link")).unwrap();

        let nodes = scan_folder(&dir);

        assert_eq!(nodes.iter().filter(|node| node.depth == 0).count(), 1);
        let root = nodes.iter().find(|node| node.depth == 0).unwrap();
        assert_eq!(root.parent_id, None);
        assert!(!nodes.iter().any(|node| node.name == "node_modules"));
        assert!(!nodes.iter().any(|node| node.name == ".hidden"));
        assert!(nodes
            .iter()
            .any(|node| node.name == "alpha-link" && node.is_symlink));

        for node in nodes.iter().filter(|node| node.depth > 0) {
            assert!(node.parent_id.is_some(), "{} has no parent", node.name);
        }
        for node in nodes.iter().filter(|node| node.depth == 1) {
            assert_eq!(node.root_group, node.id);
        }

        let total_child_counts: usize = nodes.iter().map(|node| node.child_count).sum();
        let nodes_with_parent = nodes.iter().filter(|node| node.parent_id.is_some()).count();
        assert_eq!(total_child_counts, nodes_with_parent);

        fs::remove_dir_all(dir).unwrap();
    }
}
