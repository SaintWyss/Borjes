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

    files.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.name.cmp(&b.name))
    });
    // Stable soft cap: huge folders always show the same deterministic slice.
    files.truncate(200);
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn data_path() -> PathBuf {
        PathBuf::from(std::env::var("HOME").expect("HOME not set")).join("Data")
    }

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
        let dir = std::env::temp_dir().join(format!("borjes-missing-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        assert!(scan_files(&dir).is_empty());
    }

    #[test]
    fn test_scan_count() {
        let nodes = scan_folder(&data_path());
        // El roadmap estimó ~604 nodos, pero ese número venía de un preview
        // capeado a profundidad 5. El árbol real de ~/Data a profundidad
        // completa (sin ocultos, sin gitignored, con EXCLUDED) ronda los 4600.
        assert!(
            nodes.len() >= 4300 && nodes.len() <= 4900,
            "Expected current ~/Data scan to be ~4.6k nodes, got {}",
            nodes.len()
        );
    }

    #[test]
    fn test_no_excluded_dirs() {
        let nodes = scan_folder(&data_path());

        for name in EXCLUDED {
            assert!(
                !nodes.iter().any(|node| node.name == *name),
                "Found excluded dir '{name}' in results"
            );
        }
    }

    #[test]
    fn test_symlink_01_now_once() {
        let nodes = scan_folder(&data_path());
        let matching: Vec<&FolderNode> =
            nodes.iter().filter(|node| node.name == "01_Now").collect();

        assert_eq!(
            matching.len(),
            1,
            "01_Now debe aparecer exactamente una vez"
        );

        let actual_01_now_is_symlink = std::fs::symlink_metadata(&matching[0].path)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false);

        if actual_01_now_is_symlink {
            assert!(matching[0].is_symlink, "01_Now debe tener is_symlink=true");
        } else {
            // In this real ~/Data snapshot, 01_Now is a normal directory and
            // contains symlink children instead of being the symlink itself.
            assert!(
                !matching[0].is_symlink,
                "01_Now no debe marcarse como symlink si el FS real no lo es"
            );
            assert!(
                nodes.iter().any(|node| node.is_symlink),
                "Debe existir al menos un symlink real a directorio en ~/Data"
            );
        }
    }

    #[test]
    fn test_root_node_depth_zero() {
        let nodes = scan_folder(&data_path());
        let roots: Vec<&FolderNode> = nodes.iter().filter(|node| node.depth == 0).collect();

        assert_eq!(
            roots.len(),
            1,
            "Debe haber exactamente un nodo raíz depth=0"
        );
        assert_eq!(roots[0].parent_id, None);
    }

    #[test]
    fn test_parent_ids_non_root() {
        let nodes = scan_folder(&data_path());

        nodes.iter().filter(|node| node.depth > 0).for_each(|node| {
            assert!(
                node.parent_id.is_some(),
                "Nodo {} depth={} no tiene parent_id",
                node.name,
                node.depth
            );
        });
    }

    #[test]
    fn test_root_groups_depth1() {
        let nodes = scan_folder(&data_path());

        nodes
            .iter()
            .filter(|node| node.depth == 1)
            .for_each(|node| {
                assert_eq!(
                    node.root_group, node.id,
                    "depth-1 node {} tiene root_group={} pero esperado {}",
                    node.name, node.root_group, node.id
                );
            });
    }

    #[test]
    fn test_child_counts_consistent() {
        let nodes = scan_folder(&data_path());
        let total_child_counts: usize = nodes.iter().map(|node| node.child_count).sum();
        let nodes_with_parent = nodes.iter().filter(|node| node.parent_id.is_some()).count();

        assert_eq!(
            total_child_counts, nodes_with_parent,
            "child_count totales no coinciden con cantidad de nodos con padre"
        );
    }
}
