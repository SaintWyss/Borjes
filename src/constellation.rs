use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::drill::{CamTarget, LevelNode, NavState};
use crate::root::{expand_tilde, RootConfig};
use crate::search::short_path;

// Modo constelación: con query activa, el nivel actual se esconde y TODOS los
// matches del árbol aparecen como estrellas agrupadas por rama raíz. Tab cicla
// el foco, Enter abre, click teletransporta (eso lo cablea search_ui).
#[derive(Resource, Default)]
pub struct Constellation {
    pub stars: Vec<usize>,
    pub focus: usize,
    pub active: bool,
    pub dirty: bool,
    // Cuántas ramas raíz distintas aportan estrellas (para el contador).
    pub branches: usize,
    // Tema (lente activa): título y color propios; None = búsqueda libre teal.
    pub title: Option<String>,
    pub color: Option<(u8, u8, u8)>,
    was_active: bool,
}

impl Constellation {
    pub fn set(&mut self, stars: Vec<usize>) {
        self.branches = 0; // recalculado al renderizar
        self.focus = 0;
        self.active = !stars.is_empty();
        self.stars = stars;
        self.title = None;
        self.color = None;
        self.dirty = true;
    }

    // Constelación temática: una lente montada como campo de estrellas.
    pub fn set_lens(&mut self, stars: Vec<usize>, title: String, color: (u8, u8, u8)) {
        self.set(stars);
        self.title = Some(title);
        self.color = Some(color);
    }

    pub fn clear(&mut self) {
        if self.active || !self.stars.is_empty() {
            self.dirty = true;
        }
        self.stars.clear();
        self.focus = 0;
        self.active = false;
    }

    pub fn cycle(&mut self, dir: isize) {
        if self.stars.is_empty() {
            return;
        }
        let len = self.stars.len() as isize;
        self.focus = ((self.focus as isize + dir).rem_euclid(len)) as usize;
    }

    pub fn focused(&self) -> Option<usize> {
        self.stars.get(self.focus).copied()
    }
}

#[derive(Component)]
pub struct StarNode(pub usize);

#[derive(Component)]
pub struct StarAnim {
    pub age: f32,
    pub delay: f32,
    pub base: f32,
}

const CLUSTER_RING: f32 = 13.0;
const STAR_GROW_SECS: f32 = 0.14;

// Reconstruye el campo de estrellas cuando la constelación cambia. Esconde el
// nivel normal mientras está activa y lo restaura al salir.
pub fn render_constellation(
    mut commands: Commands,
    mut constellation: ResMut<Constellation>,
    nav: Res<NavState>,
    mut cam_target: ResMut<CamTarget>,
    stars_q: Query<Entity, With<StarNode>>,
    mut level_q: Query<&mut Visibility, With<LevelNode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !constellation.dirty {
        // El nivel puede haberse reconstruido (nav.dirty) mientras la
        // constelación está activa: mantenerlo oculto.
        if constellation.active {
            for mut vis in &mut level_q {
                *vis = Visibility::Hidden;
            }
        }
        return;
    }
    constellation.dirty = false;

    for entity in &stars_q {
        commands.entity(entity).despawn();
    }

    if !constellation.active {
        if constellation.was_active {
            constellation.was_active = false;
            for mut vis in &mut level_q {
                *vis = Visibility::Inherited;
            }
        }
        return;
    }
    constellation.was_active = true;

    for mut vis in &mut level_q {
        *vis = Visibility::Hidden;
    }

    // Agrupar estrellas por rama raíz: cada grupo es un cúmulo en el anillo.
    let mut groups: Vec<(usize, Vec<usize>)> = Vec::new();
    for &id in &constellation.stars {
        let Some(node) = nav.tree.node(id) else {
            continue;
        };
        match groups.iter_mut().find(|(g, _)| *g == node.root_group) {
            Some((_, members)) => members.push(id),
            None => groups.push((node.root_group, vec![id])),
        }
    }
    constellation.branches = groups.len();

    let sphere = meshes.add(Sphere::new(1.0));
    // Color del campo: teal en búsqueda libre, el de la lente si hay tema.
    let (r, g, b) = constellation.color.unwrap_or((140, 250, 242));
    let base = Color::srgb_u8(r, g, b);
    let lin = base.to_linear();
    let mut star_mat = StandardMaterial {
        base_color: base,
        emissive: LinearRgba::rgb(lin.red * 4.5, lin.green * 4.5, lin.blue * 4.5),
        unlit: true,
        ..default()
    };
    star_mat.alpha_mode = AlphaMode::Blend;
    let star_mat = materials.add(star_mat);

    let group_count = groups.len().max(1);
    let mut order = 0usize;
    for (gi, (_, members)) in groups.iter().enumerate() {
        // Cúmulo: centro en el anillo; adentro, espiral de girasol (ángulo áureo).
        let ga =
            gi as f32 * std::f32::consts::TAU / group_count as f32 + std::f32::consts::FRAC_PI_2;
        let center = if group_count == 1 {
            Vec3::ZERO
        } else {
            Vec3::new(ga.cos() * CLUSTER_RING, ga.sin() * CLUSTER_RING, 0.0)
        };
        for (si, &id) in members.iter().enumerate() {
            let a = si as f32 * 2.399_963; // golden angle
            let r = 1.15 * (si as f32).sqrt();
            let pos = center + Vec3::new(a.cos() * r, a.sin() * r, (si % 3) as f32 * 0.3);
            commands.spawn((
                Mesh3d(sphere.clone()),
                MeshMaterial3d(star_mat.clone()),
                Transform::from_translation(pos).with_scale(Vec3::ZERO),
                StarNode(id),
                StarAnim {
                    age: 0.0,
                    delay: order as f32 * 0.012,
                    base: 0.42,
                },
            ));
            order += 1;
        }
    }

    // Alejar la cámara para abrazar el anillo completo de cúmulos.
    cam_target.distance = (CLUSTER_RING + 8.0) * 2.4;
}

// Grow-in staggered + pulso del foco.
pub fn animate_stars(
    time: Res<Time>,
    constellation: Res<Constellation>,
    mut stars: Query<(&StarNode, &mut Transform, &mut StarAnim)>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    let focused = constellation.focused();
    for (star, mut tf, mut anim) in &mut stars {
        anim.age += dt;
        let g = ((anim.age - anim.delay).max(0.0) / STAR_GROW_SECS).clamp(0.0, 1.0);
        let grow = 1.0 - (1.0 - g).powi(3);
        // La estrella en foco late y crece; el resto queda sereno.
        let scale = if focused == Some(star.0) {
            anim.base * grow * (1.55 + 0.12 * (t * 4.0).sin())
        } else {
            anim.base * grow
        };
        tf.scale = Vec3::splat(scale);
    }
}

// Nombre + path corto por estrella; la focada lleva retícula y texto pleno.
pub fn draw_star_labels(
    mut contexts: EguiContexts,
    constellation: Res<Constellation>,
    nav: Res<NavState>,
    root: Res<RootConfig>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    stars: Query<(&StarNode, &GlobalTransform)>,
) -> Result {
    if !constellation.active {
        return Ok(());
    }
    let Ok((camera, cam_tf)) = cameras.single() else {
        return Ok(());
    };
    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("star_labels"),
    ));
    let root_path = expand_tilde(&root.path);
    let focused = constellation.focused();

    for (star, gt) in &stars {
        let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) else {
            continue;
        };
        let Some(node) = nav.tree.node(star.0) else {
            continue;
        };
        let is_focus = focused == Some(star.0);
        let (size, color) = if is_focus {
            (13.5, egui::Color32::from_rgb(240, 255, 253))
        } else {
            (10.5, egui::Color32::from_rgb(140, 205, 200))
        };
        painter.text(
            egui::pos2(screen.x, screen.y + 12.0),
            egui::Align2::CENTER_TOP,
            &node.name,
            egui::FontId::proportional(size),
            color,
        );
        if is_focus {
            painter.text(
                egui::pos2(screen.x, screen.y + 12.0 + size + 3.0),
                egui::Align2::CENTER_TOP,
                short_path(&node.path, &root_path),
                egui::FontId::proportional(10.0),
                egui::Color32::from_rgb(120, 165, 175),
            );
            painter.circle_stroke(
                egui::pos2(screen.x, screen.y),
                24.0,
                egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(110, 230, 245)),
            );
        }
    }

    // Contador global, arriba del campo: "[lente —] N carpetas en M ramas".
    let counter = format!(
        "{} carpetas en {} rama{}",
        constellation.stars.len(),
        constellation.branches,
        if constellation.branches == 1 { "" } else { "s" }
    );
    let (text, color) = match (&constellation.title, constellation.color) {
        (Some(title), Some((r, g, b))) => (
            format!("◉ {title} — {counter}"),
            egui::Color32::from_rgb(r, g, b),
        ),
        _ => (counter, egui::Color32::from_rgb(110, 190, 195)),
    };
    painter.text(
        egui::pos2(ctx.content_rect().center().x, 132.0),
        egui::Align2::CENTER_TOP,
        text,
        egui::FontId::proportional(12.5),
        color,
    );

    Ok(())
}

// Click en una estrella = teletransporte al nivel de esa carpeta.
pub fn star_click(
    mut contexts: EguiContexts,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    stars: Query<(&StarNode, &GlobalTransform)>,
    mut constellation: ResMut<Constellation>,
    mut nav: ResMut<NavState>,
    mut search: ResMut<crate::search::BorjesSearch>,
) -> Result {
    if !constellation.active
        || !buttons.just_pressed(MouseButton::Left)
        || contexts.ctx_mut()?.is_pointer_over_egui()
    {
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

    let mut best: Option<(usize, f32)> = None;
    for (star, gt) in &stars {
        if let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) {
            let d = screen.distance(cursor);
            if d < 55.0 && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((star.0, d));
            }
        }
    }
    if let Some((id, _)) = best {
        nav.navigate(id);
        constellation.clear();
        search.query.clear();
        search.results.clear();
        search.selected = 0;
        search.focus_requested = true;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_wraps_both_directions() {
        let mut c = Constellation {
            stars: vec![10, 20, 30],
            ..Default::default()
        };
        c.cycle(1);
        assert_eq!(c.focus, 1);
        c.cycle(-2);
        assert_eq!(c.focus, 2); // wrap hacia atrás
        c.cycle(1);
        assert_eq!(c.focus, 0);
    }

    #[test]
    fn set_activates_and_clear_deactivates() {
        let mut c = Constellation::default();
        c.set(vec![1, 2]);
        assert!(c.active && c.dirty);
        assert_eq!(c.focused(), Some(1));
        c.dirty = false;
        c.clear();
        assert!(!c.active && c.dirty);
        assert_eq!(c.focused(), None);
    }

    #[test]
    fn set_empty_stays_inactive() {
        let mut c = Constellation::default();
        c.set(Vec::new());
        assert!(!c.active);
    }
}
