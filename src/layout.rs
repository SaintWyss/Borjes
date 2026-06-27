use std::collections::HashMap;

use bevy::math::Vec3;

use crate::scan::FolderNode;

const K: f32 = 3.0;
const CLUSTER_K: f32 = 0.05;
const ITERATIONS: usize = 150;
const INITIAL_TEMPERATURE: f32 = 4.0;
const MIN_TEMPERATURE: f32 = 0.05;
const EPSILON: f32 = 0.001;

pub fn run_layout(nodes: &mut [FolderNode], edges: &[(usize, usize)]) {
    if nodes.is_empty() {
        return;
    }

    // ponytail: FR O(N²) ~6.3B ops/pasada a 4607 nodos; si el startup se siente lento, migrar a Barnes-Hut octree O(N log N)
    let cluster_indices = cluster_indices(nodes);
    let group_lookup: HashMap<usize, usize> = cluster_indices
        .iter()
        .enumerate()
        .map(|(index, root_group)| (*root_group, index))
        .collect();

    let mut positions = seed_clustered_positions(nodes, &group_lookup, cluster_indices.len());
    let mut displacements = vec![Vec3::ZERO; nodes.len()];
    let mut centroids = vec![Vec3::ZERO; cluster_indices.len()];
    let mut counts = vec![0_usize; cluster_indices.len()];

    for iteration in 0..ITERATIONS {
        displacements.fill(Vec3::ZERO);

        apply_repulsion(&positions, &mut displacements);
        apply_attraction(&positions, &mut displacements, edges);
        apply_cluster_pull(
            nodes,
            &positions,
            &mut displacements,
            &group_lookup,
            &mut centroids,
            &mut counts,
        );

        let progress = iteration as f32 / (ITERATIONS - 1) as f32;
        let temperature = INITIAL_TEMPERATURE + (MIN_TEMPERATURE - INITIAL_TEMPERATURE) * progress;

        for (position, displacement) in positions.iter_mut().zip(&displacements) {
            let displacement = finite_vec3(*displacement);
            let length = displacement.length();

            if length > EPSILON {
                *position += displacement / length * length.min(temperature);
            }

            *position = finite_vec3(*position);
        }
    }

    for (node, position) in nodes.iter_mut().zip(positions) {
        let position = finite_vec3(position);
        node.pos = [position.x, position.y, position.z];
    }
}

fn cluster_indices(nodes: &[FolderNode]) -> Vec<usize> {
    let mut groups: Vec<usize> = nodes.iter().map(|node| node.root_group).collect();
    groups.sort_unstable();
    groups.dedup();
    groups
}

fn seed_clustered_positions(
    nodes: &[FolderNode],
    group_lookup: &HashMap<usize, usize>,
    cluster_count: usize,
) -> Vec<Vec3> {
    let radius = (cluster_count as f32 * 8.0).max(40.0);
    let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());

    nodes
        .iter()
        .map(|node| {
            let cluster_index = group_lookup
                .get(&node.root_group)
                .copied()
                .unwrap_or_default();
            let y = if cluster_count <= 1 {
                0.0
            } else {
                1.0 - (cluster_index as f32 / (cluster_count - 1) as f32) * 2.0
            };
            let ring_radius = (1.0 - y * y).sqrt();
            let theta = cluster_index as f32 * golden_angle;
            let center = Vec3::new(
                theta.cos() * ring_radius * radius,
                y * radius * 0.6,
                theta.sin() * ring_radius * radius,
            );

            center + deterministic_jitter(node.id)
        })
        .collect()
}

fn deterministic_jitter(id: usize) -> Vec3 {
    let n = id as f32 + 1.0;
    Vec3::new((n * 12.9898).sin(), (n * 78.233).sin(), (n * 37.719).cos()) * 1.5
}

fn apply_repulsion(positions: &[Vec3], displacements: &mut [Vec3]) {
    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let delta = positions[i] - positions[j];
            let (direction, distance) = direction_and_distance(delta);
            let force = K * K / distance;
            let displacement = direction * force;

            displacements[i] += displacement;
            displacements[j] -= displacement;
        }
    }
}

fn apply_attraction(positions: &[Vec3], displacements: &mut [Vec3], edges: &[(usize, usize)]) {
    for &(a, b) in edges {
        let (Some(position_a), Some(position_b)) = (positions.get(a), positions.get(b)) else {
            continue;
        };

        let delta = *position_a - *position_b;
        let (direction, distance) = direction_and_distance(delta);
        let force = distance * distance / K;
        let displacement = direction * force;

        displacements[a] -= displacement;
        displacements[b] += displacement;
    }
}

fn apply_cluster_pull(
    nodes: &[FolderNode],
    positions: &[Vec3],
    displacements: &mut [Vec3],
    group_lookup: &HashMap<usize, usize>,
    centroids: &mut [Vec3],
    counts: &mut [usize],
) {
    centroids.fill(Vec3::ZERO);
    counts.fill(0);

    for (node, position) in nodes.iter().zip(positions) {
        let Some(cluster_index) = group_lookup.get(&node.root_group).copied() else {
            continue;
        };

        centroids[cluster_index] += *position;
        counts[cluster_index] += 1;
    }

    for (centroid, count) in centroids.iter_mut().zip(counts.iter().copied()) {
        if count > 0 {
            *centroid /= count as f32;
        }
    }

    for ((node, position), displacement) in nodes.iter().zip(positions).zip(displacements) {
        let Some(cluster_index) = group_lookup.get(&node.root_group).copied() else {
            continue;
        };

        *displacement += (centroids[cluster_index] - *position) * CLUSTER_K;
    }
}

fn direction_and_distance(delta: Vec3) -> (Vec3, f32) {
    let distance_squared = delta.length_squared();

    if distance_squared > EPSILON * EPSILON && distance_squared.is_finite() {
        let distance = distance_squared.sqrt();
        (delta / distance, distance)
    } else {
        (Vec3::X, EPSILON)
    }
}

fn finite_vec3(value: Vec3) -> Vec3 {
    Vec3::new(
        finite_f32(value.x),
        finite_f32(value.y),
        finite_f32(value.z),
    )
}

fn finite_f32(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-10_000.0, 10_000.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn clusters_same_root_group_closer_than_other_clusters() {
        let mut nodes = synthetic_graph();
        let edges = synthetic_edges(&nodes);

        run_layout(&mut nodes, &edges);

        let intra = mean_distance(&nodes, |a, b| a.root_group == b.root_group);
        let inter = mean_distance(&nodes, |a, b| a.root_group != b.root_group);

        assert!(
            intra < inter,
            "expected intra-cluster distance {intra} to be less than inter-cluster distance {inter}"
        );
    }

    #[test]
    fn layout_does_not_emit_nan_or_infinite_positions() {
        let mut nodes = synthetic_graph();
        let edges = synthetic_edges(&nodes);

        run_layout(&mut nodes, &edges);

        assert!(nodes
            .iter()
            .flat_map(|node| node.pos)
            .all(|axis| axis.is_finite()));
    }

    #[test]
    fn layout_changes_positions_from_input() {
        let mut nodes = synthetic_graph();
        let edges = synthetic_edges(&nodes);
        let before: Vec<[f32; 3]> = nodes.iter().map(|node| node.pos).collect();

        run_layout(&mut nodes, &edges);

        assert!(nodes
            .iter()
            .zip(before)
            .any(|(node, original)| node.pos != original));
    }

    fn synthetic_graph() -> Vec<FolderNode> {
        let mut nodes = Vec::new();
        for root_group in [10, 20, 30] {
            let cluster_start = nodes.len();
            for offset in 0..6 {
                let id = cluster_start + offset;
                nodes.push(dummy_node(
                    id,
                    (offset > 0).then_some(cluster_start),
                    root_group,
                ));
            }
        }
        nodes
    }

    fn synthetic_edges(nodes: &[FolderNode]) -> Vec<(usize, usize)> {
        nodes
            .iter()
            .filter_map(|node| node.parent_id.map(|parent_id| (node.id, parent_id)))
            .collect()
    }

    fn mean_distance(
        nodes: &[FolderNode],
        include: impl Fn(&FolderNode, &FolderNode) -> bool,
    ) -> f32 {
        let mut total = 0.0;
        let mut count = 0_usize;

        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                if include(&nodes[i], &nodes[j]) {
                    total +=
                        Vec3::from_array(nodes[i].pos).distance(Vec3::from_array(nodes[j].pos));
                    count += 1;
                }
            }
        }

        total / count as f32
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
