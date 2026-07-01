use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts};

use crate::drill::NavState;
use crate::frecency::{self, Frecency, FrecencyEntry};
use crate::nav::FolderTree;
use crate::pins::PinStore;
use crate::root::{expand_tilde, RootConfig};
use crate::ui::normalize;

#[derive(Resource, Default)]
pub struct BorjesSearch {
    pub query: String,
    pub selected: usize,
    pub results: Vec<usize>,
    prev_visible: bool,
    focus_requested: bool,
    pub ascend_requested: bool,
}

pub fn score(name: &str, query: &str) -> u32 {
    let name = normalize(name);
    let query = normalize(query);

    if query.is_empty() {
        return 0;
    }
    if name.starts_with(&query) {
        return 3;
    }
    if name
        .match_indices(&query)
        .any(|(idx, _)| idx > 0 && matches!(name.as_bytes()[idx - 1], b' ' | b'_' | b'-' | b'.'))
    {
        return 2;
    }
    if name.contains(&query) {
        return 1;
    }

    0
}

pub fn rank(
    tree: &FolderTree,
    query: &str,
    cap: usize,
    frec: &HashMap<PathBuf, FrecencyEntry>,
    now: u64,
) -> Vec<usize> {
    let mut scored: Vec<(usize, u32, f64, usize, usize, String)> = (0..tree.len())
        .filter_map(|id| {
            let node = tree.node(id)?;
            let node_score = score(&node.name, query);
            (node_score > 0).then(|| {
                let frecency_score = frec
                    .get(&node.path)
                    .map(|entry| frecency::decayed(entry, now))
                    .unwrap_or(0.0);
                (
                    id,
                    node_score,
                    frecency_score,
                    node.depth,
                    node.name.len(),
                    node.name.clone(),
                )
            })
        })
        .collect();

    scored.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| a.3.cmp(&b.3))
            .then_with(|| a.4.cmp(&b.4))
            .then_with(|| a.5.cmp(&b.5))
    });
    scored.truncate(cap);
    scored.into_iter().map(|(id, _, _, _, _, _)| id).collect()
}

pub fn short_path(path: &Path, root: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let parts: Vec<String> = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();

    if parts.len() > 3 {
        format!("…/{}", parts[parts.len() - 3..].join("/"))
    } else {
        parts.join("/")
    }
}

pub fn search_ui(
    mut contexts: EguiContexts,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut search: ResMut<BorjesSearch>,
    mut nav: ResMut<NavState>,
    mut frecency: ResMut<Frecency>,
    root: Res<RootConfig>,
) -> Result {
    let Ok(mut window) = windows.single_mut() else {
        return Ok(());
    };

    if !window.visible {
        search.query.clear();
        search.results.clear();
        search.selected = 0;
        search.prev_visible = false;
        return Ok(());
    }

    if !search.prev_visible {
        search.focus_requested = true;
    }

    let root_path = expand_tilde(&root.path);
    let ctx = contexts.ctx_mut()?;
    let mut changed = false;
    let mut chosen = None;
    let mut open_current = false;

    egui::Area::new(egui::Id::new("borjes_search"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 40.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(4, 18, 24, 220))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_premultiplied(80, 230, 220, 160),
                ))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.set_width(420.0);
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut search.query)
                            .desired_width(420.0)
                            .hint_text("Buscar carpeta…")
                            .text_color(egui::Color32::from_rgb(80, 230, 220)),
                    );

                    if search.focus_requested {
                        response.request_focus();
                        search.focus_requested = false;
                    }
                    changed = response.changed();

                    if !search.query.is_empty() {
                        ui.add_space(8.0);
                        for (row, node_id) in search.results.iter().copied().enumerate() {
                            let Some(node) = nav.tree.node(node_id) else {
                                continue;
                            };
                            let selected = row == search.selected;
                            let row_frame = egui::Frame::new()
                                .fill(if selected {
                                    egui::Color32::from_rgba_premultiplied(40, 150, 155, 90)
                                } else {
                                    egui::Color32::TRANSPARENT
                                })
                                .stroke(if selected {
                                    egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 230, 220))
                                } else {
                                    egui::Stroke::NONE
                                })
                                .corner_radius(egui::CornerRadius::same(6))
                                .inner_margin(egui::Margin::symmetric(8, 6));

                            let response = row_frame
                                .show(ui, |ui| {
                                    ui.set_width(396.0);
                                    ui.label(
                                        egui::RichText::new(&node.name)
                                            .strong()
                                            .color(egui::Color32::from_rgb(234, 250, 255)),
                                    );
                                    ui.label(
                                        egui::RichText::new(short_path(&node.path, &root_path))
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(140, 165, 175)),
                                    );
                                })
                                .response;

                            if response.clicked() {
                                chosen = Some(node_id);
                            }
                        }
                    }
                });
        });

    if changed {
        search.results = rank(
            &nav.tree,
            &search.query,
            8,
            &frecency.0,
            frecency::now_secs(),
        );
        search.selected = 0;
    }

    let result_count = search.results.len();
    let mut teleport = None;
    ctx.input(|input| {
        if input.key_pressed(egui::Key::ArrowDown) && result_count > 0 {
            search.selected = (search.selected + 1).min(result_count - 1);
        }
        if input.key_pressed(egui::Key::ArrowUp) && result_count > 0 {
            search.selected = search.selected.saturating_sub(1);
        }
        if input.key_pressed(egui::Key::Enter) {
            if search.query.is_empty() {
                open_current = true;
            } else {
                chosen = search.results.get(search.selected).copied();
            }
        }
        // Tab = teletransporte: navegar el grafo hasta el resultado sin abrirlo.
        if input.key_pressed(egui::Key::Tab) {
            teleport = search.results.get(search.selected).copied();
        }
        // `!changed`: si este mismo Backspace borró el último char, la query quedó
        // vacía recién ahora — no es un pedido de subir al padre.
        if input.key_pressed(egui::Key::Backspace) && search.query.is_empty() && !changed {
            search.ascend_requested = true;
        }
        if input.key_pressed(egui::Key::Escape) && !search.query.is_empty() {
            search.query.clear();
            search.results.clear();
            search.selected = 0;
        }
    });

    if let Some(id) = teleport {
        if nav.tree.node(id).is_some() {
            nav.current = id;
            nav.dirty = true;
            search.query.clear();
            search.results.clear();
            search.selected = 0;
            // Mantener el foco en el buscador para seguir tipeando desde el nuevo nivel.
            search.focus_requested = true;
        }
    }

    if let Some(path) = chosen.and_then(|id| nav.tree.node(id).map(|node| node.path.clone())) {
        frecency::bump(&mut frecency.0, &path, frecency::now_secs());
        frecency::save_frecency(&frecency.0);
        std::thread::spawn(move || {
            let _ = open::that(path);
        });
        window.visible = false;
        search.query.clear();
        search.results.clear();
        search.selected = 0;
    }

    if open_current {
        if let Some(path) = nav.tree.node(nav.current).map(|node| node.path.clone()) {
            frecency::bump(&mut frecency.0, &path, frecency::now_secs());
            frecency::save_frecency(&frecency.0);
            std::thread::spawn(move || {
                let _ = open::that(path);
            });
            window.visible = false;
            search.query.clear();
            search.results.clear();
            search.selected = 0;
        }
    }

    search.prev_visible = true;
    Ok(())
}

pub fn pin_chips(
    mut contexts: EguiContexts,
    windows: Query<&Window, With<PrimaryWindow>>,
    pin_store: Res<PinStore>,
    mut nav: ResMut<NavState>,
    mut search: ResMut<BorjesSearch>,
) -> Result {
    let Ok(window) = windows.single() else {
        return Ok(());
    };
    if !window.visible {
        return Ok(());
    }

    let mut pins: Vec<PathBuf> = pin_store.0.iter().cloned().collect();
    pins.sort_by(|a, b| {
        file_label(a)
            .to_lowercase()
            .cmp(&file_label(b).to_lowercase())
            .then_with(|| a.cmp(b))
    });
    pins.truncate(9);

    if pins.is_empty() {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    let mut teleport = None;

    ctx.input(|input| {
        if input.modifiers.alt {
            for (idx, path) in pins.iter().enumerate() {
                if alt_number_pressed(input, idx) {
                    teleport = Some(path.clone());
                    break;
                }
            }
        }
    });

    egui::Area::new(egui::Id::new("pin_chips"))
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -40.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (idx, path) in pins.iter().enumerate() {
                    let label = format!("{} · {}", idx + 1, file_label(path));
                    let button = egui::Button::new(
                        egui::RichText::new(label).color(egui::Color32::from_rgb(255, 214, 90)),
                    )
                    .fill(egui::Color32::from_rgba_premultiplied(4, 18, 24, 220))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(255, 214, 90),
                    ))
                    .corner_radius(egui::CornerRadius::same(8));

                    if ui.add(button).clicked() {
                        teleport = Some(path.clone());
                    }
                }
            });
        });

    if let Some(path) = teleport {
        if let Some(id) = nav.tree.find_by_path(&path) {
            nav.current = id;
            nav.dirty = true;
            search.query.clear();
            search.results.clear();
            search.selected = 0;
            search.focus_requested = true;
        }
    }

    Ok(())
}

fn file_label(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

fn alt_number_pressed(input: &egui::InputState, idx: usize) -> bool {
    match idx {
        0 => input.key_pressed(egui::Key::Num1),
        1 => input.key_pressed(egui::Key::Num2),
        2 => input.key_pressed(egui::Key::Num3),
        3 => input.key_pressed(egui::Key::Num4),
        4 => input.key_pressed(egui::Key::Num5),
        5 => input.key_pressed(egui::Key::Num6),
        6 => input.key_pressed(egui::Key::Num7),
        7 => input.key_pressed(egui::Key::Num8),
        8 => input.key_pressed(egui::Key::Num9),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_prioritizes_prefix_boundary_and_substring() {
        assert_eq!(score("Código", "codigo"), 3);
        assert_eq!(score("mi-carpeta", "car"), 2);
        assert_eq!(score("archivo", "chi"), 1);
        assert_eq!(score("archivo", "zzz"), 0);
        assert_eq!(score("archivo", ""), 0);
    }

    #[test]
    fn short_path_keeps_three_or_fewer_components() {
        assert_eq!(
            short_path(Path::new("/home/me/Data/a/b/c"), Path::new("/home/me/Data")),
            "a/b/c"
        );
    }

    #[test]
    fn short_path_collapses_deep_paths_to_last_three_components() {
        assert_eq!(
            short_path(
                Path::new("/home/me/Data/a/b/c/d/e"),
                Path::new("/home/me/Data")
            ),
            "…/c/d/e"
        );
    }
}
