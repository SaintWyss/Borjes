use super::*;

#[derive(Resource)]
pub struct CamTarget {
    pub distance: f32,
}

impl Default for CamTarget {
    fn default() -> Self {
        Self { distance: 60.0 }
    }
}
pub fn animate_camera(
    time: Res<Time>,
    target: Res<CamTarget>,
    hover: Res<HoverState>,
    preview: Res<PreviewState>,
    children: Query<(&LevelChild, &GlobalTransform)>,
    mut cam: Query<&mut Transform, With<Camera3d>>,
    mut pan: Local<Vec2>,
) {
    let Ok(mut cam) = cam.single_mut() else {
        return;
    };

    const SPEED: f32 = 9.0;
    const PAN_SPEED: f32 = 5.0;
    // Acercamiento progresivo: arranca con un lean suave y, mientras el hover se
    // sostiene, la cámara va CENTRANDO el nodo y acercándose (el preview de sus
    // hijos aparece en el camino). La histéresis de hover_track evita que el
    // nodo "se escape" del cursor al moverse hacia el centro.
    const LEAN_MIN: f32 = 0.12;
    const LEAN_MAX: f32 = 0.55;
    const ZOOM_MAX: f32 = 0.18;

    // Rampa 0→1 durante ~0.5s de hover sostenido sobre el mismo hijo.
    let ramp = match hover.target {
        HoverTarget::Child(id) if preview.target == Some(id) => {
            ((preview.elapsed - 0.1) / 0.5).clamp(0.0, 1.0)
        }
        _ => 0.0,
    };
    let lean = LEAN_MIN + ramp * (LEAN_MAX - LEAN_MIN);

    let want = match hover.target {
        HoverTarget::Child(id) => children
            .iter()
            .find(|(c, _)| c.0 == id)
            .map(|(_, gt)| gt.translation().truncate() * lean)
            .unwrap_or(Vec2::ZERO),
        _ => Vec2::ZERO,
    };

    let dt = time.delta_secs();
    let cur = *pan;
    *pan = cur + (want - cur) * (1.0 - (-dt * PAN_SPEED).exp());

    let z = cam.translation.z;
    let z_target = target.distance * (1.0 - ZOOM_MAX * ramp);
    // Dive = rápido (SPEED); acercamiento por hover = suave (PAN_SPEED).
    let z_rate = if ramp > 0.0 { PAN_SPEED } else { SPEED };
    let eased_z = z + (z_target - z) * (1.0 - (-dt * z_rate).exp());
    let look = Vec3::new(pan.x, pan.y, 0.0);
    *cam = Transform::from_xyz(pan.x, pan.y, eased_z).looking_at(look, Vec3::Y);
}
