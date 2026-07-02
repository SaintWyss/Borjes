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

#[derive(Component)]
pub struct PreviewNode;

// Nombre de la subcarpeta que representa cada dendrita del preview.
#[derive(Component)]
pub struct PreviewLabel(pub String);

#[derive(Component)]
pub struct PreviewAnim {
    pub age: f32,
    pub full: f32,
    // Brote secuencial: cada dendrita aparece un instante después de la anterior.
    pub delay: f32,
}

#[derive(Resource, Default)]
pub struct PreviewState {
    pub target: Option<usize>,
    pub elapsed: f32,
    pub spawned_for: Option<usize>,
    pub overflow: Option<(usize, usize)>,
}

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
    // Grow-in stagger: la onda arranca en el hub y ondula por el anillo.
    pub delay: f32,
}

const GROW_SECS: f32 = 0.18;
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
                delay: 0.0,
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
                // Onda: cada hijo brota un pelo después del anterior, girando el anillo.
                delay: 0.05 + idx as f32 * 0.03,
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
                delay: 0.16,
            },
        );
    }

    if let Some(&prev) = nav.history.last().filter(|&&prev| prev != current) {
        let name = nav
            .tree
            .node(prev)
            .map(|node| node.name.clone())
            .unwrap_or_else(|| format!("folder-{prev}"));
        let mut echo_material = glow_material(Color::srgba(0.55, 0.45, 0.95, 0.12), 0.8);
        echo_material.alpha_mode = AlphaMode::Blend;
        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(materials.add(echo_material)),
            Transform::from_xyz(0.0, 0.0, -7.0).with_scale(Vec3::ZERO),
            LevelNode,
            EchoNode,
            NodeAnim {
                base: 2.6,
                age: 0.0,
                home: Vec3::new(0.0, 0.0, -7.0),
                phase: 0.0,
                delay: 0.1,
            },
            NodeLabel {
                name: format!("\u{27f2} {name}"),
                count: 0,
                kind: NodeKind::Echo,
            },
        ));
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
    delay: f32,
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
                delay: spec.delay,
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
    preview: Res<PreviewState>,
    children: Query<(&LevelChild, &GlobalTransform)>,
    mut cam: Query<&mut Transform, With<Camera3d>>,
    mut pan: Local<Vec2>,
) {
    let Ok(mut cam) = cam.single_mut() else {
        return;
    };

    const SPEED: f32 = 9.0;
    const PAN_SPEED: f32 = 5.0;
    // Acercamiento progresivo: arranca con un lean suave y, mientras el hover se
    // sostiene, la cámara va CENTRANDO el nodo y acercándose (el preview de sus
    // hijos aparece en el camino). La histéresis de hover_track evita que el
    // nodo "se escape" del cursor al moverse hacia el centro.
    const LEAN_MIN: f32 = 0.12;
    const LEAN_MAX: f32 = 0.55;
    const ZOOM_MAX: f32 = 0.18;

    // Rampa 0→1 durante ~0.5s de hover sostenido sobre el mismo hijo.
    let ramp = match hover.target {
        HoverTarget::Child(id) if preview.target == Some(id) => {
            ((preview.elapsed - 0.1) / 0.5).clamp(0.0, 1.0)
        }
        _ => 0.0,
    };
    let lean = LEAN_MIN + ramp * (LEAN_MAX - LEAN_MIN);

    let want = match hover.target {
        HoverTarget::Child(id) => children
            .iter()
            .find(|(c, _)| c.0 == id)
            .map(|(_, gt)| gt.translation().truncate() * lean)
            .unwrap_or(Vec2::ZERO),
        _ => Vec2::ZERO,
    };

    let dt = time.delta_secs();
    let cur = *pan;
    *pan = cur + (want - cur) * (1.0 - (-dt * PAN_SPEED).exp());

    let z = cam.translation.z;
    let z_target = target.distance * (1.0 - ZOOM_MAX * ramp);
    // Dive = rápido (SPEED); acercamiento por hover = suave (PAN_SPEED).
    let z_rate = if ramp > 0.0 { PAN_SPEED } else { SPEED };
    let eased_z = z + (z_target - z) * (1.0 - (-dt * z_rate).exp());
    let look = Vec3::new(pan.x, pan.y, 0.0);
    *cam = Transform::from_xyz(pan.x, pan.y, eased_z).looking_at(look, Vec3::Y);
}

const HOVER_RADIUS_PX: f32 = 95.0;
// Radio de "soltado": más grande que el de agarre, para que el centrado de
// cámara no corte el hover (ver histéresis en hover_track).
const STICKY_RADIUS_PX: f32 = 230.0;

// Which node (if any) is under the cursor right now.
fn node_under_cursor(
    cursor: Vec2,
    camera: &Camera,
    cam_tf: &GlobalTransform,
    children: &Query<(&LevelChild, &GlobalTransform)>,
    parents: &Query<&GlobalTransform, With<ParentNode>>,
    echoes: &Query<&GlobalTransform, With<EchoNode>>,
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

    let mut echo_d = f32::INFINITY;
    for gt in echoes {
        if let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) {
            let d = screen.distance(cursor);
            if d < HOVER_RADIUS_PX {
                echo_d = echo_d.min(d);
            }
        }
    }

    match best_child {
        Some((id, cd)) if cd <= parent_d && cd <= echo_d => HoverTarget::Child(id),
        _ if parent_d <= echo_d && parent_d.is_finite() => HoverTarget::Parent,
        _ if echo_d.is_finite() => HoverTarget::Echo,
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
    echoes: Query<&GlobalTransform, With<EchoNode>>,
    constellation: Res<crate::constellation::Constellation>,
    mut hover: ResMut<HoverState>,
) -> Result {
    // Con la constelación activa el nivel está oculto: nada hovereable.
    if constellation.active {
        hover.target = HoverTarget::None;
        return Ok(());
    }
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

    let fresh = node_under_cursor(cursor, camera, cam_tf, &children, &parents, &echoes);

    // Histéresis: mientras la cámara centra al hijo hovereado, el nodo se aleja
    // del cursor quieto. Si nada nuevo cae bajo el cursor, el hijo actual sigue
    // "agarrado" hasta que el cursor quede realmente lejos.
    hover.target = match (fresh, hover.target) {
        (HoverTarget::None, HoverTarget::Child(id)) => {
            let still_close = children
                .iter()
                .find(|(c, _)| c.0 == id)
                .and_then(|(_, gt)| camera.world_to_viewport(cam_tf, gt.translation()).ok())
                .is_some_and(|screen| screen.distance(cursor) < STICKY_RADIUS_PX);
            if still_close {
                HoverTarget::Child(id)
            } else {
                HoverTarget::None
            }
        }
        (fresh, _) => fresh,
    };
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
            nav.navigate(id);
        }
        HoverTarget::Parent => {
            if let Some(parent) = nav.tree.parent(nav.current) {
                nav.navigate(parent);
            }
        }
        HoverTarget::Echo => nav.back(),
        HoverTarget::None => {}
    }
    Ok(())
}

pub fn tab_hover_nav(
    keys: Res<ButtonInput<KeyCode>>,
    hover: Res<HoverState>,
    search: Res<crate::search::BorjesSearch>,
    mut nav: ResMut<NavState>,
) {
    if !keys.just_pressed(KeyCode::Tab) || !search.results.is_empty() {
        return;
    }

    match hover.target {
        HoverTarget::Child(id) => nav.navigate(id),
        HoverTarget::Parent => {
            if let Some(parent) = nav.tree.parent(nav.current) {
                nav.navigate(parent);
            }
        }
        HoverTarget::Echo => nav.back(),
        HoverTarget::None => {}
    }
}

pub fn update_hover_preview(
    mut commands: Commands,
    time: Res<Time>,
    nav: Res<NavState>,
    hover: Res<HoverState>,
    children: Query<(Entity, &LevelChild, &GlobalTransform)>,
    previews: Query<Entity, With<PreviewNode>>,
    mut state: ResMut<PreviewState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let target = match hover.target {
        HoverTarget::Child(id) if !nav.dirty => Some(id),
        _ => None,
    };

    if target != state.target || nav.dirty {
        // try_despawn: si el nivel se reconstruyó, el padre ya se llevó a las
        // dendritas (relación ChildOf) y el despawn directo duplicaría.
        for entity in &previews {
            commands.entity(entity).try_despawn();
        }
        state.target = target;
        state.elapsed = 0.0;
        state.spawned_for = None;
        state.overflow = None;
    }

    let Some(id) = state.target else {
        return;
    };

    state.elapsed += time.delta_secs();
    if state.elapsed < 0.15 || state.spawned_for == Some(id) {
        return;
    }

    let own_children = nav.tree.children(id);
    if own_children.is_empty() {
        state.spawned_for = Some(id);
        return;
    }

    let Some((hovered_entity, _, _)) = children.iter().find(|(_, child, _)| child.0 == id) else {
        return;
    };

    for entity in &previews {
        commands.entity(entity).try_despawn();
    }

    let mut material = glow_material(Color::srgba(0.3, 0.9, 0.85, 0.35), 1.2);
    material.alpha_mode = AlphaMode::Blend;
    let material = materials.add(material);
    let sphere = meshes.add(Sphere::new(1.0));
    let shown = own_children.len().min(8);

    // Anillo en el plano de pantalla (XY, la cámara mira por -Z) y como HIJOS del
    // nodo hovereado: heredan su drift y su bump, las dendritas viajan con él.
    // Empieza arriba (12 en punto) y brota en sentido horario, una por vez.
    for (idx, &preview_child) in own_children.iter().take(shown).enumerate() {
        let angle = std::f32::consts::FRAC_PI_2 - idx as f32 * std::f32::consts::TAU / shown as f32;
        let offset = Vec3::new(angle.cos() * 1.8, angle.sin() * 1.8, 0.35);
        let name = nav
            .tree
            .node(preview_child)
            .map(|node| node.name.clone())
            .unwrap_or_default();
        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(offset).with_scale(Vec3::ZERO),
            ChildOf(hovered_entity),
            PreviewNode,
            PreviewLabel(name),
            PreviewAnim {
                age: 0.0,
                full: 0.24,
                delay: idx as f32 * 0.025,
            },
        ));
    }

    state.spawned_for = Some(id);
    state.overflow = (own_children.len() > 8).then_some((id, own_children.len() - 8));
}

pub fn animate_previews(
    time: Res<Time>,
    mut previews: Query<(&mut Transform, &mut PreviewAnim), With<PreviewNode>>,
) {
    let dt = time.delta_secs();
    for (mut tf, mut anim) in &mut previews {
        anim.age += dt;
        let staggered = (anim.age - anim.delay).max(0.0);
        let g = (staggered / 0.12).clamp(0.0, 1.0);
        // Ease-out con mini overshoot: brota, pasa apenas y asienta.
        let grow = 1.0 - (1.0 - g).powi(3);
        let pop = 1.0 + 0.25 * (g * std::f32::consts::PI).sin() * (1.0 - g);
        tf.scale = Vec3::splat(anim.full * grow * pop);
    }
}

// Nombres de las subcarpetas del preview, pegados a cada dendrita. Aparecen en
// fade siguiendo el brote de su esfera.
pub fn draw_preview_labels(
    mut contexts: EguiContexts,
    cameras: Query<(&Camera, &GlobalTransform)>,
    previews: Query<(&GlobalTransform, &PreviewLabel, &PreviewAnim)>,
) -> Result {
    if previews.is_empty() {
        return Ok(());
    }
    let Ok((camera, cam_tf)) = cameras.single() else {
        return Ok(());
    };
    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("preview_labels"),
    ));

    for (gt, label, anim) in &previews {
        let fade = ((anim.age - anim.delay) / 0.12).clamp(0.0, 1.0);
        if fade <= 0.0 || label.0.is_empty() {
            continue;
        }
        let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) else {
            continue;
        };
        painter.text(
            egui::pos2(screen.x, screen.y + 10.0),
            egui::Align2::CENTER_TOP,
            &label.0,
            egui::FontId::proportional(10.5),
            egui::Color32::from_rgba_unmultiplied(150, 225, 220, (fade * 235.0) as u8),
        );
    }

    Ok(())
}

// Draw a reticle around the hovered (clickeable) node: two slowly-rotating arcs.
pub fn draw_hover_ring(
    mut contexts: EguiContexts,
    time: Res<Time>,
    hover: Res<HoverState>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    children: Query<(&LevelChild, &GlobalTransform)>,
    parents: Query<&GlobalTransform, With<ParentNode>>,
    echoes: Query<&GlobalTransform, With<EchoNode>>,
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
        HoverTarget::Echo => echoes.iter().next().map(|gt| gt.translation()),
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

    // Retícula: dos arcos de 110° enfrentados que giran despacio — señala
    // "clickeable" sin ser un círculo muerto. Halo interior tenue de apoyo.
    let spin = time.elapsed_secs() * 0.9;
    let stroke = egui::Stroke::new(2.5, egui::Color32::from_rgb(110, 230, 245));
    for half in 0..2 {
        let start = spin + half as f32 * std::f32::consts::PI;
        let points: Vec<egui::Pos2> = (0..=18)
            .map(|i| {
                let a = start + i as f32 / 18.0 * 1.92; // ~110°
                egui::pos2(center.x + a.cos() * 38.0, center.y + a.sin() * 38.0)
            })
            .collect();
        painter.add(egui::Shape::line(points, stroke));
    }
    painter.circle_stroke(
        center,
        30.0,
        egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_premultiplied(60, 130, 140, 90),
        ),
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
        nav.navigate(parent);
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
        Option<&EchoNode>,
    )>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    for (mut tf, mut anim, child, parent, echo) in &mut nodes {
        anim.age += dt;
        let staggered = (anim.age - anim.delay).max(0.0);
        let g = (staggered / GROW_SECS).clamp(0.0, 1.0);
        let grow = 1.0 - (1.0 - g).powi(3); // ease-out cubic

        let hovered = match hover.target {
            HoverTarget::Child(id) => child.is_some_and(|c| c.0 == id),
            HoverTarget::Parent => parent.is_some(),
            HoverTarget::Echo => echo.is_some(),
            HoverTarget::None => false,
        };
        // El eco respira despacio: recuerdo vivo, no escenografía muerta.
        let breathe = if echo.is_some() {
            1.0 + 0.04 * (t * 0.8).sin()
        } else {
            1.0
        };
        let target = anim.base * grow * breathe * if hovered { HOVER_BUMP } else { 1.0 };

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
    labels: Query<(&NodeLabel, &GlobalTransform, Option<&LevelFolder>)>,
    constellation: Res<crate::constellation::Constellation>,
    lenses: Res<crate::lens::Lenses>,
    nav: Res<NavState>,
) -> Result {
    // Nivel oculto durante la constelación: sus labels tampoco se pintan.
    if constellation.active {
        return Ok(());
    }
    let Ok((camera, cam_tf)) = cameras.single() else {
        return Ok(());
    };

    // Keywords normalizadas una vez por frame: el anillo de lente es barato.
    let lens_index: Vec<((u8, u8, u8), Vec<String>)> = lenses
        .all
        .iter()
        .filter_map(|l| {
            let color = crate::lens::parse_color(&l.color)?;
            let kws = l.keywords.iter().map(|k| crate::ui::normalize(k)).collect();
            Some((color, kws))
        })
        .collect();
    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("labels"),
    ));

    for (label, gt, folder) in &labels {
        let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) else {
            continue;
        };

        // Miembro de una lente → anillo fino de su color (identidad temática).
        if matches!(label.kind, NodeKind::Hub | NodeKind::Child) {
            if let Some(folder) = folder {
                if let Some(norm_name) = nav.tree.normalized(folder.0) {
                    if let Some(((r, g, b), _)) = lens_index.iter().find(|(_, kws)| {
                        kws.iter()
                            .any(|kw| crate::search::score_normalized(norm_name, kw) > 0)
                    }) {
                        painter.circle_stroke(
                            egui::pos2(screen.x, screen.y),
                            26.0,
                            egui::Stroke::new(1.5, egui::Color32::from_rgb(*r, *g, *b)),
                        );
                    }
                }
            }
        }
        let text = if label.count == 0 {
            label.name.clone()
        } else {
            format!("{}  ({})", label.name, label.count)
        };
        let (size, color, dy) = match label.kind {
            NodeKind::Hub => (16.0, egui::Color32::from_rgb(234, 250, 255), 22.0),
            NodeKind::Child => (12.5, egui::Color32::from_rgb(180, 222, 232), 16.0),
            NodeKind::Parent => (12.0, egui::Color32::from_rgb(150, 160, 175), 14.0),
            NodeKind::Echo => (12.0, egui::Color32::from_rgb(150, 135, 230), 28.0),
        };
        painter.text(
            egui::pos2(screen.x, screen.y + dy),
            egui::Align2::CENTER_TOP,
            text,
            egui::FontId::proportional(size),
            color,
        );
        // Carpeta sin subcarpetas: el vacío invita a actuar, no a mirar.
        if label.kind == NodeKind::Hub && label.count == 0 {
            painter.text(
                egui::pos2(screen.x, screen.y + dy + 22.0),
                egui::Align2::CENTER_TOP,
                "sin subcarpetas · Enter la abre en el explorador",
                egui::FontId::proportional(11.5),
                egui::Color32::from_rgb(110, 150, 160),
            );
        }
    }

    Ok(())
}

pub fn draw_preview_overflow(
    mut contexts: EguiContexts,
    state: Res<PreviewState>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    children: Query<(&LevelChild, &GlobalTransform)>,
) -> Result {
    let Some((id, hidden)) = state.overflow else {
        return Ok(());
    };
    let Ok((camera, cam_tf)) = cameras.single() else {
        return Ok(());
    };
    let Some((_, gt)) = children.iter().find(|(child, _)| child.0 == id) else {
        return Ok(());
    };
    let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation() + Vec3::new(1.2, 0.4, 0.0))
    else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("preview_overflow"),
    ));
    painter.text(
        egui::pos2(screen.x, screen.y),
        egui::Align2::CENTER_CENTER,
        format!("+{hidden}"),
        egui::FontId::proportional(11.0),
        egui::Color32::from_rgb(80, 190, 185),
    );

    Ok(())
}

// Breadcrumb top bar: BORJES · root / … / current. Cada segmento es un salto:
// click en un ancestro navega directo ahí (mismo history que el resto).
#[allow(deprecated)] // egui marks Panel::show deprecated but show(ctx) is correct (matches ui.rs)
pub fn draw_breadcrumb(mut contexts: EguiContexts, mut nav: ResMut<NavState>) -> Result {
    let mut chain: Vec<(usize, String)> = Vec::new();
    let mut cursor = Some(nav.current);
    while let Some(id) = cursor {
        if let Some(node) = nav.tree.node(id) {
            chain.push((id, node.name.clone()));
        }
        cursor = nav.tree.parent(id);
    }
    chain.reverse();

    let ctx = contexts.ctx_mut()?;
    let mut jump = None;
    egui::TopBottomPanel::top("breadcrumb").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("BORJES")
                    .strong()
                    .color(egui::Color32::from_rgb(90, 220, 235)),
            );
            ui.label(egui::RichText::new("·").weak());
            let last = chain.len().saturating_sub(1);
            for (pos, (id, name)) in chain.iter().enumerate() {
                if pos == last {
                    // El nivel actual no es un link: es donde estás parado.
                    ui.label(
                        egui::RichText::new(name)
                            .strong()
                            .color(egui::Color32::from_rgb(234, 250, 255)),
                    );
                } else {
                    let seg = ui.add(
                        egui::Label::new(
                            egui::RichText::new(name).color(egui::Color32::from_rgb(150, 200, 215)),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if seg.hovered() {
                        seg.clone().highlight();
                    }
                    if seg.clicked() {
                        jump = Some(*id);
                    }
                    ui.label(egui::RichText::new("/").weak());
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(
                        "clic entra · Tab entra al hover · Backspace sube · breadcrumb salta",
                    )
                    .weak()
                    .size(11.0),
                );
            });
        });
    });

    if let Some(id) = jump {
        nav.navigate(id);
    }

    Ok(())
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
