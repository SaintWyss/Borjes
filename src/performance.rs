use std::time::{Duration, Instant};
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, RequestRedraw};
use crate::{drill, search};

const ACTIVE_RENDER_WINDOW: Duration = Duration::from_millis(900);

#[derive(Resource)]
pub struct RenderGovernor { active_until: Instant, cursor: Option<Vec2>, visible: bool }
impl Default for RenderGovernor {
    fn default() -> Self { Self { active_until: Instant::now()+ACTIVE_RENDER_WINDOW, cursor: None, visible: false } }
}

pub fn govern_rendering(
    windows: Query<&Window, With<PrimaryWindow>>, keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>,
    nav: Res<drill::NavState>, hover: Res<drill::HoverState>, search: Res<search::BorjesSearch>,
    mut governor: ResMut<RenderGovernor>, mut redraw: MessageWriter<RequestRedraw>,
) {
    let Ok(window)=windows.single() else { return; };
    let cursor=window.cursor_position();
    let input_changed=governor.cursor!=cursor || keys.get_just_pressed().next().is_some() || mouse.get_just_pressed().next().is_some();
    let state_changed=nav.is_changed() || hover.is_changed() || search.is_changed();
    let visibility_changed=governor.visible!=window.visible;
    if input_changed || state_changed || visibility_changed { governor.active_until=Instant::now()+ACTIVE_RENDER_WINDOW; }
    governor.cursor=cursor; governor.visible=window.visible;
    if window.visible && Instant::now()<governor.active_until { redraw.write(RequestRedraw); }
}

#[cfg(test)] mod tests { use super::*; #[test] fn active_window_returns_to_idle(){ assert!(ACTIVE_RENDER_WINDOW<Duration::from_secs(2)); } }
