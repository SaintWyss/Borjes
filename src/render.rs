use std::collections::HashMap;

use bevy::prelude::*;
use rand::Rng;

use crate::interaction::OriginalMaterial;
use crate::scan::FolderNode;

#[derive(Resource)]
pub struct GraphData(pub Vec<FolderNode>);

#[derive(Component)]
pub struct NodeRef(pub usize);

const MIN_NODE_SCALE: f32 = 0.5;
const MAX_NODE_SCALE: f32 = 3.0;
const NODE_SCALE_DIVISOR: f32 = 8.0;

// File nodes are deliberately smaller than MIN_NODE_SCALE so they read as leaves.
#[allow(dead_code)] // consumed by plan 07-02
pub const FILE_NODE_SCALE: f32 = 0.35;
// Must clear MAX_NODE_SCALE so burst files are not buried in the parent folder.
#[allow(dead_code)] // consumed by plan 07-02
pub const BURST_RADIUS: f32 = 4.0;

#[derive(Resource)]
#[allow(dead_code)] // fields read by plan 07-02 burst spawn
pub struct FileAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

pub fn node_scale(child_count: usize) -> f32 {
    let normalized = (child_count as f32).sqrt() / NODE_SCALE_DIVISOR;
    let scale = MIN_NODE_SCALE + (MAX_NODE_SCALE - MIN_NODE_SCALE) * normalized.min(1.0);

    scale.clamp(MIN_NODE_SCALE, MAX_NODE_SCALE)
}

pub fn setup_file_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(0.4, 0.4, 0.4));
    let gray = Color::srgb(0.6, 0.6, 0.62);
    let material = materials.add(StandardMaterial {
        base_color: gray,
        emissive: gray.to_linear(),
        unlit: true,
        ..default()
    });

    commands.insert_resource(FileAssets { mesh, material });
}

#[allow(dead_code)] // consumed by plan 07-02
pub fn fibonacci_offset(i: usize, n: usize) -> Vec3 {
    let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
    let y = if n <= 1 {
        0.0
    } else {
        1.0 - (i as f32 / (n - 1) as f32) * 2.0
    };
    let ring_radius = (1.0 - y * y).max(0.0).sqrt();
    let theta = i as f32 * golden_angle;

    // Callers must special-case n == 1 with Vec3::Y * BURST_RADIUS to avoid overlap.
    Vec3::new(theta.cos() * ring_radius, y, theta.sin() * ring_radius)
}

#[allow(dead_code)]
pub fn seed_random_positions(nodes: &mut [FolderNode]) {
    let mut rng = rand::thread_rng();

    for node in nodes {
        node.pos = [
            rng.gen_range(-50.0..50.0),
            rng.gen_range(-50.0..50.0),
            rng.gen_range(-50.0..50.0),
        ];
    }
}

#[allow(dead_code)]
pub fn spawn_graph(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    graph: Res<GraphData>,
) {
    spawn_graph_nodes(&mut commands, &mut meshes, &mut materials, &graph);
}

pub fn spawn_graph_nodes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    graph: &GraphData,
) {
    let sphere = meshes.add(Sphere::new(0.5));
    let symlink_color = Color::srgb(1.0, 0.0, 0.95);
    let symlink_material = materials.add(StandardMaterial {
        base_color: symlink_color,
        emissive: symlink_color.to_linear(),
        unlit: true,
        ..default()
    });
    let mut cluster_materials: HashMap<usize, Handle<StandardMaterial>> = HashMap::new();

    for node in &graph.0 {
        let material = if node.is_symlink {
            symlink_material.clone()
        } else {
            match cluster_materials.get(&node.root_group) {
                Some(material) => material.clone(),
                None => {
                    let color = cluster_color(cluster_materials.len());
                    let material = materials.add(StandardMaterial {
                        base_color: color,
                        emissive: color.to_linear(),
                        unlit: true,
                        ..default()
                    });
                    cluster_materials.insert(node.root_group, material.clone());
                    material
                }
            }
        };

        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(material.clone()),
            OriginalMaterial(material),
            Transform::from_translation(Vec3::from_array(node.pos))
                .with_scale(Vec3::splat(node_scale(node.child_count))),
            NodeRef(node.id),
        ));
    }
}

pub fn draw_edges(mut gizmos: Gizmos, graph: Res<GraphData>) {
    for node in &graph.0 {
        let Some(parent_id) = node.parent_id else {
            continue;
        };

        let Some(parent) = graph.0.get(parent_id) else {
            continue;
        };

        gizmos.line(
            Vec3::from_array(node.pos),
            Vec3::from_array(parent.pos),
            Color::srgba(1.0, 1.0, 1.0, 0.15),
        );
    }
}

fn cluster_color(index: usize) -> Color {
    const COLORS: &[(f32, f32, f32)] = &[
        (0.95, 0.24, 0.32),
        (0.16, 0.62, 1.00),
        (0.20, 0.78, 0.38),
        (1.00, 0.74, 0.18),
        (0.70, 0.40, 1.00),
        (0.00, 0.78, 0.75),
        (1.00, 0.46, 0.20),
        (0.46, 0.58, 1.00),
        (0.68, 0.92, 0.16),
        (1.00, 0.34, 0.56),
    ];

    let (r, g, b) = COLORS[index % COLORS.len()];
    Color::srgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn node_scale_zero_uses_minimum() {
        assert_eq!(node_scale(0), MIN_NODE_SCALE);
    }

    #[test]
    fn node_scale_grows_monotonically_with_sqrt_compression() {
        assert!(node_scale(0) < node_scale(4));
        assert!(node_scale(4) < node_scale(64));
    }

    #[test]
    fn node_scale_clamps_to_supported_range() {
        assert_eq!(node_scale(10_000), MAX_NODE_SCALE);

        for child_count in [0, 1, 4, 64, 10_000, usize::MAX] {
            assert!((MIN_NODE_SCALE..=MAX_NODE_SCALE).contains(&node_scale(child_count)));
        }
    }

    #[test]
    fn seed_random_positions_sets_all_nodes_inside_range() {
        let mut nodes = vec![
            dummy_node(0, None, 0),
            dummy_node(1, Some(0), 1),
            dummy_node(2, Some(1), 1),
        ];
        let initial_len = nodes.len();

        seed_random_positions(&mut nodes);

        assert_eq!(nodes.len(), initial_len);
        assert!(nodes.iter().all(|node| node.pos != [0.0; 3]));

        for node in nodes {
            for axis in node.pos {
                assert!(
                    (-50.0..50.0).contains(&axis),
                    "axis value {axis} is outside [-50.0, 50.0)"
                );
            }
        }
    }

    fn dummy_node(id: usize, parent_id: Option<usize>, root_group: usize) -> FolderNode {
        FolderNode {
            id,
            path: PathBuf::from(format!("/tmp/node-{id}")),
            name: format!("node-{id}"),
            parent_id,
            root_group,
            depth: usize::from(parent_id.is_some()),
            child_count: 0,
            is_symlink: false,
            pos: [0.0; 3],
        }
    }
}
