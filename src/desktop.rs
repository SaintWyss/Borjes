use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WINIT_WINDOWS;
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
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    keys: Res<ButtonInput<KeyCode>>,
    search: Res<crate::search::BorjesSearch>,
    mut focus_frames: Local<u8>,
    _main_thread: NonSendMarker,
) {
    let Ok((entity, mut window)) = windows.single_mut() else {
        return;
    };

    if TOGGLE_REQUESTED.swap(false, Ordering::SeqCst) {
        window.visible = !window.visible;
        if window.visible {
            window.focused = true;
            *focus_frames = 10;
        }
    }

    // KWin no da foco de teclado al re-mapear una ventana ya existente (focus
    // stealing prevention): el primer show tras el launch enfoca, los siguientes
    // no. Pedir el foco a winit durante unos frames post-show (cuando la ventana
    // ya está mapeada) para que Meta+1 → tipear funcione siempre.
    //
    // Además X11 LIMPIA _NET_WM_STATE_ABOVE al hacer unmap: si no se re-aplica,
    // el overlay re-aparece DEBAJO de las demás ventanas ("no me abre"). Se
    // re-pide always-on-top directo a winit durante los mismos frames.
    if *focus_frames > 0 && window.visible {
        *focus_frames -= 1;
        WINIT_WINDOWS.with_borrow(|winit_windows| {
            if let Some(winit_window) = winit_windows.get_window(entity) {
                winit_window.focus_window();
                // Directo a winit: Bevy cachea el campo y no re-aplica valores
                // "iguales", pero X11 ya descartó el ABOVE en el unmap.
                winit_window.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
            }
        });
    }

    if window.visible && keys.just_pressed(KeyCode::Escape) && search.query.is_empty() {
        window.visible = false;
    }
}
