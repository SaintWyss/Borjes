use bevy::prelude::*;

use crate::nav;

mod anim;
mod camera;
mod hover;
mod level;
mod overlay;
mod preview;

pub use anim::*;
pub use camera::*;
pub use hover::*;
pub use level::*;
pub use overlay::*;
pub use preview::*;

#[derive(Resource)]
pub struct NavState {
    pub tree: nav::FolderTree,
    pub current: usize,
    pub dirty: bool,
    pub history: Vec<usize>,
}

impl NavState {
    pub fn navigate(&mut self, id: usize) {
        if id == self.current {
            return;
        }

        self.history.push(self.current);
        if self.history.len() > 32 {
            self.history.remove(0);
        }
        self.current = id;
        self.dirty = true;
    }

    pub fn back(&mut self) {
        let Some(previous) = self.history.pop() else {
            return;
        };

        self.current = previous;
        self.dirty = true;
    }
}
// Hover state: which node the cursor is over. Used for the highlight ring,
// scale bump and camera lean; a left-click on it dives (see click_dive).
#[derive(Clone, Copy, PartialEq, Default)]
pub enum HoverTarget {
    #[default]
    None,
    Child(usize),
    Parent,
    Echo,
}

#[derive(Resource, Default)]
pub struct HoverState {
    pub target: HoverTarget,
}

#[derive(Component)]
pub struct LevelNode;

#[derive(Component)]
pub struct LevelFolder(pub usize);

#[derive(Component)]
pub struct LevelChild(pub usize);

// The parent of the focus, drawn faded behind/above. Hovering it ascends.
#[derive(Component)]
pub struct ParentNode;

#[derive(Component)]
pub struct EchoNode;

#[derive(Resource, Default)]
pub struct IconMenu {
    pub open: bool,
    pub node_id: Option<usize>,
    pub name: String,
    pub path: std::path::PathBuf,
    pub pos: Vec2,
    pub emoji: String,
    pub png_asset: String,
    pub model_asset: String,
    pub scale: f32,
    pub halo: bool,
    pub just_opened: bool,
}

// Name + child-count carried by each rendered node so the egui label pass can
// draw it at the node's projected screen position.
#[derive(Component)]
pub struct NodeLabel {
    pub name: String,
    pub count: usize,
    pub kind: NodeKind,
}

#[derive(Clone, Copy, PartialEq)]
pub enum NodeKind {
    Hub,
    Child,
    Parent,
    Echo,
}

pub(crate) const HUB_RADIUS: f32 = 1.4;
pub(crate) const CHILD_RADIUS: f32 = 0.9;
pub(crate) const PARENT_RADIUS: f32 = 0.7;

// Per-node grow-in + hover-bump animation.
#[derive(Component)]
pub struct NodeAnim {
    pub base: f32,
    pub age: f32,
    // Rest position; organic drift wobbles the node around it (children only).
    pub home: Vec3,
    pub phase: f32,
    // Grow-in stagger: la onda arranca en el hub y ondula por el anillo.
    pub delay: f32,
}

pub(crate) const GROW_SECS: f32 = 0.18;
pub(crate) const HOVER_BUMP: f32 = 1.28;
// Organic drift: children sway gently around home for the "alive neuron" feel.
// Kept small so the field stays calm and clickable.
pub(crate) const DRIFT_AMP: f32 = 0.15;

// A hub→child connector that re-aims at the (drifting) child each frame.
#[derive(Component)]
pub struct Connector {
    pub child: usize,
}

pub(crate) fn glow_material(color: Color, intensity: f32) -> StandardMaterial {
    let mut emissive = color.to_linear();
    emissive.red *= intensity;
    emissive.green *= intensity;
    emissive.blue *= intensity;
    StandardMaterial {
        base_color: color,
        emissive,
        unlit: true,
        ..default()
    }
}

pub(crate) fn textured_material(texture: Handle<Image>) -> StandardMaterial {
    // No halo behind icons anymore, so the icon itself glows: emissive follows
    // the icon shape (emissive_texture) and blooms. Teal tint over white icons.
    StandardMaterial {
        base_color: Color::srgb(0.5, 0.92, 1.0),
        base_color_texture: Some(texture.clone()),
        emissive: LinearRgba::rgb(0.6, 2.4, 2.8),
        emissive_texture: Some(texture),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        unlit: true,
        ..default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::FolderNode;
    use std::path::{Path, PathBuf};

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

    fn nav_state() -> NavState {
        NavState {
            tree: crate::nav::FolderTree::from_nodes(vec![
                folder_node(0, "/data", None, 0),
                folder_node(1, "/data/work", Some(0), 1),
                folder_node(2, "/data/life", Some(0), 1),
                folder_node(3, "/data/work/code", Some(1), 2),
            ]),
            current: 0,
            dirty: false,
            history: Vec::new(),
        }
    }

    #[test]
    fn navigate_pushes_current_and_marks_dirty() {
        let mut nav = nav_state();

        nav.navigate(1);
        nav.navigate(3);

        assert_eq!(nav.current, 3);
        assert_eq!(nav.history, vec![0, 1]);
        assert!(nav.dirty);
    }

    #[test]
    fn navigate_same_id_is_noop() {
        let mut nav = nav_state();
        nav.navigate(1);
        nav.dirty = false;

        nav.navigate(1);

        assert_eq!(nav.current, 1);
        assert_eq!(nav.history, vec![0]);
        assert!(!nav.dirty);
    }

    #[test]
    fn navigate_caps_history_at_32_entries() {
        let mut nav = nav_state();

        for i in 0..40 {
            nav.navigate((i % 3) + 1);
        }

        assert_eq!(nav.history.len(), 32);
        assert_eq!(nav.history.first().copied(), Some(2));
    }

    #[test]
    fn back_pops_without_repush() {
        let mut nav = nav_state();
        nav.navigate(1);
        nav.navigate(3);
        nav.dirty = false;

        nav.back();
        assert_eq!(nav.current, 1);
        assert_eq!(nav.history, vec![0]);
        assert!(nav.dirty);

        nav.dirty = false;
        nav.back();
        assert_eq!(nav.current, 0);
        assert!(nav.history.is_empty());
        assert!(nav.dirty);

        nav.dirty = false;
        nav.back();
        assert_eq!(nav.current, 0);
        assert!(nav.history.is_empty());
        assert!(!nav.dirty);
    }
}
