use bevy::prelude::*;
use bevy_egui::EguiContexts;

use super::*;

pub(crate) const HOVER_RADIUS_PX: f32 = 95.0;
// Radio de "soltado": más grande que el de agarre, para que el centrado de
// cámara no corte el hover (ver histéresis en hover_track).
pub(crate) const STICKY_RADIUS_PX: f32 = 230.0;

// Which node (if any) is under the cursor right now.
pub(crate) fn node_under_cursor(
    cursor: Vec2,
    camera: &Camera,
    cam_tf: &GlobalTransform,
    children: &Query<(&LevelChild, &GlobalTransform)>,
    parents: &Query<&GlobalTransform, With<ParentNode>>,
    echoes: &Query<&GlobalTransform, With<EchoNode>>,
) -> HoverTarget {
    let mut best_child: Option<(usize, f32)> = None;
    for (child, gt) in children {
        if let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) {
            let d = screen.distance(cursor);
            if d < HOVER_RADIUS_PX && best_child.is_none_or(|(_, bd)| d < bd) {
                best_child = Some((child.0, d));
            }
        }
    }

    let mut parent_d = f32::INFINITY;
    for gt in parents {
        if let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) {
            let d = screen.distance(cursor);
            if d < HOVER_RADIUS_PX {
                parent_d = parent_d.min(d);
            }
        }
    }

    let mut echo_d = f32::INFINITY;
    for gt in echoes {
        if let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) {
            let d = screen.distance(cursor);
            if d < HOVER_RADIUS_PX {
                echo_d = echo_d.min(d);
            }
        }
    }

    match best_child {
        Some((id, cd)) if cd <= parent_d && cd <= echo_d => HoverTarget::Child(id),
        _ if parent_d <= echo_d && parent_d.is_finite() => HoverTarget::Parent,
        _ if echo_d.is_finite() => HoverTarget::Echo,
        _ => HoverTarget::None,
    }
}

pub(crate) fn folder_under_cursor(
    cursor: Vec2,
    camera: &Camera,
    cam_tf: &GlobalTransform,
    folders: &Query<(&LevelFolder, &GlobalTransform)>,
) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (folder, gt) in folders {
        if let Ok(screen) = camera.world_to_viewport(cam_tf, gt.translation()) {
            let d = screen.distance(cursor);
            if d < HOVER_RADIUS_PX && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((folder.0, d));
            }
        }
    }
    best.map(|(id, _)| id)
}
// Hover only *highlights* the node under the cursor (scale bump + gentle camera
// lean). Diving is an explicit left-click — auto-dive felt uncontrollable.
pub fn hover_track(
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    children: Query<(&LevelChild, &GlobalTransform)>,
    parents: Query<&GlobalTransform, With<ParentNode>>,
    echoes: Query<&GlobalTransform, With<EchoNode>>,
    constellation: Res<crate::constellation::Constellation>,
    mut hover: ResMut<HoverState>,
) -> Result {
    // Con la constelación activa el nivel está oculto: nada hovereable.
    if constellation.active {
        hover.target = HoverTarget::None;
        return Ok(());
    }
    let Ok(window) = windows.single() else {
        return Ok(());
    };
    let Some(cursor) = window.cursor_position() else {
        hover.target = HoverTarget::None;
        return Ok(());
    };
    if contexts.ctx_mut()?.is_pointer_over_egui() {
        hover.target = HoverTarget::None;
        return Ok(());
    }
    let Ok((camera, cam_tf)) = cameras.single() else {
        return Ok(());
    };

    let fresh = node_under_cursor(cursor, camera, cam_tf, &children, &parents, &echoes);

    // Histéresis: mientras la cámara centra al hijo hovereado, el nodo se aleja
    // del cursor quieto. Si nada nuevo cae bajo el cursor, el hijo actual sigue
    // "agarrado" hasta que el cursor quede realmente lejos.
    hover.target = match (fresh, hover.target) {
        (HoverTarget::None, HoverTarget::Child(id)) => {
            let still_close = children
                .iter()
                .find(|(c, _)| c.0 == id)
                .and_then(|(_, gt)| camera.world_to_viewport(cam_tf, gt.translation()).ok())
                .is_some_and(|screen| screen.distance(cursor) < STICKY_RADIUS_PX);
            if still_close {
                HoverTarget::Child(id)
            } else {
                HoverTarget::None
            }
        }
        (fresh, _) => fresh,
    };
    Ok(())
}

// Left-click the hovered node to dive: child = descend, parent ghost = ascend.
pub fn click_dive(
    mut contexts: EguiContexts,
    buttons: Res<ButtonInput<MouseButton>>,
    hover: Res<HoverState>,
    mut nav: ResMut<NavState>,
) -> Result {
    if !buttons.just_pressed(MouseButton::Left) || contexts.ctx_mut()?.is_pointer_over_egui() {
        return Ok(());
    }
    match hover.target {
        HoverTarget::Child(id) => {
            nav.navigate(id);
        }
        HoverTarget::Parent => {
            if let Some(parent) = nav.tree.parent(nav.current) {
                nav.navigate(parent);
            }
        }
        HoverTarget::Echo => nav.back(),
        HoverTarget::None => {}
    }
    Ok(())
}

pub fn tab_hover_nav(
    keys: Res<ButtonInput<KeyCode>>,
    hover: Res<HoverState>,
    search: Res<crate::search::BorjesSearch>,
    mut nav: ResMut<NavState>,
) {
    if !keys.just_pressed(KeyCode::Tab) || !search.results.is_empty() {
        return;
    }

    match hover.target {
        HoverTarget::Child(id) => nav.navigate(id),
        HoverTarget::Parent => {
            if let Some(parent) = nav.tree.parent(nav.current) {
                nav.navigate(parent);
            }
        }
        HoverTarget::Echo => nav.back(),
        HoverTarget::None => {}
    }
}
// Backspace ascends too (fallback to the hover-up gesture).
pub fn ascend(
    keys: Res<ButtonInput<KeyCode>>,
    mut nav: ResMut<NavState>,
    mut search: ResMut<crate::search::BorjesSearch>,
) {
    let requested = keys.just_pressed(KeyCode::Backspace) || search.ascend_requested;
    search.ascend_requested = false;

    if !requested {
        return;
    }
    if let Some(parent) = nav.tree.parent(nav.current) {
        nav.navigate(parent);
    }
}
