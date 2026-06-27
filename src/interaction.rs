use std::time::{Duration, Instant};

use bevy::picking::prelude::{Click, Out, Over, Pointer};
use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::render::{GraphData, NodeRef};

#[derive(Resource)]
pub struct Selected(pub Option<usize>);

#[derive(Resource)]
pub struct HighlightMaterial(pub Handle<StandardMaterial>);

#[derive(Component)]
pub struct OriginalMaterial(pub Handle<StandardMaterial>);

pub fn on_node_over(
    trigger: On<Pointer<Over>>,
    highlight: Res<HighlightMaterial>,
    mut nodes: Query<&mut MeshMaterial3d<StandardMaterial>, With<NodeRef>>,
) {
    let Ok(mut material) = nodes.get_mut(trigger.entity) else {
        return;
    };

    material.0 = highlight.0.clone();
}

pub fn on_node_out(
    trigger: On<Pointer<Out>>,
    highlight: Res<HighlightMaterial>,
    mut nodes: Query<
        (
            &OriginalMaterial,
            &mut MeshMaterial3d<StandardMaterial>,
            Option<&crate::ui::SearchMatch>,
        ),
        With<NodeRef>,
    >,
) {
    let Ok((original, mut material, search_match)) = nodes.get_mut(trigger.entity) else {
        return;
    };

    material.0 = if search_match.is_some() {
        highlight.0.clone()
    } else {
        original.0.clone()
    };
}

pub fn on_node_click(
    trigger: On<Pointer<Click>>,
    mut contexts: EguiContexts,
    mut selected: ResMut<Selected>,
    graph: Res<GraphData>,
    nodes: Query<&NodeRef>,
    mut last_click: Local<Option<(Instant, Entity)>>,
) -> Result {
    if contexts.ctx_mut()?.is_pointer_over_egui() {
        return Ok(());
    }

    let entity = trigger.entity;
    let Ok(node_ref) = nodes.get(entity) else {
        return Ok(());
    };

    let Some(node) = graph.0.get(node_ref.0) else {
        return Ok(());
    };

    selected.0 = Some(node_ref.0);
    info!("selected: {}", node.path.display());

    let is_double_click = last_click.as_ref().is_some_and(|(instant, previous)| {
        *previous == entity && instant.elapsed() < Duration::from_millis(300)
    });

    if is_double_click {
        let path = node.path.clone();
        std::thread::spawn(move || {
            let _ = open::that(path);
        });
        *last_click = None;
    } else {
        *last_click = Some((Instant::now(), entity));
    }

    Ok(())
}
