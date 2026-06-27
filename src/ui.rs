use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_panorbit_camera::PanOrbitCamera;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

use crate::{
    interaction::{HighlightMaterial, OriginalMaterial, Selected},
    render::{GraphData, NodeRef},
};

#[derive(Resource, Default)]
pub struct SearchState {
    pub query: String,
    pub first_match: Option<usize>,
}

#[derive(Resource, Default)]
pub struct SearchTarget(pub Option<Vec3>);

#[derive(Component)]
pub struct SearchMatch;

pub fn normalize(s: &str) -> String {
    s.trim()
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(|c| c.to_lowercase())
        .collect()
}

pub fn matches(name: &str, query: &str) -> bool {
    if query.trim().is_empty() {
        return false;
    }

    normalize(name).contains(&normalize(query))
}

#[allow(clippy::too_many_arguments)] // sistema Bevy: muchos system-params es idiomático
pub fn search_panel(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut search: ResMut<SearchState>,
    graph: Res<GraphData>,
    selected: Res<Selected>,
    mut target: ResMut<SearchTarget>,
    highlight: Res<HighlightMaterial>,
    mut nodes: Query<(
        Entity,
        &NodeRef,
        &OriginalMaterial,
        &mut MeshMaterial3d<StandardMaterial>,
        Option<&SearchMatch>,
    )>,
) -> Result {
    let mut changed = false;
    let displayed = selected.0.or(search.first_match);

    // ponytail: egui 0.34 marca Panel::show deprecated por churn de API, pero show(ctx)
    // sigue siendo la forma correcta para un panel top-level sobre el Context.
    #[allow(deprecated)]
    egui::TopBottomPanel::top("search").show(contexts.ctx_mut()?, |ui| {
        ui.horizontal(|ui| {
            ui.label("Buscar");
            let response = ui.text_edit_singleline(&mut search.query);
            changed = response.changed();
        });

        if let Some(id) = displayed.and_then(|id| graph.0.get(id)) {
            ui.label(id.path.display().to_string());
        }
    });

    if !changed {
        return Ok(());
    }

    let query = search.query.clone();
    let mut first_match: Option<(usize, Vec3)> = None;

    for (entity, node_ref, original, mut material, active_match) in &mut nodes {
        let Some(node) = graph.0.get(node_ref.0) else {
            continue;
        };

        if matches(&node.name, &query) {
            material.0 = highlight.0.clone();
            if active_match.is_none() {
                commands.entity(entity).insert(SearchMatch);
            }

            let position = Vec3::from_array(node.pos);
            if first_match.is_none_or(|(id, _)| node_ref.0 < id) {
                first_match = Some((node_ref.0, position));
            }
        } else {
            material.0 = original.0.clone();
            if active_match.is_some() {
                commands.entity(entity).remove::<SearchMatch>();
            }
        }
    }

    search.first_match = first_match.map(|(id, _)| id);
    target.0 = first_match.map(|(_, position)| position);

    Ok(())
}

pub fn animate_camera_to_target(
    mut target: ResMut<SearchTarget>,
    mut cameras: Query<&mut PanOrbitCamera>,
) {
    let Some(position) = target.0 else {
        return;
    };

    let mut reached = false;
    for mut camera in &mut cameras {
        camera.target_focus = position;
        reached |= camera.focus.distance(position) < 0.5;
    }

    if reached {
        target.0 = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_lowercases_and_removes_accents() {
        assert_eq!(normalize("AdSI"), "adsi");
        assert_eq!(normalize("María"), "maria");
        assert_eq!(normalize("  Espacios "), "espacios");
    }

    #[test]
    fn matches_is_accent_insensitive() {
        assert!(matches("AdSI", "adsi"));
        assert!(matches("María", "maria"));
        assert!(matches("Camión", "camion"));
    }

    #[test]
    fn empty_query_does_not_match_everything() {
        assert!(!matches("AdSI", ""));
        assert!(!matches("AdSI", "   "));
    }
}
