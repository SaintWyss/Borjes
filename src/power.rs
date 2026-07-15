use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::{UpdateMode, WinitSettings};
use std::time::Duration;

use crate::drill::{HoverState, HoverTarget, NavState};
use crate::search::BorjesSearch;

const SETTLE_FRAMES: u16 = 48;

#[derive(Resource)]
pub struct PowerGovernor {
    last_cursor: Option<Vec2>,
    was_visible: bool,
    active_frames: u16,
    sleeping: bool,
}

impl Default for PowerGovernor {
    fn default() -> Self {
        Self {
            last_cursor: None,
            was_visible: false,
            active_frames: 0,
            sleeping: true,
        }
    }
}

/// Switches Bevy between fluid rendering and a true event-driven sleep.
///
/// Hidden and visually settled windows wait indefinitely. Winit input events and
/// the external EventLoopProxy wake the app without polling. Permanent decorative
/// motion intentionally freezes at rest and resumes on the next interaction.
pub fn adaptive_render_mode(
    windows: Query<&Window, With<PrimaryWindow>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    nav: Res<NavState>,
    search: Res<BorjesSearch>,
    hover: Res<HoverState>,
    mut settings: ResMut<WinitSettings>,
    mut governor: ResMut<PowerGovernor>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let cursor = window.cursor_position();
    let cursor_changed = cursor != governor.last_cursor;
    let visibility_changed = window.visible != governor.was_visible;
    let keyboard_activity = keys.get_just_pressed().next().is_some()
        || keys.get_just_released().next().is_some();
    let mouse_activity = mouse.get_just_pressed().next().is_some()
        || mouse.get_just_released().next().is_some();

    let state_activity = nav.is_changed() || search.is_changed();
    if visibility_changed
        || cursor_changed
        || keyboard_activity
        || mouse_activity
        || state_activity
    {
        governor.active_frames = SETTLE_FRAMES;
    } else {
        governor.active_frames = governor.active_frames.saturating_sub(1);
    }

    let visual_activity = !matches!(hover.target, HoverTarget::None)
        || !search.query.is_empty()
        || nav.dirty;
    let should_render_continuously =
        window.visible && (governor.active_frames > 0 || visual_activity);

    if should_render_continuously && governor.sleeping {
        settings.focused_mode = UpdateMode::Continuous;
        governor.sleeping = false;
    } else if !should_render_continuously && !governor.sleeping {
        settings.focused_mode = UpdateMode::reactive(Duration::MAX);
        governor.sleeping = true;
    }

    governor.last_cursor = cursor;
    governor.was_visible = window.visible;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settle_budget_is_short_but_not_single_frame() {
        assert!((30..=120).contains(&SETTLE_FRAMES));
    }
}
