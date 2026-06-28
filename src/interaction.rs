use std::path::{Path, PathBuf};

use bevy::picking::{
    pointer::PointerButton,
    prelude::{Click, Out, Over, Pointer},
};
use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::render::{self, FileAssets, GraphData, NodeRef};
use crate::scan;

#[derive(Resource)]
pub struct Selected(pub Option<usize>);

#[derive(Resource)]
pub struct HighlightMaterial(pub Handle<StandardMaterial>);

#[derive(Component)]
pub struct OriginalMaterial(pub Handle<StandardMaterial>);

#[derive(Component)]
#[allow(dead_code)] // v2 pin visual — rewired into drill-down in F13
pub struct Pinned {
    pub original: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct PinMenu {
    pub open: bool,
    pub path: Option<PathBuf>,
    pub entity: Option<Entity>,
    pub pos: Vec2,
    // True for the first frame after opening. The right-click that opens the
    // menu is seen by egui as a click OUTSIDE the freshly-created Area, so
    // clicked_elsewhere() would dismiss it on frame 1 before the user can
    // click Pin. Skip the dismissal check while this is set.
    pub just_opened: bool,
}

#[derive(Component)]
pub struct FileNode {
    pub parent: Entity,
    pub path: PathBuf,
}

#[derive(Component)]
pub struct Expanded;

pub fn on_node_over(
    trigger: On<Pointer<Over>>,
    highlight: Res<HighlightMaterial>,
    mut nodes: Query<&mut MeshMaterial3d<StandardMaterial>, Or<(With<NodeRef>, With<FileNode>)>>,
) {
    let Ok(mut material) = nodes.get_mut(trigger.entity) else {
        return;
    };

    material.0 = highlight.0.clone();
}

pub fn on_node_out(
    trigger: On<Pointer<Out>>,
    highlight: Res<HighlightMaterial>,
    mut nodes: Query<
        (
            &OriginalMaterial,
            &mut MeshMaterial3d<StandardMaterial>,
            Option<&crate::ui::SearchMatch>,
        ),
        Or<(With<NodeRef>, With<FileNode>)>,
    >,
) {
    let Ok((original, mut material, search_match)) = nodes.get_mut(trigger.entity) else {
        return;
    };

    material.0 = if search_match.is_some() {
        highlight.0.clone()
    } else {
        original.0.clone()
    };
}

#[allow(dead_code)]
pub fn on_node_click(
    trigger: On<Pointer<Click>>,
    mut contexts: EguiContexts,
    mut selected: ResMut<Selected>,
    mut pin_menu: ResMut<PinMenu>,
    graph: Res<GraphData>,
    nodes: Query<&NodeRef>,
    windows: Query<&Window>,
    folder_transforms: Query<&Transform>,
    files: Query<(Entity, &FileNode)>,
    expanded: Query<(), With<Expanded>>,
    file_assets: Res<FileAssets>,
    mut commands: Commands,
) -> Result {
    if contexts.ctx_mut()?.is_pointer_over_egui() {
        return Ok(());
    }

    let entity = trigger.entity;

    if trigger.button == PointerButton::Secondary {
        if let Ok(node_ref) = nodes.get(entity) {
            if let Some(node) = graph.0.get(node_ref.0) {
                let pos = windows
                    .iter()
                    .next()
                    .and_then(Window::cursor_position)
                    .unwrap_or_default();
                info!("pin menu: {}", node.path.display());
                pin_menu.open = true;
                pin_menu.path = Some(node.path.clone());
                pin_menu.entity = Some(entity);
                pin_menu.pos = pos;
                pin_menu.just_opened = true;
            }
        }

        return Ok(());
    }

    if trigger.button != PointerButton::Primary {
        return Ok(());
    }

    if let Ok(node_ref) = nodes.get(entity) {
        let Some(node) = graph.0.get(node_ref.0) else {
            return Ok(());
        };

        selected.0 = Some(node_ref.0);
        info!("selected: {}", node.path.display());

        if trigger.count >= 2 {
            if expanded.get(entity).is_err() {
                if let Ok(transform) = folder_transforms.get(entity) {
                    spawn_file_burst(
                        entity,
                        transform.translation,
                        &node.path,
                        &file_assets,
                        &mut commands,
                    );
                }
            }

            let path = node.path.clone();
            std::thread::spawn(move || {
                let _ = open::that(path);
            });
        } else if expanded.get(entity).is_ok() {
            collapse_files(entity, &files, &mut commands);
        } else if let Ok(transform) = folder_transforms.get(entity) {
            spawn_file_burst(
                entity,
                transform.translation,
                &node.path,
                &file_assets,
                &mut commands,
            );
        }

        return Ok(());
    }

    if let Ok((_, file)) = files.get(entity) {
        if trigger.count >= 2 {
            let path = file.path.clone();
            std::thread::spawn(move || {
                let _ = open::that(path);
            });
        }

        return Ok(());
    }

    Ok(())
}

pub fn apply_pin_visual(
    commands: &mut Commands,
    entity: Entity,
    current_original: &Handle<StandardMaterial>,
    gold: &Handle<StandardMaterial>,
) {
    commands.entity(entity).insert((
        MeshMaterial3d(gold.clone()),
        OriginalMaterial(gold.clone()),
        Pinned {
            original: current_original.clone(),
        },
    ));
}

#[allow(dead_code)] // v2 pin visual — rewired in F13
pub fn clear_pin_visual(commands: &mut Commands, entity: Entity, pinned: &Pinned) {
    commands.entity(entity).insert((
        MeshMaterial3d(pinned.original.clone()),
        OriginalMaterial(pinned.original.clone()),
    ));
    commands.entity(entity).remove::<Pinned>();
}

// Re-mark freshly spawned folder nodes gold if their path is in the loaded
// PinStore. `Without<Pinned>` filters out already-gold nodes so the system
// stays cheap. Covers BOTH startup load (PIN-02) and Reescanear (PIN-03):
// after either, pinned folders exist again as plain NodeRef entities.
#[allow(dead_code)]
pub fn apply_pins(
    mut commands: Commands,
    pin_store: Res<crate::pins::PinStore>,
    pin_assets: Res<render::PinAssets>,
    graph: Res<GraphData>,
    nodes: Query<(Entity, &NodeRef, &OriginalMaterial), Without<Pinned>>,
) {
    for (entity, node_ref, original) in &nodes {
        let Some(node) = graph.0.get(node_ref.0) else {
            continue;
        };
        if pin_store.0.contains(&node.path) {
            apply_pin_visual(&mut commands, entity, &original.0, &pin_assets.material);
        }
    }
}

fn spawn_file_burst(
    folder_entity: Entity,
    parent_pos: Vec3,
    dir: &Path,
    file_assets: &FileAssets,
    commands: &mut Commands,
) {
    let entries = scan::scan_files(dir);
    let n = entries.len();

    for (i, entry) in entries.into_iter().enumerate() {
        let offset = file_burst_offset(i, n);
        commands.spawn((
            Mesh3d(file_assets.mesh.clone()),
            MeshMaterial3d(file_assets.material.clone()),
            OriginalMaterial(file_assets.material.clone()),
            Transform::from_translation(parent_pos + offset)
                .with_scale(Vec3::splat(render::FILE_NODE_SCALE)),
            Name::new(entry.name),
            FileNode {
                parent: folder_entity,
                path: entry.path,
            },
        ));
    }

    commands.entity(folder_entity).insert(Expanded);
}

fn collapse_files(
    folder_entity: Entity,
    files: &Query<(Entity, &FileNode)>,
    commands: &mut Commands,
) {
    for (file_entity, file) in files {
        if file.parent == folder_entity {
            commands.entity(file_entity).despawn();
        }
    }

    commands.entity(folder_entity).remove::<Expanded>();
}

fn file_burst_offset(i: usize, n: usize) -> Vec3 {
    let dir = if n == 1 {
        Vec3::Y
    } else {
        render::fibonacci_offset(i, n)
    };
    dir * render::BURST_RADIUS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_file_burst_offsets_up_by_radius() {
        assert_eq!(file_burst_offset(0, 1), Vec3::Y * render::BURST_RADIUS);
    }
}
