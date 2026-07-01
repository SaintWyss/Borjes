// Lints idiomáticos en Bevy (Query tuples + system-params). Allow a nivel crate.
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod desktop;
mod drill;
mod frecency;
mod icons;
mod interaction;
mod layout;
mod nav;
mod pins;
mod render;
mod root;
mod scan;
mod search;
mod ui;

use bevy::camera::Hdr;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use bevy::window::{MonitorSelection, Window, WindowMode, WindowPlugin};
use bevy::winit::{UpdateMode, WinitSettings};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use std::path::PathBuf;
use std::time::Duration;

use crate::desktop::{install_signal_handler, toggle_overlay_visibility};
use crate::root::{expand_tilde, PendingRescan, RootConfig};
use crate::scan::FolderNode;

#[derive(Resource)]
#[allow(dead_code)]
struct LayoutTask {
    task: Task<Vec<FolderNode>>,
    root_path: PathBuf,
}

#[derive(Component)]
#[allow(dead_code)]
struct LayoutText;

fn main() {
    // Install the SIGUSR1 handler BEFORE Bevy builds (window/GPU init takes
    // ~1-2s). Otherwise a USR1 arriving in that window hits the default
    // disposition (terminate) and the overlay dies — "se cierra y no abre".
    install_signal_handler();

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        decorations: false,
                        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                        visible: false,
                        window_level: bevy::window::WindowLevel::AlwaysOnTop,
                        ..default()
                    }),
                    ..default()
                })
                // Absolute asset root so icons load no matter the cwd (autostart
                // launches the binary from $HOME, not the repo).
                .set(bevy::asset::AssetPlugin {
                    file_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("assets")
                        .display()
                        .to_string(),
                    unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                    ..default()
                }),
            MeshPickingPlugin,
            EguiPlugin::default(),
            PanOrbitCameraPlugin,
        ))
        .insert_resource(WinitSettings {
            // Visible/focused → continuous so the dwell ring + glow animate even
            // with the cursor held still. Hidden/unfocused → low power (idle cheap).
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs(60)),
        })
        .insert_resource(render::GraphData(Vec::new()))
        .insert_resource(RootConfig::default())
        .insert_resource(PendingRescan::default())
        .insert_resource(interaction::Selected(None))
        .insert_resource(interaction::PinMenu::default())
        .insert_resource(pins::PinStore(pins::load_pins()))
        .insert_resource(frecency::Frecency(frecency::load_frecency()))
        .insert_resource(icons::load_icon_config())
        .insert_resource(ui::SearchState::default())
        .insert_resource(ui::SearchTarget::default())
        .insert_resource(search::BorjesSearch::default())
        .insert_resource(drill::HoverState::default())
        .insert_resource(drill::IconMenu::default())
        .insert_resource(drill::CamTarget { distance: 60.0 })
        .insert_resource(ClearColor(Color::srgb(0.01, 0.02, 0.04)))
        .add_systems(
            Startup,
            (
                setup_camera,
                render::setup_file_assets,
                render::setup_pin_assets,
                icons::load_icon_assets,
                init_nav,
            ),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                search::search_ui,
                search::pin_chips,
                drill::draw_breadcrumb,
                drill::draw_icon_menu,
                drill::draw_labels,
                drill::draw_hover_ring,
            ),
        )
        .add_systems(
            Update,
            (
                toggle_overlay_visibility,
                handle_rescan,
                drill::open_icon_menu,
                drill::ascend,
                drill::hover_track,
                drill::click_dive,
                drill::render_level,
                drill::animate_camera,
                drill::animate_nodes,
                drill::animate_connectors,
                ui::animate_camera_to_target,
            )
                .chain(),
        )
        .add_observer(interaction::on_node_over)
        .add_observer(interaction::on_node_out)
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

    // Perspective camera with HDR + Bloom so the emissive nodes glow like neurons.
    commands.spawn((
        Camera3d::default(),
        Hdr,
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        Transform::from_xyz(0.0, 0.0, 60.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(0.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn init_nav(mut commands: Commands, mut root: ResMut<RootConfig>) {
    let root_path = expand_tilde(&root.path);
    let nodes = scan::scan_folder(&root_path);
    let tree = nav::FolderTree::from_nodes(nodes);
    let current = tree.root().unwrap_or(0);

    root.error = if tree.is_empty() {
        Some(if root_path.is_dir() {
            format!("No se encontraron carpetas en {}", root_path.display())
        } else {
            format!(
                "La ruta no existe o no es un directorio: {}",
                root_path.display()
            )
        })
    } else {
        None
    };

    commands.insert_resource(drill::NavState {
        tree,
        current,
        dirty: true,
    });
}

#[allow(dead_code)]
fn start_layout(mut commands: Commands, root: Res<RootConfig>) {
    start_layout_task(&mut commands, expand_tilde(&root.path));
}

#[allow(dead_code)]
fn start_layout_task(commands: &mut Commands, root_path: PathBuf) {
    let task_root_path = root_path.clone();
    let pool = AsyncComputeTaskPool::get();
    let task = pool.spawn(async move {
        let mut nodes = scan::scan_folder(&task_root_path);
        let edges: Vec<(usize, usize)> = nodes
            .iter()
            .filter_map(|node| node.parent_id.map(|parent_id| (node.id, parent_id)))
            .collect();

        layout::run_layout(&mut nodes, &edges);
        nodes
    });

    commands.insert_resource(LayoutTask { task, root_path });
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

fn handle_rescan(
    mut commands: Commands,
    mut pending: ResMut<PendingRescan>,
    node_entities: Query<Entity, With<render::NodeRef>>,
    level_entities: Query<Entity, With<drill::LevelNode>>,
    file_entities: Query<Entity, With<interaction::FileNode>>,
    expanded_folders: Query<Entity, With<interaction::Expanded>>,
    mut nav: ResMut<drill::NavState>,
    mut selected: ResMut<interaction::Selected>,
    mut search: ResMut<ui::SearchState>,
    mut target: ResMut<ui::SearchTarget>,
    mut graph: ResMut<render::GraphData>,
    mut root: ResMut<RootConfig>,
) {
    let Some(root_path) = pending.0.take() else {
        return;
    };

    for entity in &node_entities {
        commands.entity(entity).despawn();
    }
    for entity in &level_entities {
        commands.entity(entity).despawn();
    }
    // Files are absolute siblings, not ChildOf children — despawn them explicitly
    // so no gray cube survives a reroot (07-03).
    for entity in &file_entities {
        commands.entity(entity).despawn();
    }
    for entity in &expanded_folders {
        commands.entity(entity).remove::<interaction::Expanded>();
    }

    selected.0 = None;
    *search = ui::SearchState::default();
    target.0 = None;
    graph.0.clear();
    root.path = root_path.display().to_string();
    let nodes = scan::scan_folder(&root_path);
    let tree = nav::FolderTree::from_nodes(nodes);
    let current = tree.root().unwrap_or(0);

    root.error = if tree.is_empty() {
        Some(if root_path.is_dir() {
            format!("No se encontraron carpetas en {}", root_path.display())
        } else {
            format!(
                "La ruta no existe o no es un directorio: {}",
                root_path.display()
            )
        })
    } else {
        None
    };

    *nav = drill::NavState {
        tree,
        current,
        dirty: true,
    };
}

#[allow(dead_code)]
fn apply_layout(
    mut commands: Commands,
    task: Option<ResMut<LayoutTask>>,
    mut graph: ResMut<render::GraphData>,
    mut root: ResMut<RootConfig>,
    pending: Res<PendingRescan>,
    text_q: Query<Entity, With<LayoutText>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(mut task) = task else {
        return;
    };

    let scanned_path = task.root_path.clone();
    let Some(nodes) = block_on(poll_once(&mut task.task)) else {
        return;
    };

    let is_stale_for_pending_rescan =
        pending.0.is_some() && expand_tilde(&root.path) != scanned_path;

    graph.0 = nodes;
    if is_stale_for_pending_rescan {
        graph.0.clear();
    } else {
        if graph.0.is_empty() && expand_tilde(&root.path) == scanned_path {
            root.error = Some(if scanned_path.is_dir() {
                format!("No se encontraron carpetas en {}", scanned_path.display())
            } else {
                format!(
                    "La ruta no existe o no es un directorio: {}",
                    scanned_path.display()
                )
            });
        } else if expand_tilde(&root.path) == scanned_path {
            root.error = None;
        }

        render::spawn_graph_nodes(&mut commands, &mut meshes, &mut materials, &graph);
    }

    for entity in &text_q {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<LayoutTask>();
}
