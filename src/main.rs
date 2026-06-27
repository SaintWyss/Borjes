// Lints idiomáticos en Bevy (Query tuples + system-params). Allow a nivel crate.
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod interaction;
mod layout;
mod render;
mod scan;
mod ui;

use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::prelude::*;
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use std::path::PathBuf;

use crate::scan::FolderNode;

#[derive(Resource)]
struct LayoutTask(Task<Vec<FolderNode>>);

#[derive(Component)]
struct LayoutText;

fn main() {
    let home = std::env::var("HOME").expect("HOME not set");
    let data_path = PathBuf::from(home).join("Data");
    let nodes = scan::scan_folder(&data_path);

    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
            EguiPlugin::default(),
            PanOrbitCameraPlugin,
        ))
        .insert_resource(render::GraphData(nodes))
        .insert_resource(interaction::Selected(None))
        .insert_resource(ui::SearchState::default())
        .insert_resource(ui::SearchTarget::default())
        .add_systems(Startup, (setup_camera, start_layout))
        .add_systems(EguiPrimaryContextPass, ui::search_panel)
        .add_systems(
            Update,
            (
                apply_layout,
                render::draw_edges,
                ui::animate_camera_to_target,
            ),
        )
        .add_observer(interaction::on_node_over)
        .add_observer(interaction::on_node_out)
        .add_observer(interaction::on_node_click)
        .run();
}

fn setup_camera(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let highlight = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: Color::WHITE.to_linear(),
        unlit: true,
        ..default()
    });
    commands.insert_resource(interaction::HighlightMaterial(highlight));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 150.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(0.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn start_layout(mut commands: Commands, mut graph: ResMut<render::GraphData>) {
    let mut nodes = std::mem::take(&mut graph.0);
    let edges: Vec<(usize, usize)> = nodes
        .iter()
        .filter_map(|node| node.parent_id.map(|parent_id| (node.id, parent_id)))
        .collect();

    let pool = AsyncComputeTaskPool::get();
    let task = pool.spawn(async move {
        layout::run_layout(&mut nodes, &edges);
        nodes
    });

    commands.insert_resource(LayoutTask(task));
    commands.spawn((
        Text::new("Computing layout…"),
        TextFont::from_font_size(28.0),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            ..default()
        },
        LayoutText,
    ));
}

fn apply_layout(
    mut commands: Commands,
    task: Option<ResMut<LayoutTask>>,
    mut graph: ResMut<render::GraphData>,
    text_q: Query<Entity, With<LayoutText>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(mut task) = task else {
        return;
    };

    let Some(nodes) = block_on(poll_once(&mut task.0)) else {
        return;
    };

    graph.0 = nodes;
    render::spawn_graph_nodes(&mut commands, &mut meshes, &mut materials, &graph);

    for entity in &text_q {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<LayoutTask>();
}
