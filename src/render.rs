use std::collections::HashMap;

use bevy::prelude::*;
use rand::Rng;

use crate::interaction::OriginalMaterial;
use crate::scan::FolderNode;

#[derive(Resource)]
pub struct GraphData(pub Vec<FolderNode>);

#[derive(Component)]
pub struct NodeRef(pub usize);

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
    let mut cluster_materials: HashMap<usize, Handle<StandardMaterial>> = HashMap::new();

    for node in &graph.0 {
        let material = match cluster_materials.get(&node.root_group) {
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
        };

        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(material.clone()),
            OriginalMaterial(material),
            Transform::from_translation(Vec3::from_array(node.pos)),
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
        (0.90, 0.22, 0.27),
        (0.12, 0.56, 1.00),
        (0.24, 0.72, 0.36),
        (1.00, 0.72, 0.20),
        (0.78, 0.34, 0.92),
        (0.00, 0.78, 0.75),
        (1.00, 0.42, 0.24),
        (0.58, 0.72, 1.00),
        (0.72, 0.90, 0.18),
        (1.00, 0.36, 0.68),
    ];

    let (r, g, b) = COLORS[index % COLORS.len()];
    Color::srgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

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
