mod render;
mod scan;

use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use std::path::PathBuf;

fn main() {
    let home = std::env::var("HOME").expect("HOME not set");
    let data_path = PathBuf::from(home).join("Data");
    let mut nodes = scan::scan_folder(&data_path);
    render::seed_random_positions(&mut nodes);

    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin, PanOrbitCameraPlugin))
        .insert_resource(render::GraphData(nodes))
        .add_systems(Startup, (setup_camera, render::spawn_graph))
        .add_systems(Update, render::draw_edges)
        .run();
}

fn setup_camera(mut commands: Commands) {
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
