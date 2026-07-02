use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use super::*;
use crate::icons::{self, IconConfig, VisualKind, VisualSpec};
use crate::nav::{self, radial_layout};
use crate::pins::PinStore;

#[derive(Clone)]
pub struct CachedLevelAssets {
    sphere: Handle<Mesh>,
    quad: Handle<Mesh>,
    connector: Handle<Mesh>,
    hub_mat: Handle<StandardMaterial>,
    child_mat: Handle<StandardMaterial>,
    pinned_mat: Handle<StandardMaterial>,
    parent_mat: Handle<StandardMaterial>,
    connector_mat: Handle<StandardMaterial>,
    echo_mat: Handle<StandardMaterial>,
}

impl CachedLevelAssets {
    fn build(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let mut echo_material = glow_material(Color::srgba(0.55, 0.45, 0.95, 0.12), 0.8);
        echo_material.alpha_mode = AlphaMode::Blend;

        Self {
            sphere: meshes.add(Sphere::new(1.0)),
            quad: meshes.add(Rectangle::new(2.0, 2.0)),
            connector: meshes.add(Cylinder::new(0.05, 1.0)),
            hub_mat: materials.add(glow_material(Color::srgb(0.45, 0.95, 1.0), 6.0)),
            child_mat: materials.add(glow_material(Color::srgb(0.2, 0.72, 0.78), 3.0)),
            pinned_mat: materials.add(glow_material(Color::srgb(1.0, 0.84, 0.3), 4.0)),
            parent_mat: materials.add(glow_material(Color::srgb(0.5, 0.55, 0.62), 1.5)),
            connector_mat: materials.add(glow_material(Color::srgb(0.18, 0.62, 0.7), 2.2)),
            echo_mat: materials.add(echo_material),
        }
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
    mut cache: Local<Option<CachedLevelAssets>>,
) {
    if !nav.dirty {
        return;
    }

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    // Meshes/materiales compartidos se crean UNA vez (antes era por navegación).
    let assets = cache
        .get_or_insert_with(|| CachedLevelAssets::build(&mut meshes, &mut materials))
        .clone();

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
                material: assets.hub_mat.clone(),
                sphere: assets.sphere.clone(),
                quad: assets.quad.clone(),
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
            Mesh3d(assets.connector.clone()),
            MeshMaterial3d(assets.connector_mat.clone()),
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
            assets.pinned_mat.clone()
        } else {
            assets.child_mat.clone()
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
                sphere: assets.sphere.clone(),
                quad: assets.quad.clone(),
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
                material: assets.parent_mat.clone(),
                sphere: assets.sphere.clone(),
                quad: assets.quad.clone(),
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
        commands.spawn((
            Mesh3d(assets.sphere.clone()),
            MeshMaterial3d(assets.echo_mat.clone()),
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
