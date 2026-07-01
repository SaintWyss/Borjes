use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::icons::{self, IconConfig, VisualKind, VisualSpec, ICON_IDS};
use crate::nav::{self, radial_layout};
use crate::pins::{self, PinStore};

#[derive(Resource)]
pub struct NavState {
    pub tree: nav::FolderTree,
    pub current: usize,
    pub dirty: bool,
}

#[derive(Resource)]
pub struct CamTarget {
    pub distance: f32,
}

impl Default for CamTarget {
    fn default() -> Self {
        Self { distance: 60.0 }
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
}

const HUB_RADIUS: f32 = 1.4;
const CHILD_RADIUS: f32 = 0.9;
const PARENT_RADIUS: f32 = 0.7;

// Per-node grow-in + hover-bump animation.
#[derive(Component)]
pub struct NodeAnim {
    pub base: f32,
    pub age: f32,
    // Rest position; organic drift wobbles the node around it (children only).
    pub home: Vec3,
    pub phase: f32,
}

const GROW_SECS: f32 = 0.28;
const HOVER_BUMP: f32 = 1.28;
// Organic drift: children sway gently around home for the "alive neuron" feel.
// Kept small so the field stays calm and clickable.
const DRIFT_AMP: f32 = 0.15;

// A hub→child connector that re-aims at the (drifting) child each frame.
#[derive(Component)]
pub struct Connector {
    pub child: usize,
}

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

fn textured_material(texture: Handle<Image>) -> StandardMaterial {
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

pub fn render_level(
    mut commands: Commands,
    mut nav: ResMut<NavState>,
    mut cam_target: ResMut<CamTarget>,
    existing: Query<Entity, With<LevelNode>>,
    asset_server: Res<AssetServer>,
    icon_cfg: Res<IconConfig>,
    pin_store: Res<PinStore>,
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
    let quad = meshes.add(Rectangle::new(2.0, 2.0));
    let hub_mat = materials.add(glow_material(Color::srgb(0.45, 0.95, 1.0), 6.0));
    let child_mat = materials.add(glow_material(Color::srgb(0.2, 0.72, 0.78), 3.0));
    let pinned_child_mat = materials.add(glow_material(Color::srgb(1.0, 0.84, 0.3), 4.0));
    let parent_mat = materials.add(glow_material(Color::srgb(0.5, 0.55, 0.62), 1.5));
    let connector = meshes.add(Cylinder::new(0.05, 1.0));
    let connector_mat = materials.add(glow_material(Color::srgb(0.18, 0.62, 0.7), 2.2));

    let current = nav.current;
    let child_ids = nav.tree.children(current).to_vec();
    let positions = radial_layout(child_ids.len());

    // Hub = focus node at the origin.
    if let Some(node) = nav.tree.node(current) {
        let visual = icons::visual_for(&node.path, &node.name, &icon_cfg);
        spawn_folder_node(
            &mut commands,
            &asset_server,
            &mut materials,
            NodeSpawn {
                id: current,
                name: node.name.clone(),
                count: child_ids.len(),
                kind: NodeKind::Hub,
                translation: Vec3::ZERO,
                base: HUB_RADIUS,
                material: hub_mat.clone(),
                sphere: sphere.clone(),
                quad: quad.clone(),
                visual,
                role: NodeRole::Hub,
            },
        );
    }

    // Children orbit the hub on the radial ring (with a tiny z stagger for depth).
    for (idx, (child_id, position)) in child_ids.iter().copied().zip(positions).enumerate() {
        let (child_name, child_path, grandchildren) = nav
            .tree
            .node(child_id)
            .map(|node| {
                (
                    node.name.clone(),
                    node.path.clone(),
                    nav.tree.children(child_id).len(),
                )
            })
            .unwrap_or_else(|| {
                (
                    format!("folder-{child_id}"),
                    std::path::PathBuf::from(format!("folder-{child_id}")),
                    0,
                )
            });

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
            Connector { child: child_id },
        ));

        let visual = icons::visual_for(&child_path, &child_name, &icon_cfg);
        let material = if pin_store.0.contains(&child_path) {
            pinned_child_mat.clone()
        } else {
            child_mat.clone()
        };
        spawn_folder_node(
            &mut commands,
            &asset_server,
            &mut materials,
            NodeSpawn {
                id: child_id,
                name: child_name,
                count: grandchildren,
                kind: NodeKind::Child,
                translation: child_pos,
                base: CHILD_RADIUS,
                material,
                sphere: sphere.clone(),
                quad: quad.clone(),
                visual,
                role: NodeRole::Child(child_id),
            },
        );
    }

    // Parent ghost above the hub — hover it to ascend.
    if let Some(parent_id) = nav.tree.parent(current) {
        let ring_radius = nav::RADIAL_BASE + nav::RADIAL_PER_CHILD * child_ids.len() as f32;
        let (parent_name, parent_path) = nav
            .tree
            .node(parent_id)
            .map(|node| (node.name.clone(), node.path.clone()))
            .unwrap_or_else(|| ("..".to_string(), std::path::PathBuf::from("..")));
        let visual = icons::visual_for(&parent_path, &parent_name, &icon_cfg);
        spawn_folder_node(
            &mut commands,
            &asset_server,
            &mut materials,
            NodeSpawn {
                id: parent_id,
                name: format!("\u{2191} {parent_name}"),
                count: 0,
                kind: NodeKind::Parent,
                translation: Vec3::new(0.0, ring_radius + 4.0, -1.0),
                base: PARENT_RADIUS,
                material: parent_mat,
                sphere: sphere.clone(),
                quad: quad.clone(),
                visual,
                role: NodeRole::Parent,
            },
        );
    }

    // Pull the perspective camera back so the whole ring frames nicely.
    let ring_radius = nav::RADIAL_BASE + nav::RADIAL_PER_CHILD * child_ids.len() as f32;
    let distance = (ring_radius + 6.0) * 2.2;
    cam_target.distance = distance;

    nav.dirty = false;
}

enum NodeRole {
    Hub,
    Child(usize),
    Parent,
}

struct NodeSpawn {
    id: usize,
    name: String,
    count: usize,
    kind: NodeKind,
    translation: Vec3,
    base: f32,
    material: Handle<StandardMaterial>,
    sphere: Handle<Mesh>,
    quad: Handle<Mesh>,
    visual: VisualSpec,
    role: NodeRole,
}

fn spawn_folder_node(
    commands: &mut Commands,
    asset_server: &AssetServer,
    materials: &mut Assets<StandardMaterial>,
    spec: NodeSpawn,
) {
    let entity = commands
        .spawn((
            Transform::from_translation(spec.translation).with_scale(Vec3::ZERO),
            Visibility::default(),
            LevelNode,
            LevelFolder(spec.id),
            NodeAnim {
                base: spec.base,
                age: 0.0,
                home: spec.translation,
                // Per-node phase offset so siblings drift out of sync.
                phase: (spec.id as f32 * 2.399_963) % std::f32::consts::TAU,
            },
            NodeLabel {
                name: spec.name,
                count: spec.count,
                kind: spec.kind,
            },
        ))
        .id();

    match spec.role {
        NodeRole::Hub => {}
        NodeRole::Child(id) => {
            commands.entity(entity).insert(LevelChild(id));
        }
        NodeRole::Parent => {
            commands.entity(entity).insert(ParentNode);
        }
    }

    commands.entity(entity).with_children(|parent| {
        // Only a bare Sphere node draws the glowing ball; icon/emoji/model nodes
        // show just their image (the icon itself glows via emissive_texture).
        let needs_sphere = matches!(spec.visual.kind, VisualKind::Sphere);
        if needs_sphere {
            parent.spawn((
                Mesh3d(spec.sphere.clone()),
                MeshMaterial3d(spec.material.clone()),
                Transform::default(),
            ));
        }

        match spec.visual.kind {
            VisualKind::Sphere => {}
            VisualKind::Png => {
                let Some(asset) = spec
                    .visual
                    .asset
                    .as_deref()
                    .and_then(icons::resolve_asset_path)
                else {
                    return;
                };
                let material = materials.add(textured_material(asset_server.load(asset)));
                parent.spawn((
                    Mesh3d(spec.quad.clone()),
                    MeshMaterial3d(material),
                    // In front of the halo sphere's front pole (radius 1.0)
                    // so the glow sits behind the icon instead of covering it.
                    Transform::from_xyz(0.0, 0.0, 1.2)
                        .with_scale(Vec3::splat(spec.visual.sane_scale())),
                ));
            }
            VisualKind::Emoji => {
                let Some(asset) = spec
                    .visual
                    .emoji
                    .as_deref()
                    .and_then(icons::ensure_emoji_png)
                    .and_then(|path| icons::resolve_asset_path(&path))
                else {
                    return;
                };
                let material = materials.add(textured_material(asset_server.load(asset)));
                parent.spawn((
                    Mesh3d(spec.quad.clone()),
                    MeshMaterial3d(material),
                    // In front of the halo sphere's front pole (radius 1.0)
                    // so the glow sits behind the icon instead of covering it.
                    Transform::from_xyz(0.0, 0.0, 1.2)
                        .with_scale(Vec3::splat(spec.visual.sane_scale())),
                ));
            }
            VisualKind::Model => {
                let Some(asset) = spec
                    .visual
                    .asset
                    .as_deref()
                    .and_then(icons::resolve_asset_path)
                else {
                    return;
                };
                parent.spawn((
                    WorldAssetRoot(
                        asset_server.load(GltfAssetLabel::Scene(0).from_asset(asset.clone())),
                    ),
                    Transform::from_scale(Vec3::splat(spec.visual.sane_scale())),
                ));
            }
        }
    });
}

pub fn animate_camera(
    time: Res<Time>,
    target: Res<CamTarget>,
    hover: Res<HoverState>,
    children: Query<(&LevelChild, &GlobalTransform)>,
    mut cam: Query<&mut Transform, With<Camera3d>>,
    mut pan: Local<Vec2>,
) {
    let Ok(mut cam) = cam.single_mut() else {
        return;
    };

    const SPEED: f32 = 6.0;
    const PAN_SPEED: f32 = 5.0;
    // Fraction of the way the view leans toward the hovered child. Kept gentle
    // (mock uses 0.18) so the node doesn't slide out from under a still cursor
    // before the dwell ring fills.
    const LEAN: f32 = 0.12;

    // Lean target: 0 when nothing is hovered, else 18% toward the hovered child
    // (the hub sits at the origin, so the child's world xy is the full vector).
    let want = match hover.target {
        HoverTarget::Child(id) => children
            .iter()
            .find(|(c, _)| c.0 == id)
            .map(|(_, gt)| gt.translation().truncate() * LEAN)
            .unwrap_or(Vec2::ZERO),
        _ => Vec2::ZERO,
    };

    let dt = time.delta_secs();
    let cur = *pan;
    *pan = cur + (want - cur) * (1.0 - (-dt * PAN_SPEED).exp());

    let z = cam.translation.z;
    let eased_z = z + (target.distance - z) * (1.0 - (-dt * SPEED).exp());
    let look = Vec3::new(pan.x, pan.y, 0.0);
    *cam = Transform::from_xyz(pan.x, pan.y, eased_z).looking_at(look, Vec3::Y);
}

const HOVER_RADIUS_PX: f32 = 95.0;

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

fn folder_under_cursor(
    cursor: Vec2,
    camera: &Camera,
    cam_tf: &GlobalTransform,
    folders: &Query<(&LevelFolder, &GlobalTransform)>,
) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (folder, gt) in folders {
        if let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) {
            let d = screen.distance(cursor);
            if d < HOVER_RADIUS_PX && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((folder.0, d));
            }
        }
    }
    best.map(|(id, _)| id)
}

pub fn open_icon_menu(
    mut contexts: EguiContexts,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    folders: Query<(&LevelFolder, &GlobalTransform)>,
    nav: Res<NavState>,
    icon_cfg: Res<IconConfig>,
    mut menu: ResMut<IconMenu>,
    mut hover: ResMut<HoverState>,
) -> Result {
    if !buttons.just_pressed(MouseButton::Right) || contexts.ctx_mut()?.is_pointer_over_egui() {
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
    let Some(node_id) = folder_under_cursor(cursor, camera, cam_tf, &folders) else {
        return Ok(());
    };
    let Some(node) = nav.tree.node(node_id) else {
        return Ok(());
    };

    let visual = icons::visual_for(&node.path, &node.name, &icon_cfg);
    menu.open = true;
    menu.node_id = Some(node_id);
    menu.name = node.name.clone();
    menu.path = node.path.clone();
    menu.pos = cursor;
    menu.emoji = visual.emoji.clone().unwrap_or_else(|| "📁".to_string());
    menu.png_asset = visual
        .asset
        .clone()
        .unwrap_or_else(|| "icons/folder.png".to_string());
    menu.model_asset = visual
        .asset
        .clone()
        .unwrap_or_else(|| "custom/model.glb".to_string());
    menu.scale = visual.sane_scale();
    menu.halo = visual.halo;
    menu.just_opened = true;
    hover.target = HoverTarget::None;

    Ok(())
}

pub fn draw_icon_menu(
    mut contexts: EguiContexts,
    mut icon_cfg: ResMut<IconConfig>,
    mut pin_store: ResMut<PinStore>,
    mut nav: ResMut<NavState>,
    mut menu: ResMut<IconMenu>,
) -> Result {
    if !menu.open {
        return Ok(());
    }

    let Some(node_id) = menu.node_id else {
        menu.open = false;
        return Ok(());
    };
    let Some(node) = nav.tree.node(node_id) else {
        menu.open = false;
        return Ok(());
    };
    let node_path = node.path.clone();
    let node_name = node.name.clone();

    let ctx = contexts.ctx_mut()?;
    let style = ctx.global_style();
    let pos = menu.pos;
    let mut close = false;
    let mut changed = false;

    let was_just_opened = std::mem::take(&mut menu.just_opened);
    let area_response = egui::Area::new(egui::Id::new("icon_menu"))
        .fixed_pos(egui::pos2(pos.x, pos.y))
        .show(ctx, |ui| {
            egui::Frame::menu(&style).show(ui, |ui| {
                ui.set_min_width(260.0);
                ui.label(egui::RichText::new(&menu.name).strong());
                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    for id in ICON_IDS {
                        if ui.small_button(*id).clicked() {
                            icons::set_override(&mut icon_cfg, &node_path, VisualSpec::builtin(id));
                            changed = true;
                            close = true;
                        }
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Emoji");
                    ui.text_edit_singleline(&mut menu.emoji);
                    if ui.button("Usar").clicked() {
                        icons::set_override(
                            &mut icon_cfg,
                            &node_path,
                            VisualSpec::emoji(menu.emoji.clone(), menu.scale, menu.halo),
                        );
                        changed = true;
                        close = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("PNG");
                    ui.text_edit_singleline(&mut menu.png_asset);
                    if ui.button("Usar").clicked() {
                        icons::set_override(
                            &mut icon_cfg,
                            &node_path,
                            VisualSpec::png(menu.png_asset.clone(), menu.scale, menu.halo),
                        );
                        changed = true;
                        close = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("GLB");
                    ui.text_edit_singleline(&mut menu.model_asset);
                    if ui.button("Usar").clicked() {
                        icons::set_override(
                            &mut icon_cfg,
                            &node_path,
                            VisualSpec::model(menu.model_asset.clone(), menu.scale, menu.halo),
                        );
                        changed = true;
                        close = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Scale");
                    ui.add(
                        egui::DragValue::new(&mut menu.scale)
                            .range(0.2..=4.0)
                            .speed(0.05),
                    );
                    ui.checkbox(&mut menu.halo, "halo");
                });

                ui.separator();
                let is_pinned = pin_store.0.contains(&menu.path);
                let pin_label = if is_pinned {
                    "📌 Quitar pin"
                } else {
                    "📌 Pin"
                };
                if ui.button(pin_label).clicked() {
                    if is_pinned {
                        pin_store.0.remove(&menu.path);
                    } else {
                        pin_store.0.insert(menu.path.clone());
                    }
                    pins::save_pins(&pin_store.0);
                    nav.dirty = true;
                    close = true;
                }

                ui.separator();
                if ui.button("Esfera default").clicked() {
                    icons::set_override(&mut icon_cfg, &node_path, VisualSpec::sphere());
                    changed = true;
                    close = true;
                }
                if ui.button("Reset override").clicked() {
                    icons::clear_override(&mut icon_cfg, &node_path, &node_name);
                    changed = true;
                    close = true;
                }
            });
        });

    let dismissed = !was_just_opened && area_response.response.clicked_elsewhere();
    if changed {
        icons::save_icon_config(&icon_cfg);
        nav.dirty = true;
    }
    if close || dismissed {
        menu.open = false;
    }

    Ok(())
}

// Hover only *highlights* the node under the cursor (scale bump + gentle camera
// lean). Diving is an explicit left-click — auto-dive felt uncontrollable.
pub fn hover_track(
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    children: Query<(&LevelChild, &GlobalTransform)>,
    parents: Query<&GlobalTransform, With<ParentNode>>,
    mut hover: ResMut<HoverState>,
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

    hover.target = node_under_cursor(cursor, camera, cam_tf, &children, &parents);
    Ok(())
}

// Left-click the hovered node to dive: child = descend, parent ghost = ascend.
pub fn click_dive(
    mut contexts: EguiContexts,
    buttons: Res<ButtonInput<MouseButton>>,
    hover: Res<HoverState>,
    mut nav: ResMut<NavState>,
) -> Result {
    if !buttons.just_pressed(MouseButton::Left) || contexts.ctx_mut()?.is_pointer_over_egui() {
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
    Ok(())
}

// Draw a static highlight ring around the hovered (clickeable) node.
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

    let center = egui::pos2(screen.x, screen.y);

    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("ring"),
    ));

    // Static "clickeable" ring: marks the node a left-click would dive into.
    painter.circle_stroke(
        center,
        38.0,
        egui::Stroke::new(2.5, egui::Color32::from_rgb(110, 230, 245)),
    );

    Ok(())
}

// Backspace ascends too (fallback to the hover-up gesture).
pub fn ascend(
    keys: Res<ButtonInput<KeyCode>>,
    mut nav: ResMut<NavState>,
    mut search: ResMut<crate::search::BorjesSearch>,
) {
    let requested = keys.just_pressed(KeyCode::Backspace) || search.ascend_requested;
    search.ascend_requested = false;

    if !requested {
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
    let t = time.elapsed_secs();
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

        // Children wobble organically around home (mock: cos(t*0.6)/sin(t*0.5)).
        if child.is_some() {
            let dx = (t * 0.6 + anim.phase).cos() * DRIFT_AMP;
            let dy = (t * 0.5 + anim.phase).sin() * DRIFT_AMP;
            tf.translation = anim.home + Vec3::new(dx, dy, 0.0);
        }
    }
}

// Re-aim each hub→child connector at the child's drifting position so the edges
// stay attached and sway with the nodes. O(n²) over children, but n is tiny.
pub fn animate_connectors(
    children: Query<(&LevelChild, &GlobalTransform)>,
    mut connectors: Query<(&Connector, &mut Transform)>,
) {
    for (conn, mut tf) in &mut connectors {
        let Some((_, gt)) = children.iter().find(|(c, _)| c.0 == conn.child) else {
            continue;
        };
        let p = gt.translation();
        let len = p.length().max(0.001);
        tf.translation = p * 0.5;
        tf.rotation = Quat::from_rotation_arc(Vec3::Y, p / len);
        tf.scale = Vec3::new(1.0, len, 1.0);
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

// Breadcrumb top bar: BORJES · root / … / current.
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
                egui::RichText::new("BORJES")
                    .strong()
                    .color(egui::Color32::from_rgb(90, 220, 235)),
            );
            ui.label(egui::RichText::new("·").weak());
            ui.label(egui::RichText::new(path).color(egui::Color32::from_rgb(207, 233, 245)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("clic para entrar · ↑ padre para subir · Backspace")
                        .weak()
                        .size(11.0),
                );
            });
        });
    });

    Ok(())
}
