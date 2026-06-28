use bevy::picking::{
    pointer::PointerButton,
    prelude::{Click, Pointer},
};
use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::nav::{self, radial_layout};

#[derive(Resource)]
pub struct NavState {
    pub tree: nav::FolderTree,
    pub current: usize,
    pub dirty: bool,
}

#[derive(Component)]
pub struct LevelNode;

#[derive(Component)]
pub struct LevelChild(pub usize);

pub fn render_level(
    mut commands: Commands,
    mut nav: ResMut<NavState>,
    existing: Query<Entity, With<LevelNode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !nav.dirty {
        return;
    }

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let sphere = meshes.add(Sphere::new(0.5));
    let hub_color = Color::srgb(0.35, 0.95, 1.0);
    let child_color = Color::srgb(0.2, 0.7, 0.75);
    let hub_material = materials.add(StandardMaterial {
        base_color: hub_color,
        emissive: hub_color.to_linear(),
        unlit: true,
        ..default()
    });
    let child_material = materials.add(StandardMaterial {
        base_color: child_color,
        emissive: child_color.to_linear(),
        unlit: true,
        ..default()
    });

    let current = nav.current;
    if let Some(node) = nav.tree.node(current) {
        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(hub_material),
            Transform::from_translation(Vec3::ZERO),
            LevelNode,
            Name::new(node.name.clone()),
        ));
    }

    let child_ids = nav.tree.children(current).to_vec();
    let positions = radial_layout(child_ids.len());

    // F-future: replace plain despawn/respawn with a per-level entity pool.
    for (child_id, position) in child_ids.into_iter().zip(positions) {
        let child_name = nav
            .tree
            .node(child_id)
            .map(|node| node.name.clone())
            .unwrap_or_else(|| format!("folder-{child_id}"));

        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(child_material.clone()),
            Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
            LevelNode,
            LevelChild(child_id),
            Name::new(child_name),
        ));
    }

    nav.dirty = false;
}

pub fn on_level_click(
    trigger: On<Pointer<Click>>,
    mut contexts: EguiContexts,
    children: Query<&LevelChild>,
    mut nav: ResMut<NavState>,
) -> Result {
    if contexts.ctx_mut()?.is_pointer_over_egui() {
        return Ok(());
    }

    if trigger.button != PointerButton::Primary {
        return Ok(());
    }

    if let Ok(LevelChild(child_id)) = children.get(trigger.entity) {
        nav.current = *child_id;
        nav.dirty = true;
    }

    Ok(())
}

pub fn ascend(keys: Res<ButtonInput<KeyCode>>, mut nav: ResMut<NavState>) {
    if !keys.just_pressed(KeyCode::Backspace) {
        return;
    }

    if let Some(parent) = nav.tree.parent(nav.current) {
        nav.current = parent;
        nav.dirty = true;
    }
}
