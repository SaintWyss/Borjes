use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::sync::atomic::{AtomicBool, Ordering};

static TOGGLE_REQUESTED: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_sigusr1(_sig: i32) {
    TOGGLE_REQUESTED.store(true, Ordering::SeqCst);
}

pub fn install_signal_handler() {
    // SAFETY: handler only does an atomic store, async-signal-safe.
    unsafe {
        libc::signal(
            libc::SIGUSR1,
            handle_sigusr1 as *const () as libc::sighandler_t,
        );
    }
}

pub fn toggle_overlay_visibility(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    keys: Res<ButtonInput<KeyCode>>,
    search: Res<crate::search::NeuraSearch>,
) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    if TOGGLE_REQUESTED.swap(false, Ordering::SeqCst) {
        window.visible = !window.visible;
        if window.visible {
            window.focused = true;
        }
    }

    if window.visible && keys.just_pressed(KeyCode::Escape) && search.query.is_empty() {
        window.visible = false;
    }
}
