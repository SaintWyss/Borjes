use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

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

// Name + child-count carried by each rendered node so the egui label pass can
// draw it at the node's projected screen position.
#[derive(Component)]
pub struct NodeLabel {
    pub name: String,
    pub count: usize,
    pub is_hub: bool,
}

pub fn render_level(
    mut commands: Commands,
    mut nav: ResMut<NavState>,
    existing: Query<Entity, With<LevelNode>>,
    mut projections: Query<&mut Projection, With<Camera3d>>,
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
    let child_ids = nav.tree.children(current).to_vec();
    let positions = radial_layout(child_ids.len());

    if let Some(node) = nav.tree.node(current) {
        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(hub_material),
            Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(1.3)),
            LevelNode,
            NodeLabel {
                name: node.name.clone(),
                count: child_ids.len(),
                is_hub: true,
            },
        ));
    }

    // F-future: replace plain despawn/respawn with a per-level entity pool.
    for (child_id, position) in child_ids.into_iter().zip(positions) {
        let (child_name, grandchildren) = nav
            .tree
            .node(child_id)
            .map(|node| (node.name.clone(), nav.tree.children(child_id).len()))
            .unwrap_or_else(|| (format!("folder-{child_id}"), 0));

        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(child_material.clone()),
            Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
            LevelNode,
            LevelChild(child_id),
            NodeLabel {
                name: child_name,
                count: grandchildren,
                is_hub: false,
            },
        ));
    }

    // Fit the orthographic camera to the current ring so nodes never read as
    // tiny far-away dots: viewport height ~ 2.2x the ring radius (+ margin).
    let ring_radius = nav::RADIAL_BASE + nav::RADIAL_PER_CHILD * child_count_radius(&nav, current);
    let viewport_height = (ring_radius * 2.4 + 4.0).max(12.0);
    if let Ok(mut projection) = projections.single_mut() {
        if let Projection::Orthographic(ortho) = &mut *projection {
            ortho.scaling_mode = ScalingMode::FixedVertical { viewport_height };
        }
    }

    nav.dirty = false;
}

fn child_count_radius(nav: &NavState, current: usize) -> f32 {
    nav.tree.children(current).len() as f32
}

// Manual hit-test instead of mesh-picking: Bevy's raycast picking does not fire
// reliably under the orthographic camera. We project each child's world position
// to the viewport and pick the nearest one within a pixel radius of the cursor.
pub fn level_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    children: Query<(&LevelChild, &GlobalTransform)>,
    mut nav: ResMut<NavState>,
) -> Result {
    if !mouse.just_pressed(MouseButton::Left) {
        return Ok(());
    }
    if contexts.ctx_mut()?.is_pointer_over_egui() {
        return Ok(());
    }

    let Ok(window) = windows.single() else {
        return Ok(());
    };
    let Some(cursor) = window.cursor_position() else {
        return Ok(());
    };
    let Ok((camera, cam_tf)) = cameras.single() else {
        return Ok(());
    };

    // PICK_RADIUS_PX: click tolerance around a node centre in screen pixels.
    const PICK_RADIUS_PX: f32 = 40.0;
    let mut best: Option<(usize, f32)> = None;
    for (child, gt) in &children {
        let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) else {
            continue;
        };
        let d = screen.distance(cursor);
        if d < PICK_RADIUS_PX && best.is_none_or(|(_, bd)| d < bd) {
            best = Some((child.0, d));
        }
    }

    if let Some((child_id, _)) = best {
        info!("descend to child_id={child_id}");
        nav.current = child_id;
        nav.dirty = true;
    }

    Ok(())
}

pub fn ascend(keys: Res<ButtonInput<KeyCode>>, mut nav: ResMut<NavState>) {
    if !keys.just_pressed(KeyCode::Backspace) {
        return;
    }

    info!("ascend: backspace pressed");
    if let Some(parent) = nav.tree.parent(nav.current) {
        nav.current = parent;
        nav.dirty = true;
    }
}

// Draw each node's name (+ child count for children) at its projected screen
// position, via an egui foreground painter. Hub gets just its name.
pub fn draw_labels(
    mut contexts: EguiContexts,
    cameras: Query<(&Camera, &GlobalTransform)>,
    labels: Query<(&NodeLabel, &GlobalTransform)>,
) -> Result {
    let Ok((camera, cam_tf)) = cameras.single() else {
        return Ok(());
    };
    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("labels"),
    ));

    for (label, gt) in &labels {
        let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) else {
            continue;
        };
        let text = if label.is_hub || label.count == 0 {
            label.name.clone()
        } else {
            format!("{}  ({})", label.name, label.count)
        };
        let (size, color) = if label.is_hub {
            (15.0, egui::Color32::from_rgb(234, 250, 255))
        } else {
            (12.5, egui::Color32::from_rgb(180, 210, 224))
        };
        painter.text(
            egui::pos2(screen.x, screen.y + 16.0),
            egui::Align2::CENTER_TOP,
            text,
            egui::FontId::proportional(size),
            color,
        );
    }

    Ok(())
}

// Breadcrumb top bar: NEURA · root / … / current, so you always know where you
// are in the drill-down.
#[allow(deprecated)] // egui marks Panel::show deprecated but show(ctx) is the correct call (matches ui.rs)
pub fn draw_breadcrumb(mut contexts: EguiContexts, nav: Res<NavState>) -> Result {
    let mut chain: Vec<String> = Vec::new();
    let mut cursor = Some(nav.current);
    while let Some(id) = cursor {
        if let Some(node) = nav.tree.node(id) {
            chain.push(node.name.clone());
        }
        cursor = nav.tree.parent(id);
    }
    chain.reverse();
    let path = chain.join("  /  ");

    let ctx = contexts.ctx_mut()?;
    egui::TopBottomPanel::top("breadcrumb").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("NEURA")
                    .strong()
                    .color(egui::Color32::from_rgb(90, 220, 235)),
            );
            ui.label(egui::RichText::new("·").weak());
            ui.label(egui::RichText::new(path).color(egui::Color32::from_rgb(207, 233, 245)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("click=entrar · retroceso=subir")
                        .weak()
                        .size(11.0),
                );
            });
        });
    });

    Ok(())
}
