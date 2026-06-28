use bevy::math::Vec2;
use std::path::Path;

use crate::scan::FolderNode;

pub const RADIAL_BASE: f32 = 6.0;
pub const RADIAL_PER_CHILD: f32 = 0.4;

pub struct FolderTree {
    nodes: Vec<FolderNode>,
    children: Vec<Vec<usize>>,
}

impl FolderTree {
    pub fn from_nodes(nodes: Vec<FolderNode>) -> FolderTree {
        let mut children = vec![Vec::new(); nodes.len()];

        for node in &nodes {
            if let Some(parent_id) = node.parent_id {
                if let Some(parent_children) = children.get_mut(parent_id) {
                    parent_children.push(node.id);
                }
            }
        }

        FolderTree { nodes, children }
    }

    pub fn children(&self, id: usize) -> &[usize] {
        self.children.get(id).map_or(&[], Vec::as_slice)
    }

    pub fn parent(&self, id: usize) -> Option<usize> {
        self.node(id).and_then(|node| node.parent_id)
    }

    pub fn node(&self, id: usize) -> Option<&FolderNode> {
        self.nodes.get(id)
    }

    pub fn root(&self) -> Option<usize> {
        self.nodes
            .iter()
            .find(|node| node.parent_id.is_none())
            .map(|node| node.id)
    }

    #[allow(dead_code)]
    pub fn find_by_path(&self, path: &Path) -> Option<usize> {
        self.nodes
            .iter()
            .find(|node| node.path.as_path() == path)
            .map(|node| node.id)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

pub fn radial_layout(n: usize) -> Vec<Vec2> {
    if n == 0 {
        return Vec::new();
    }

    let radius = RADIAL_BASE + RADIAL_PER_CHILD * n as f32;

    (0..n)
        .map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / n as f32;
            Vec2::new(angle.cos() * radius, angle.sin() * radius)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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

    fn sample_nodes() -> Vec<FolderNode> {
        vec![
            folder_node(0, "/data", None, 0),
            folder_node(1, "/data/work", Some(0), 1),
            folder_node(2, "/data/life", Some(0), 1),
            folder_node(3, "/data/work/code", Some(1), 2),
        ]
    }

    #[test]
    fn folder_tree_indexes_children_parent_root_and_path() {
        let tree = FolderTree::from_nodes(sample_nodes());

        assert_eq!(tree.len(), 4);
        assert!(!tree.is_empty());
        assert_eq!(tree.children(0), &[1, 2]);
        assert_eq!(tree.children(1), &[3]);
        assert_eq!(tree.parent(3), Some(1));
        assert_eq!(tree.node(2).map(|node| node.name.as_str()), Some("life"));
        assert_eq!(tree.root(), Some(0));
        assert_eq!(tree.find_by_path(Path::new("/data/work/code")), Some(3));
        assert!(tree.children(99).is_empty());
    }

    #[test]
    fn radial_layout_len_matches_requested_count() {
        for n in [0, 1, 5, 12] {
            assert_eq!(radial_layout(n).len(), n);
        }
    }

    #[test]
    fn radial_layout_single_child_is_on_ring() {
        let positions = radial_layout(1);
        let expected_radius = RADIAL_BASE + RADIAL_PER_CHILD;

        assert_eq!(positions.len(), 1);
        assert!((positions[0].length() - expected_radius).abs() < f32::EPSILON);
        assert_ne!(positions[0], Vec2::ZERO);
    }

    #[test]
    fn radial_layout_positions_share_one_circle() {
        let n = 12;
        let positions = radial_layout(n);
        let expected_radius = RADIAL_BASE + RADIAL_PER_CHILD * n as f32;

        assert!(positions
            .iter()
            .all(|position| (position.length() - expected_radius).abs() < 0.000_001));
    }

    #[test]
    fn radial_layout_is_deterministic() {
        assert_eq!(radial_layout(5), radial_layout(5));
    }
}
