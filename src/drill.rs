use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::nav::{self, radial_layout};

#[derive(Resource)]
pub struct NavState {
    pub tree: nav::FolderTree,
    pub current: usize,
    pub dirty: bool,
}

// Hover-dwell state: hovering a node fills a 1s loading ring; only when it
// completes do we dive. Leaving the node resets the timer.
#[derive(Clone, Copy, PartialEq)]
pub enum HoverTarget {
    None,
    Child(usize),
    Parent,
}

#[derive(Resource)]
pub struct HoverState {
    pub target: HoverTarget,
    pub elapsed: f32,
}

impl Default for HoverState {
    fn default() -> Self {
        HoverState {
            target: HoverTarget::None,
            elapsed: 0.0,
        }
    }
}

// Seconds of continuous hover before diving into a node.
pub const DWELL_SECS: f32 = 0.5;

#[derive(Component)]
pub struct LevelNode;

#[derive(Component)]
pub struct LevelChild(pub usize);

// The parent of the focus, drawn faded behind/above. Hovering it ascends.
#[derive(Component)]
pub struct ParentNode;

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
}

const HUB_RADIUS: f32 = 1.4;
const CHILD_RADIUS: f32 = 0.9;
const PARENT_RADIUS: f32 = 0.7;

// Per-node grow-in + hover-bump animation.
#[derive(Component)]
pub struct NodeAnim {
    pub base: f32,
    pub age: f32,
}

const GROW_SECS: f32 = 0.28;
const HOVER_BUMP: f32 = 1.28;

fn glow_material(color: Color, intensity: f32) -> StandardMaterial {
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

pub fn render_level(
    mut commands: Commands,
    mut nav: ResMut<NavState>,
    existing: Query<Entity, With<LevelNode>>,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !nav.dirty {
        return;
    }

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let sphere = meshes.add(Sphere::new(1.0));
    let hub_mat = materials.add(glow_material(Color::srgb(0.45, 0.95, 1.0), 6.0));
    let child_mat = materials.add(glow_material(Color::srgb(0.2, 0.72, 0.78), 3.0));
    let parent_mat = materials.add(glow_material(Color::srgb(0.5, 0.55, 0.62), 1.5));
    let connector = meshes.add(Cylinder::new(0.05, 1.0));
    let connector_mat = materials.add(glow_material(Color::srgb(0.18, 0.62, 0.7), 2.2));

    let current = nav.current;
    let child_ids = nav.tree.children(current).to_vec();
    let positions = radial_layout(child_ids.len());

    // Hub = focus node at the origin.
    if let Some(node) = nav.tree.node(current) {
        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(hub_mat),
            Transform::from_translation(Vec3::ZERO).with_scale(Vec3::ZERO),
            LevelNode,
            NodeAnim {
                base: HUB_RADIUS,
                age: 0.0,
            },
            NodeLabel {
                name: node.name.clone(),
                count: child_ids.len(),
                kind: NodeKind::Hub,
            },
        ));
    }

    // Children orbit the hub on the radial ring (with a tiny z stagger for depth).
    for (idx, (child_id, position)) in child_ids.iter().copied().zip(positions).enumerate() {
        let (child_name, grandchildren) = nav
            .tree
            .node(child_id)
            .map(|node| (node.name.clone(), nav.tree.children(child_id).len()))
            .unwrap_or_else(|| (format!("folder-{child_id}"), 0));

        let z = ((idx % 3) as f32 - 1.0) * 0.6;
        let child_pos = Vec3::new(position.x, position.y, z);

        // Glowing connector from the hub to this child.
        let dir = child_pos;
        let len = dir.length().max(0.001);
        commands.spawn((
            Mesh3d(connector.clone()),
            MeshMaterial3d(connector_mat.clone()),
            Transform {
                translation: child_pos * 0.5,
                rotation: Quat::from_rotation_arc(Vec3::Y, dir / len),
                scale: Vec3::new(1.0, len, 1.0),
            },
            LevelNode,
        ));

        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(child_mat.clone()),
            Transform::from_translation(child_pos).with_scale(Vec3::ZERO),
            LevelNode,
            LevelChild(child_id),
            NodeAnim {
                base: CHILD_RADIUS,
                age: 0.0,
            },
            NodeLabel {
                name: child_name,
                count: grandchildren,
                kind: NodeKind::Child,
            },
        ));
    }

    // Parent ghost above the hub — hover it to ascend.
    if let Some(parent_id) = nav.tree.parent(current) {
        let ring_radius = nav::RADIAL_BASE + nav::RADIAL_PER_CHILD * child_ids.len() as f32;
        let parent_name = nav
            .tree
            .node(parent_id)
            .map(|node| node.name.clone())
            .unwrap_or_else(|| "..".to_string());
        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(parent_mat),
            Transform::from_translation(Vec3::new(0.0, ring_radius + 4.0, -1.0))
                .with_scale(Vec3::ZERO),
            LevelNode,
            ParentNode,
            NodeAnim {
                base: PARENT_RADIUS,
                age: 0.0,
            },
            NodeLabel {
                name: format!("\u{2191} {parent_name}"),
                count: 0,
                kind: NodeKind::Parent,
            },
        ));
    }

    // Pull the perspective camera back so the whole ring frames nicely.
    let ring_radius = nav::RADIAL_BASE + nav::RADIAL_PER_CHILD * child_ids.len() as f32;
    let distance = (ring_radius + 6.0) * 2.2;
    if let Ok(mut cam) = cameras.single_mut() {
        *cam = Transform::from_xyz(0.0, 0.0, distance).looking_at(Vec3::ZERO, Vec3::Y);
    }

    nav.dirty = false;
}

const HOVER_RADIUS_PX: f32 = 70.0;

// Which node (if any) is under the cursor right now.
fn node_under_cursor(
    cursor: Vec2,
    camera: &Camera,
    cam_tf: &GlobalTransform,
    children: &Query<(&LevelChild, &GlobalTransform)>,
    parents: &Query<&GlobalTransform, With<ParentNode>>,
) -> HoverTarget {
    let mut best_child: Option<(usize, f32)> = None;
    for (child, gt) in children {
        if let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) {
            let d = screen.distance(cursor);
            if d < HOVER_RADIUS_PX && best_child.is_none_or(|(_, bd)| d < bd) {
                best_child = Some((child.0, d));
            }
        }
    }

    let mut parent_d = f32::INFINITY;
    for gt in parents {
        if let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) {
            let d = screen.distance(cursor);
            if d < HOVER_RADIUS_PX {
                parent_d = parent_d.min(d);
            }
        }
    }

    match best_child {
        Some((id, cd)) if cd <= parent_d => HoverTarget::Child(id),
        _ if parent_d.is_finite() => HoverTarget::Parent,
        _ => HoverTarget::None,
    }
}

// Neuron-flow: hovering a node fills a 1s ring (DWELL_SECS); on completion we
// dive (child = descend, parent ghost = ascend). Moving off the node resets it.
pub fn hover_dwell(
    time: Res<Time>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    children: Query<(&LevelChild, &GlobalTransform)>,
    parents: Query<&GlobalTransform, With<ParentNode>>,
    mut hover: ResMut<HoverState>,
    mut nav: ResMut<NavState>,
) -> Result {
    let Ok(window) = windows.single() else {
        return Ok(());
    };
    let Some(cursor) = window.cursor_position() else {
        hover.target = HoverTarget::None;
        return Ok(());
    };
    if contexts.ctx_mut()?.is_pointer_over_egui() {
        hover.target = HoverTarget::None;
        return Ok(());
    }
    let Ok((camera, cam_tf)) = cameras.single() else {
        return Ok(());
    };

    let target = node_under_cursor(cursor, camera, cam_tf, &children, &parents);

    if target == HoverTarget::None || target != hover.target {
        hover.target = target;
        hover.elapsed = 0.0;
        return Ok(());
    }

    // Same target as last frame — accumulate dwell.
    hover.elapsed += time.delta_secs();
    if hover.elapsed < DWELL_SECS {
        return Ok(());
    }

    match hover.target {
        HoverTarget::Child(id) => {
            nav.current = id;
            nav.dirty = true;
        }
        HoverTarget::Parent => {
            if let Some(parent) = nav.tree.parent(nav.current) {
                nav.current = parent;
                nav.dirty = true;
            }
        }
        HoverTarget::None => {}
    }
    hover.target = HoverTarget::None;
    hover.elapsed = 0.0;

    Ok(())
}

// Draw the loading ring around the node being dwelled on.
pub fn draw_hover_ring(
    mut contexts: EguiContexts,
    hover: Res<HoverState>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    children: Query<(&LevelChild, &GlobalTransform)>,
    parents: Query<&GlobalTransform, With<ParentNode>>,
) -> Result {
    if hover.target == HoverTarget::None {
        return Ok(());
    }
    let Ok((camera, cam_tf)) = cameras.single() else {
        return Ok(());
    };

    // Resolve the dwelled node's world position.
    let world = match hover.target {
        HoverTarget::Child(id) => children
            .iter()
            .find(|(c, _)| c.0 == id)
            .map(|(_, gt)| gt.translation()),
        HoverTarget::Parent => parents.iter().next().map(|gt| gt.translation()),
        HoverTarget::None => None,
    };
    let Some(world) = world else {
        return Ok(());
    };
    let Ok(screen) = camera.world_to_viewport(cam_tf, world) else {
        return Ok(());
    };

    let progress = (hover.elapsed / DWELL_SECS).clamp(0.0, 1.0);
    let center = egui::pos2(screen.x, screen.y);
    let radius = 34.0;

    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("ring"),
    ));

    // Faint full track.
    painter.circle_stroke(
        center,
        radius,
        egui::Stroke::new(
            2.0,
            egui::Color32::from_rgba_unmultiplied(120, 200, 220, 60),
        ),
    );

    // Progress arc, clockwise from the top.
    let segments = 64;
    let filled = (segments as f32 * progress).ceil().max(1.0) as usize;
    let mut points = Vec::with_capacity(filled + 1);
    for i in 0..=filled {
        let t = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let angle = -std::f32::consts::FRAC_PI_2 + t;
        points.push(egui::pos2(
            center.x + angle.cos() * radius,
            center.y + angle.sin() * radius,
        ));
    }
    painter.add(egui::Shape::line(
        points,
        egui::Stroke::new(3.5, egui::Color32::from_rgb(110, 230, 245)),
    ));

    Ok(())
}

// Backspace ascends too (fallback to the hover-up gesture).
pub fn ascend(keys: Res<ButtonInput<KeyCode>>, mut nav: ResMut<NavState>) {
    if !keys.just_pressed(KeyCode::Backspace) {
        return;
    }
    if let Some(parent) = nav.tree.parent(nav.current) {
        nav.current = parent;
        nav.dirty = true;
    }
}

// Grow-in (scale 0→base, ease-out) + smooth hover bump, per node.
pub fn animate_nodes(
    time: Res<Time>,
    hover: Res<HoverState>,
    mut nodes: Query<(
        &mut Transform,
        &mut NodeAnim,
        Option<&LevelChild>,
        Option<&ParentNode>,
    )>,
) {
    let dt = time.delta_secs();
    for (mut tf, mut anim, child, parent) in &mut nodes {
        anim.age += dt;
        let g = (anim.age / GROW_SECS).clamp(0.0, 1.0);
        let grow = 1.0 - (1.0 - g).powi(3); // ease-out cubic

        let hovered = match hover.target {
            HoverTarget::Child(id) => child.is_some_and(|c| c.0 == id),
            HoverTarget::Parent => parent.is_some(),
            HoverTarget::None => false,
        };
        let target = anim.base * grow * if hovered { HOVER_BUMP } else { 1.0 };

        let current = tf.scale.x;
        let next = current + (target - current) * (dt * 12.0).min(1.0);
        tf.scale = Vec3::splat(next);
    }
}

// Draw each node's name (+ child count) at its projected screen position.
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
        let text = if label.count == 0 {
            label.name.clone()
        } else {
            format!("{}  ({})", label.name, label.count)
        };
        let (size, color, dy) = match label.kind {
            NodeKind::Hub => (16.0, egui::Color32::from_rgb(234, 250, 255), 22.0),
            NodeKind::Child => (12.5, egui::Color32::from_rgb(180, 222, 232), 16.0),
            NodeKind::Parent => (12.0, egui::Color32::from_rgb(150, 160, 175), 14.0),
        };
        painter.text(
            egui::pos2(screen.x, screen.y + dy),
            egui::Align2::CENTER_TOP,
            text,
            egui::FontId::proportional(size),
            color,
        );
    }

    Ok(())
}

// Breadcrumb top bar: NEURA · root / … / current.
#[allow(deprecated)] // egui marks Panel::show deprecated but show(ctx) is correct (matches ui.rs)
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
                    egui::RichText::new("mové el mouse para sumergirte")
                        .weak()
                        .size(11.0),
                );
            });
        });
    });

    Ok(())
}
