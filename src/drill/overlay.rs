use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::*;
use crate::icons::{self, IconConfig, VisualSpec, ICON_IDS};
use crate::pins::{self, PinStore};

pub fn open_icon_menu(
    mut contexts: EguiContexts,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    folders: Query<(&LevelFolder, &GlobalTransform)>,
    nav: Res<NavState>,
    icon_cfg: Res<IconConfig>,
    mut menu: ResMut<IconMenu>,
    mut hover: ResMut<HoverState>,
) -> Result {
    if !buttons.just_pressed(MouseButton::Right) || contexts.ctx_mut()?.is_pointer_over_egui() {
        return Ok(());
    }
    let Ok(window) = windows.single() else {
        return Ok(());
    };
    let Some(cursor) = window.cursor_position() else {
        return Ok(());
    };
    let Ok((camera, cam_tf)) = cameras.single() else {
        return Ok(());
    };
    let Some(node_id) = folder_under_cursor(cursor, camera, cam_tf, &folders) else {
        return Ok(());
    };
    let Some(node) = nav.tree.node(node_id) else {
        return Ok(());
    };

    let visual = icons::visual_for(&node.path, &node.name, &icon_cfg);
    menu.open = true;
    menu.node_id = Some(node_id);
    menu.name = node.name.clone();
    menu.path = node.path.clone();
    menu.pos = cursor;
    menu.emoji = visual.emoji.clone().unwrap_or_else(|| "📁".to_string());
    menu.png_asset = visual
        .asset
        .clone()
        .unwrap_or_else(|| "icons/folder.png".to_string());
    menu.model_asset = visual
        .asset
        .clone()
        .unwrap_or_else(|| "custom/model.glb".to_string());
    menu.scale = visual.sane_scale();
    menu.halo = visual.halo;
    menu.just_opened = true;
    hover.target = HoverTarget::None;

    Ok(())
}

pub fn draw_icon_menu(
    mut contexts: EguiContexts,
    mut icon_cfg: ResMut<IconConfig>,
    mut pin_store: ResMut<PinStore>,
    mut nav: ResMut<NavState>,
    mut menu: ResMut<IconMenu>,
) -> Result {
    if !menu.open {
        return Ok(());
    }

    let Some(node_id) = menu.node_id else {
        menu.open = false;
        return Ok(());
    };
    let Some(node) = nav.tree.node(node_id) else {
        menu.open = false;
        return Ok(());
    };
    let node_path = node.path.clone();
    let node_name = node.name.clone();

    let ctx = contexts.ctx_mut()?;
    let style = ctx.global_style();
    let pos = menu.pos;
    let mut close = false;
    let mut changed = false;

    let was_just_opened = std::mem::take(&mut menu.just_opened);
    let area_response = egui::Area::new(egui::Id::new("icon_menu"))
        .fixed_pos(egui::pos2(pos.x, pos.y))
        .show(ctx, |ui| {
            egui::Frame::menu(&style).show(ui, |ui| {
                ui.set_min_width(260.0);
                ui.label(egui::RichText::new(&menu.name).strong());
                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    for id in ICON_IDS {
                        if ui.small_button(*id).clicked() {
                            icons::set_override(&mut icon_cfg, &node_path, VisualSpec::builtin(id));
                            changed = true;
                            close = true;
                        }
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Emoji");
                    ui.text_edit_singleline(&mut menu.emoji);
                    if ui.button("Usar").clicked() {
                        icons::set_override(
                            &mut icon_cfg,
                            &node_path,
                            VisualSpec::emoji(menu.emoji.clone(), menu.scale, menu.halo),
                        );
                        changed = true;
                        close = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("PNG");
                    ui.text_edit_singleline(&mut menu.png_asset);
                    if ui.button("Usar").clicked() {
                        icons::set_override(
                            &mut icon_cfg,
                            &node_path,
                            VisualSpec::png(menu.png_asset.clone(), menu.scale, menu.halo),
                        );
                        changed = true;
                        close = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("GLB");
                    ui.text_edit_singleline(&mut menu.model_asset);
                    if ui.button("Usar").clicked() {
                        icons::set_override(
                            &mut icon_cfg,
                            &node_path,
                            VisualSpec::model(menu.model_asset.clone(), menu.scale, menu.halo),
                        );
                        changed = true;
                        close = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Scale");
                    ui.add(
                        egui::DragValue::new(&mut menu.scale)
                            .range(0.2..=4.0)
                            .speed(0.05),
                    );
                    ui.checkbox(&mut menu.halo, "halo");
                });

                ui.separator();
                let is_pinned = pin_store.0.contains(&menu.path);
                let pin_label = if is_pinned {
                    "📌 Quitar pin"
                } else {
                    "📌 Pin"
                };
                if ui.button(pin_label).clicked() {
                    if is_pinned {
                        pin_store.0.remove(&menu.path);
                    } else {
                        pin_store.0.insert(menu.path.clone());
                    }
                    pins::save_pins(&pin_store.0);
                    nav.dirty = true;
                    close = true;
                }

                ui.separator();
                if ui.button("Esfera default").clicked() {
                    icons::set_override(&mut icon_cfg, &node_path, VisualSpec::sphere());
                    changed = true;
                    close = true;
                }
                if ui.button("Reset override").clicked() {
                    icons::clear_override(&mut icon_cfg, &node_path, &node_name);
                    changed = true;
                    close = true;
                }
            });
        });

    let dismissed = !was_just_opened && area_response.response.clicked_elsewhere();
    if changed {
        icons::save_icon_config(&icon_cfg);
        nav.dirty = true;
    }
    if close || dismissed {
        menu.open = false;
    }

    Ok(())
}
// Nombres de las subcarpetas del preview, pegados a cada dendrita. Aparecen en
// fade siguiendo el brote de su esfera.
pub fn draw_preview_labels(
    mut contexts: EguiContexts,
    cameras: Query<(&Camera, &GlobalTransform)>,
    previews: Query<(&GlobalTransform, &PreviewLabel, &PreviewAnim)>,
) -> Result {
    if previews.is_empty() {
        return Ok(());
    }
    let Ok((camera, cam_tf)) = cameras.single() else {
        return Ok(());
    };
    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("preview_labels"),
    ));

    for (gt, label, anim) in &previews {
        let fade = ((anim.age - anim.delay) / 0.12).clamp(0.0, 1.0);
        if fade <= 0.0 || label.0.is_empty() {
            continue;
        }
        let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) else {
            continue;
        };
        painter.text(
            egui::pos2(screen.x, screen.y + 10.0),
            egui::Align2::CENTER_TOP,
            &label.0,
            egui::FontId::proportional(10.5),
            egui::Color32::from_rgba_unmultiplied(150, 225, 220, (fade * 235.0) as u8),
        );
    }

    Ok(())
}

// Draw a reticle around the hovered (clickeable) node: two slowly-rotating arcs.
pub fn draw_hover_ring(
    mut contexts: EguiContexts,
    time: Res<Time>,
    hover: Res<HoverState>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    children: Query<(&LevelChild, &GlobalTransform)>,
    parents: Query<&GlobalTransform, With<ParentNode>>,
    echoes: Query<&GlobalTransform, With<EchoNode>>,
) -> Result {
    if hover.target == HoverTarget::None {
        return Ok(());
    }
    let Ok((camera, cam_tf)) = cameras.single() else {
        return Ok(());
    };

    // Resolve the dwelled node's world position.
    let world = match hover.target {
        HoverTarget::Child(id) => children
            .iter()
            .find(|(c, _)| c.0 == id)
            .map(|(_, gt)| gt.translation()),
        HoverTarget::Parent => parents.iter().next().map(|gt| gt.translation()),
        HoverTarget::Echo => echoes.iter().next().map(|gt| gt.translation()),
        HoverTarget::None => None,
    };
    let Some(world) = world else {
        return Ok(());
    };
    let Ok(screen) = camera.world_to_viewport(cam_tf, world) else {
        return Ok(());
    };

    let center = egui::pos2(screen.x, screen.y);

    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("ring"),
    ));

    // Retícula: dos arcos de 110° enfrentados que giran despacio — señala
    // "clickeable" sin ser un círculo muerto. Halo interior tenue de apoyo.
    let spin = time.elapsed_secs() * 0.9;
    let stroke = egui::Stroke::new(2.5_f32, egui::Color32::from_rgb(110, 230, 245));
    for half in 0..2 {
        let start = spin + half as f32 * std::f32::consts::PI;
        let points: Vec<egui::Pos2> = (0..=18)
            .map(|i| {
                let a = start + i as f32 / 18.0 * 1.92; // ~110°
                egui::pos2(center.x + a.cos() * 38.0, center.y + a.sin() * 38.0)
            })
            .collect();
        painter.add(egui::Shape::line(points, stroke));
    }
    painter.circle_stroke(
        center,
        30.0,
        egui::Stroke::new(
            1.0_f32,
            egui::Color32::from_rgba_premultiplied(60, 130, 140, 90),
        ),
    );

    Ok(())
}
// Draw each node's name (+ child count) at its projected screen position.
pub fn draw_labels(
    mut contexts: EguiContexts,
    cameras: Query<(&Camera, &GlobalTransform)>,
    labels: Query<(&NodeLabel, &GlobalTransform, Option<&LevelFolder>)>,
    constellation: Res<crate::constellation::Constellation>,
    lenses: Res<crate::lens::Lenses>,
    nav: Res<NavState>,
) -> Result {
    // Nivel oculto durante la constelación: sus labels tampoco se pintan.
    if constellation.active {
        return Ok(());
    }
    let Ok((camera, cam_tf)) = cameras.single() else {
        return Ok(());
    };

    // Keywords normalizadas una vez por frame: el anillo de lente es barato.
    let lens_index: Vec<((u8, u8, u8), Vec<String>)> = lenses
        .all
        .iter()
        .filter_map(|l| {
            let color = crate::lens::parse_color(&l.color)?;
            let kws = l.keywords.iter().map(|k| crate::ui::normalize(k)).collect();
            Some((color, kws))
        })
        .collect();
    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("labels"),
    ));

    for (label, gt, folder) in &labels {
        let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) else {
            continue;
        };

        // Miembro de una lente → anillo fino de su color (identidad temática).
        if matches!(label.kind, NodeKind::Hub | NodeKind::Child) {
            if let Some(folder) = folder {
                if let Some(norm_name) = nav.tree.normalized(folder.0) {
                    if let Some(((r, g, b), _)) = lens_index.iter().find(|(_, kws)| {
                        kws.iter()
                            .any(|kw| crate::search::score_normalized(norm_name, kw) > 0)
                    }) {
                        painter.circle_stroke(
                            egui::pos2(screen.x, screen.y),
                            26.0,
                            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(*r, *g, *b)),
                        );
                    }
                }
            }
        }
        let text = if label.count == 0 {
            label.name.clone()
        } else {
            format!("{}  ({})", label.name, label.count)
        };
        let (size, color, dy) = match label.kind {
            NodeKind::Hub => (16.0, egui::Color32::from_rgb(234, 250, 255), 22.0),
            NodeKind::Child => (12.5, egui::Color32::from_rgb(180, 222, 232), 16.0),
            NodeKind::Parent => (12.0, egui::Color32::from_rgb(150, 160, 175), 14.0),
            NodeKind::Echo => (12.0, egui::Color32::from_rgb(150, 135, 230), 28.0),
        };
        painter.text(
            egui::pos2(screen.x, screen.y + dy),
            egui::Align2::CENTER_TOP,
            text,
            egui::FontId::proportional(size),
            color,
        );
        // Carpeta sin subcarpetas: el vacío invita a actuar, no a mirar.
        if label.kind == NodeKind::Hub && label.count == 0 {
            painter.text(
                egui::pos2(screen.x, screen.y + dy + 22.0),
                egui::Align2::CENTER_TOP,
                "sin subcarpetas · Enter la abre en el explorador",
                egui::FontId::proportional(11.5),
                egui::Color32::from_rgb(110, 150, 160),
            );
        }
    }

    Ok(())
}

pub fn draw_preview_overflow(
    mut contexts: EguiContexts,
    state: Res<PreviewState>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    children: Query<(&LevelChild, &GlobalTransform)>,
) -> Result {
    let Some((id, hidden)) = state.overflow else {
        return Ok(());
    };
    let Ok((camera, cam_tf)) = cameras.single() else {
        return Ok(());
    };
    let Some((_, gt)) = children.iter().find(|(child, _)| child.0 == id) else {
        return Ok(());
    };
    let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation() + Vec3::new(1.2, 0.4, 0.0))
    else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("preview_overflow"),
    ));
    painter.text(
        egui::pos2(screen.x, screen.y),
        egui::Align2::CENTER_CENTER,
        format!("+{hidden}"),
        egui::FontId::proportional(11.0),
        egui::Color32::from_rgb(80, 190, 185),
    );

    Ok(())
}

// Breadcrumb top bar: BORJES · root / … / current. Cada segmento es un salto:
// click en un ancestro navega directo ahí (mismo history que el resto).
#[allow(deprecated)] // egui marks Panel::show deprecated but show(ctx) is correct (matches ui.rs)
pub fn draw_breadcrumb(mut contexts: EguiContexts, mut nav: ResMut<NavState>) -> Result {
    let mut chain: Vec<(usize, String)> = Vec::new();
    let mut cursor = Some(nav.current);
    while let Some(id) = cursor {
        if let Some(node) = nav.tree.node(id) {
            chain.push((id, node.name.clone()));
        }
        cursor = nav.tree.parent(id);
    }
    chain.reverse();

    let ctx = contexts.ctx_mut()?;
    let mut jump = None;
    egui::TopBottomPanel::top("breadcrumb").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("BORJES")
                    .strong()
                    .color(egui::Color32::from_rgb(90, 220, 235)),
            );
            ui.label(egui::RichText::new("·").weak());
            let last = chain.len().saturating_sub(1);
            for (pos, (id, name)) in chain.iter().enumerate() {
                if pos == last {
                    // El nivel actual no es un link: es donde estás parado.
                    ui.label(
                        egui::RichText::new(name)
                            .strong()
                            .color(egui::Color32::from_rgb(234, 250, 255)),
                    );
                } else {
                    let seg = ui.add(
                        egui::Label::new(
                            egui::RichText::new(name).color(egui::Color32::from_rgb(150, 200, 215)),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if seg.hovered() {
                        seg.clone().highlight();
                    }
                    if seg.clicked() {
                        jump = Some(*id);
                    }
                    ui.label(egui::RichText::new("/").weak());
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(
                        "clic entra · Tab entra al hover · Backspace sube · breadcrumb salta",
                    )
                    .weak()
                    .size(11.0),
                );
            });
        });
    });

    if let Some(id) = jump {
        nav.navigate(id);
    }

    Ok(())
}
