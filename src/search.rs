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
    pub focus_requested: bool,
    pub ascend_requested: bool,
}

// search_ui usa el camino indexado (score_normalized + all_matches); score/rank
// quedan como API de lib y para los tests.
#[allow(dead_code)]
pub fn score(name: &str, query: &str) -> u32 {
    score_normalized(&normalize(name), &normalize(query))
}

pub fn score_normalized(norm_name: &str, norm_query: &str) -> u32 {
    if norm_query.is_empty() {
        return 0;
    }
    if norm_name.starts_with(norm_query) {
        return 3;
    }
    if norm_name.match_indices(norm_query).any(|(idx, _)| {
        idx > 0 && matches!(norm_name.as_bytes()[idx - 1], b' ' | b'_' | b'-' | b'.')
    }) {
        return 2;
    }
    if norm_name.contains(norm_query) {
        return 1;
    }

    0
}

#[allow(dead_code)]
pub fn rank(
    tree: &FolderTree,
    query: &str,
    cap: usize,
    frec: &HashMap<PathBuf, FrecencyEntry>,
    now: u64,
) -> Vec<usize> {
    let norm_query = normalize(query);
    let mut scored: Vec<(usize, u32, f64, usize, usize, String)> = (0..tree.len())
        .filter_map(|id| {
            let node = tree.node(id)?;
            let norm_name = tree.normalized(id)?;
            let node_score = score_normalized(norm_name, &norm_query);
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

pub fn all_matches(tree: &FolderTree, query: &str, cap: usize) -> Vec<usize> {
    let norm_query = normalize(query);
    let mut scored: Vec<(usize, u32, usize, String)> = (0..tree.len())
        .filter_map(|id| {
            let node = tree.node(id)?;
            let norm_name = tree.normalized(id)?;
            let node_score = score_normalized(norm_name, &norm_query);
            (node_score > 0).then(|| (id, node_score, node.depth, node.name.clone()))
        })
        .collect();

    scored.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| a.2.cmp(&b.2))
            .then_with(|| a.3.cmp(&b.3))
    });
    scored.truncate(cap);
    scored.into_iter().map(|(id, _, _, _)| id).collect()
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
    nav: ResMut<NavState>,
    mut frecency: ResMut<Frecency>,
    mut constellation: ResMut<crate::constellation::Constellation>,
    lenses: Res<crate::lens::Lenses>,
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
        constellation.clear();
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
    let mut lens_clicked: Option<usize> = None;

    egui::Area::new(egui::Id::new("borjes_search"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 40.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(4, 18, 24, 220))
                .stroke(egui::Stroke::new(
                    1.0_f32,
                    egui::Color32::from_rgba_premultiplied(80, 230, 220, 160),
                ))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.set_width(420.0);
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut search.query)
                            .desired_width(420.0)
                            .hint_text("Buscar carpeta…  (@lente)")
                            .text_color(egui::Color32::from_rgb(80, 230, 220)),
                    );

                    if search.focus_requested {
                        response.request_focus();
                        search.focus_requested = false;
                    }
                    changed = response.changed();

                    // Chips de lentes: un click monta el tema entero.
                    if !lenses.all.is_empty() {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            for (idx, l) in lenses.all.iter().enumerate() {
                                let (r, g, b) =
                                    crate::lens::parse_color(&l.color).unwrap_or((140, 250, 242));
                                let color = egui::Color32::from_rgb(r, g, b);
                                let chip = egui::Button::new(
                                    egui::RichText::new(format!("◉ {}", l.name))
                                        .size(11.5)
                                        .color(color),
                                )
                                .fill(egui::Color32::from_rgba_premultiplied(4, 18, 24, 200))
                                .stroke(egui::Stroke::new(1.0_f32, color))
                                .corner_radius(egui::CornerRadius::same(10));
                                if ui.add(chip).clicked() {
                                    lens_clicked = Some(idx);
                                }
                            }
                        });
                    }

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
                                    egui::Stroke::new(
                                        1.0_f32,
                                        egui::Color32::from_rgb(80, 230, 220),
                                    )
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

    // Chip clickeado = como haber tipeado "@Lente" (mismo camino, query visible).
    if let Some(idx) = lens_clicked {
        if let Some(l) = lenses.all.get(idx) {
            search.query = format!("@{}", l.name);
            changed = true;
            search.focus_requested = true;
        }
    }

    if changed {
        if search.query.trim().is_empty() {
            search.results.clear();
            search.selected = 0;
            constellation.clear();
        } else if let Some(prefix) = search.query.strip_prefix('@') {
            // "@lente": monta la lente como constelación temática en vivo.
            match crate::lens::find_lens(&lenses.all, prefix) {
                Some(l) => {
                    let stars = crate::lens::lens_matches(&nav.tree, l);
                    let color = crate::lens::parse_color(&l.color).unwrap_or((140, 250, 242));
                    search.results = stars.iter().take(8).copied().collect();
                    search.selected = 0;
                    constellation.set_lens(stars, l.name.clone(), color);
                }
                None => {
                    search.results.clear();
                    search.selected = 0;
                    constellation.clear();
                }
            }
        } else {
            // Fuente única: TODOS los matches del árbol. Frecuencia desempata
            // dentro de cada score (sort estable preserva depth/name del índice).
            let mut stars = all_matches(&nav.tree, &search.query, 64);
            let now = frecency::now_secs();
            let norm_query = normalize(&search.query);
            let star_score = |id: usize| {
                nav.tree
                    .normalized(id)
                    .map(|name| score_normalized(name, &norm_query))
                    .unwrap_or(0)
            };
            let star_frec = |id: usize| {
                nav.tree
                    .node(id)
                    .and_then(|node| frecency.0.get(&node.path))
                    .map(|entry| frecency::decayed(entry, now))
                    .unwrap_or(0.0)
            };
            stars.sort_by(|&a, &b| {
                star_score(b).cmp(&star_score(a)).then_with(|| {
                    star_frec(b)
                        .partial_cmp(&star_frec(a))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            });
            search.results = stars.iter().take(8).copied().collect();
            search.selected = 0;
            constellation.set(stars);
        }
    }

    let result_count = search.results.len();
    ctx.input(|input| {
        // Tab y flechas ciclan el foco de la constelación (la lista lo espeja).
        let step = |constellation: &mut crate::constellation::Constellation,
                    search: &mut BorjesSearch,
                    dir: isize| {
            constellation.cycle(dir);
            search.selected = constellation.focus.min(result_count.saturating_sub(1));
        };
        if input.key_pressed(egui::Key::Tab) && constellation.active {
            let dir = if input.modifiers.shift { -1 } else { 1 };
            step(&mut constellation, &mut search, dir);
        }
        if input.key_pressed(egui::Key::ArrowDown) && constellation.active {
            step(&mut constellation, &mut search, 1);
        }
        if input.key_pressed(egui::Key::ArrowUp) && constellation.active {
            step(&mut constellation, &mut search, -1);
        }
        if input.key_pressed(egui::Key::Enter) {
            if search.query.is_empty() {
                open_current = true;
            } else {
                chosen = constellation
                    .focused()
                    .or_else(|| search.results.get(search.selected).copied());
            }
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
            constellation.clear();
        }
    });

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
        constellation.clear();
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
                        1.0_f32,
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
            nav.navigate(id);
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
    use crate::scan::FolderNode;

    fn folder_node(id: usize, name: &str, parent_id: Option<usize>, depth: usize) -> FolderNode {
        FolderNode {
            id,
            path: PathBuf::from(format!("/data/{name}")),
            name: name.to_string(),
            parent_id,
            root_group: 0,
            depth,
            child_count: 0,
            is_symlink: false,
            pos: [0.0; 3],
        }
    }

    fn search_tree() -> FolderTree {
        FolderTree::from_nodes(vec![
            folder_node(0, "data", None, 0),
            folder_node(1, "project", Some(0), 1),
            folder_node(2, "other", Some(0), 1),
            folder_node(3, "alpha project", Some(1), 2),
            folder_node(4, "my-project", Some(2), 1),
            folder_node(5, "deep project", Some(2), 3),
        ])
    }

    #[test]
    fn score_prioritizes_prefix_boundary_and_substring() {
        assert_eq!(score("Código", "codigo"), 3);
        assert_eq!(score("mi-carpeta", "car"), 2);
        assert_eq!(score("archivo", "chi"), 1);
        assert_eq!(score("archivo", "zzz"), 0);
        assert_eq!(score("archivo", ""), 0);
    }

    #[test]
    fn all_matches_returns_matches_sorted_without_frecency() {
        let tree = search_tree();

        assert_eq!(all_matches(&tree, "pro", 10), vec![1, 4, 3, 5]);
    }

    #[test]
    fn all_matches_respects_cap() {
        let tree = search_tree();

        assert_eq!(all_matches(&tree, "pro", 2), vec![1, 4]);
    }

    #[test]
    fn rank_uses_normalized_index_and_keeps_existing_ordering() {
        let tree = search_tree();

        assert_eq!(rank(&tree, "pro", 10, &HashMap::new(), 0), vec![1, 4, 3, 5]);
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
