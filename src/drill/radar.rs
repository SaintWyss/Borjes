use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::NavState;

pub fn draw_context_radar(
    mut contexts: EguiContexts,
    nav: Res<NavState>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) -> Result {
    let Ok(window) = windows.single() else {
        return Ok(());
    };
    if !window.visible || nav.tree.is_empty() {
        return Ok(());
    }

    let Some(root) = nav.tree.root() else {
        return Ok(());
    };
    let branches = nav.tree.children(root);
    if branches.is_empty() {
        return Ok(());
    }

    let active_group = nav
        .tree
        .node(nav.current)
        .map(|node| node.root_group)
        .unwrap_or(root);
    let depth = nav
        .tree
        .node(nav.current)
        .map(|node| node.depth)
        .unwrap_or_default();

    let ctx = contexts.ctx_mut()?;
    egui::Area::new(egui::Id::new("context_radar"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-24.0, -24.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(3, 14, 22, 210))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(35, 100, 112),
                ))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("MAPA")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(110, 180, 190)),
                    );

                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(128.0, 92.0), egui::Sense::hover());
                    let painter = ui.painter_at(rect);
                    let center = rect.center();
                    painter.circle_filled(center, 3.0, egui::Color32::from_rgb(130, 245, 245));

                    let radius = 34.0;
                    for (index, branch) in branches.iter().copied().enumerate() {
                        let angle = index as f32 * std::f32::consts::TAU / branches.len() as f32
                            - std::f32::consts::FRAC_PI_2;
                        let point = center + egui::vec2(angle.cos(), angle.sin()) * radius;
                        let active = branch == active_group;
                        let color = if active {
                            egui::Color32::from_rgb(255, 214, 90)
                        } else {
                            egui::Color32::from_rgb(45, 120, 132)
                        };
                        painter.line_segment(
                            [center, point],
                            egui::Stroke::new(if active { 1.8 } else { 0.8 }, color),
                        );
                        painter.circle_filled(point, if active { 5.0 } else { 3.0 }, color);
                    }

                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "nivel {depth} · {} ramas",
                                branches.len()
                            ))
                            .size(10.0)
                            .color(egui::Color32::from_rgb(125, 170, 178)),
                        );
                    });
                });
        });

    Ok(())
}
