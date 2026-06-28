// v2 search panel + pin menu live here; v3 drill-down unregisters them and they
// are rewired in F13. Only animate_camera_to_target stays live for now.
#![allow(dead_code)]

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_panorbit_camera::PanOrbitCamera;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

use crate::{
    interaction::{self, HighlightMaterial, OriginalMaterial, Selected},
    pins,
    render::{self, GraphData, NodeRef},
    root::{expand_tilde, PendingRescan, RootConfig},
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
    mut root: ResMut<RootConfig>,
    mut pending_rescan: ResMut<PendingRescan>,
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
    let mut rescan_clicked = false;
    let displayed = selected.0.or(search.first_match);

    // ponytail: egui 0.34 marca Panel::show deprecated por churn de API, pero show(ctx)
    // sigue siendo la forma correcta para un panel top-level sobre el Context.
    #[allow(deprecated)]
    egui::TopBottomPanel::top("search").show(contexts.ctx_mut()?, |ui| {
        ui.horizontal(|ui| {
            ui.label("Raíz");
            ui.text_edit_singleline(&mut root.path);
            rescan_clicked = ui.button("Reescanear").clicked();
        });

        if let Some(error) = &root.error {
            ui.colored_label(egui::Color32::from_rgb(255, 96, 96), error);
        }

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Buscar");
            let response = ui.text_edit_singleline(&mut search.query);
            changed = response.changed();
        });

        if let Some(id) = displayed.and_then(|id| graph.0.get(id)) {
            ui.label(id.path.display().to_string());
        }
    });

    if rescan_clicked {
        let root_path = expand_tilde(&root.path);
        if root_path.is_dir() {
            root.error = None;
            pending_rescan.0 = Some(root_path);
        } else {
            root.error = Some(format!(
                "La ruta no existe o no es un directorio: {}",
                root_path.display()
            ));
            pending_rescan.0 = None;
        }
    }

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

pub fn pin_menu(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut pin_menu: ResMut<interaction::PinMenu>,
    mut pin_store: ResMut<pins::PinStore>,
    pin_assets: Res<render::PinAssets>,
    originals: Query<&interaction::OriginalMaterial>,
    pinned_q: Query<&interaction::Pinned>,
) -> Result {
    if !pin_menu.open {
        return Ok(());
    }

    let Some(path) = pin_menu.path.take() else {
        pin_menu.open = false;
        return Ok(());
    };
    let Some(entity) = pin_menu.entity.take() else {
        pin_menu.open = false;
        return Ok(());
    };

    let is_pinned = pin_store.0.contains(&path);
    let pos = pin_menu.pos;
    let ctx = contexts.ctx_mut()?;
    let style = ctx.global_style();
    let mut clicked = false;

    let was_just_opened = std::mem::take(&mut pin_menu.just_opened);
    let area_response = egui::Area::new(egui::Id::new("pin_menu"))
        .fixed_pos(egui::pos2(pos.x, pos.y))
        .show(ctx, |ui| {
            egui::Frame::menu(&style).show(ui, |ui| {
                let label = if is_pinned { "Unpin" } else { "Pin" };
                clicked = ui.button(label).clicked();
            });
        });
    // Don't let the opening right-click dismiss the menu on its first frame.
    let dismissed = !was_just_opened && area_response.response.clicked_elsewhere();

    if clicked {
        if is_pinned {
            pin_store.0.remove(&path);
            if let Ok(pinned) = pinned_q.get(entity) {
                interaction::clear_pin_visual(&mut commands, entity, pinned);
            }
        } else {
            pin_store.0.insert(path.clone());
            if let Ok(original) = originals.get(entity) {
                interaction::apply_pin_visual(
                    &mut commands,
                    entity,
                    &original.0,
                    &pin_assets.material,
                );
            }
        }

        pins::save_pins(&pin_store.0);
        pin_menu.open = false;
    } else if dismissed {
        pin_menu.open = false;
    } else {
        pin_menu.path = Some(path);
        pin_menu.entity = Some(entity);
    }

    Ok(())
}

pub fn animate_camera_to_target(
    mut target: ResMut<SearchTarget>,
    mut cameras: Query<&mut PanOrbitCamera>,
) {
    let Some(position) = target.0 else {
        return;
    };

    // Acercar de verdad: además de recentrar el foco, bajar el radio de órbita
    // (zoom in). Sin esto la cámara recentra pero queda lejos ("vuela apenitas").
    const SEARCH_ZOOM_RADIUS: f32 = 10.0;

    let mut reached = false;
    for mut camera in &mut cameras {
        camera.target_focus = position;
        camera.target_radius = SEARCH_ZOOM_RADIUS;
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
