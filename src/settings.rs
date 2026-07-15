use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts};

use crate::lens::{self, Lens, Lenses};
use crate::root::{PendingRescan, RootConfig};

#[derive(Clone, Default)]
struct LensDraft {
    name: String,
    keywords: String,
    color: String,
}

impl From<&Lens> for LensDraft {
    fn from(lens: &Lens) -> Self {
        Self {
            name: lens.name.clone(),
            keywords: lens.keywords.join(", "),
            color: lens.color.clone(),
        }
    }
}

#[derive(Resource, Default)]
pub struct SettingsState {
    open: bool,
    initialized: bool,
    root: String,
    lenses: Vec<LensDraft>,
    status: Option<String>,
}

pub fn settings_ui(
    mut contexts: EguiContexts,
    windows: Query<&Window, With<PrimaryWindow>>,
    root: Res<RootConfig>,
    mut pending_rescan: ResMut<PendingRescan>,
    mut lenses: ResMut<Lenses>,
    mut settings: ResMut<SettingsState>,
) -> Result {
    let Ok(window) = windows.single() else {
        return Ok(());
    };
    if !window.visible {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    if ctx.input(|input| input.key_pressed(egui::Key::F10)) {
        settings.open = !settings.open;
    }

    egui::Area::new(egui::Id::new("settings_button"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-24.0, 22.0))
        .show(ctx, |ui| {
            if ui
                .button("⚙")
                .on_hover_text("Configuración · F10")
                .clicked()
            {
                settings.open = true;
            }
        });

    if settings.open && !settings.initialized {
        settings.root = root.path.clone();
        settings.lenses = lenses.all.iter().map(LensDraft::from).collect();
        settings.initialized = true;
    }
    if !settings.open {
        return Ok(());
    }

    let mut open = settings.open;
    let mut save = false;
    egui::Window::new("Configuración de Borjes")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(540.0)
        .show(ctx, |ui| {
            ui.heading("Biblioteca");
            ui.label("Carpeta raíz");
            ui.text_edit_singleline(&mut settings.root);
            ui.add_space(12.0);
            ui.separator();
            ui.heading("Lentes temáticas");
            ui.label("Separá las palabras clave con comas.");

            let mut remove = None;
            for (index, lens) in settings.lenses.iter_mut().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Nombre");
                        ui.text_edit_singleline(&mut lens.name);
                        ui.label("Color");
                        ui.add_sized([90.0, 22.0], egui::TextEdit::singleline(&mut lens.color));
                        if ui.small_button("Eliminar").clicked() {
                            remove = Some(index);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Keywords");
                        ui.text_edit_singleline(&mut lens.keywords);
                    });
                });
            }
            if let Some(index) = remove {
                settings.lenses.remove(index);
            }
            if ui.button("+ Nueva lente").clicked() {
                settings.lenses.push(LensDraft {
                    name: "Nueva lente".to_string(),
                    keywords: String::new(),
                    color: "#8cfaf2".to_string(),
                });
            }

            if let Some(status) = &settings.status {
                ui.add_space(8.0);
                ui.label(status);
            }
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Guardar y aplicar").clicked() {
                    save = true;
                }
                if ui.button("Cancelar").clicked() {
                    settings.initialized = false;
                    settings.open = false;
                }
            });
        });
    settings.open = open;

    if save {
        match validated_lenses(&settings.lenses) {
            Ok(updated) => {
                lens::save_lenses(&updated);
                lenses.all = updated;
                let requested = crate::root::expand_tilde(settings.root.trim());
                if requested != crate::root::expand_tilde(&root.path) {
                    pending_rescan.0 = Some(requested);
                }
                settings.status = Some("Configuración guardada".to_string());
                settings.initialized = false;
                settings.open = false;
            }
            Err(message) => settings.status = Some(message),
        }
    }

    Ok(())
}

fn validated_lenses(drafts: &[LensDraft]) -> Result<Vec<Lens>, String> {
    let mut result = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let name = draft.name.trim();
        let keywords: Vec<String> = draft
            .keywords
            .split(',')
            .map(str::trim)
            .filter(|keyword| !keyword.is_empty())
            .map(str::to_owned)
            .collect();
        if name.is_empty() {
            return Err("Cada lente necesita un nombre".to_string());
        }
        if keywords.is_empty() {
            return Err(format!("La lente {name} necesita al menos una keyword"));
        }
        if lens::parse_color(draft.color.trim()).is_none() {
            return Err(format!("El color de {name} debe tener formato #RRGGBB"));
        }
        result.push(Lens {
            name: name.to_string(),
            keywords,
            color: draft.color.trim().to_string(),
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_and_trims_lens_editor_values() {
        let drafts = vec![LensDraft {
            name: " Seguridad ".to_string(),
            keywords: " cyber, pentest, ".to_string(),
            color: "#ff6b6b".to_string(),
        }];
        let lenses = validated_lenses(&drafts).unwrap();
        assert_eq!(lenses[0].name, "Seguridad");
        assert_eq!(lenses[0].keywords, ["cyber", "pentest"]);
    }

    #[test]
    fn rejects_invalid_lens_color() {
        let drafts = vec![LensDraft {
            name: "Tema".to_string(),
            keywords: "uno".to_string(),
            color: "red".to_string(),
        }];
        assert!(validated_lenses(&drafts).is_err());
    }
}
