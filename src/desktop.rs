use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::{EventLoopProxyWrapper, WinitUserEvent, WINIT_WINDOWS};
use std::sync::atomic::{AtomicBool, Ordering};

static TOGGLE_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Blocks SIGUSR1 in every subsequently-created thread. A dedicated sigwait
/// thread consumes it later, so no async signal handler or polling is needed.
pub fn install_signal_handler() {
    unsafe {
        let mut signals = std::mem::zeroed();
        libc::sigemptyset(&mut signals);
        libc::sigaddset(&mut signals, libc::SIGUSR1);
        let result = libc::pthread_sigmask(libc::SIG_BLOCK, &signals, std::ptr::null_mut());
        if result != 0 {
            eprintln!("borjes: could not block SIGUSR1 ({result})");
        }
    }
}

/// Waits without consuming CPU and wakes Bevy exactly when SIGUSR1 arrives.
pub fn start_signal_listener(proxy: Res<EventLoopProxyWrapper>) {
    let proxy: winit::event_loop::EventLoopProxy<WinitUserEvent> = (**proxy).clone();

    let _ = std::thread::Builder::new()
        .name("borjes-signal-listener".to_string())
        .spawn(move || loop {
            let mut signal = 0;
            let result = unsafe {
                let mut signals = std::mem::zeroed();
                libc::sigemptyset(&mut signals);
                libc::sigaddset(&mut signals, libc::SIGUSR1);
                libc::sigwait(&signals, &mut signal)
            };

            if result != 0 {
                eprintln!("borjes: sigwait failed ({result})");
                break;
            }

            if signal == libc::SIGUSR1 {
                TOGGLE_REQUESTED.store(true, Ordering::Release);
                if proxy.send_event(WinitUserEvent::WakeUp).is_err() {
                    break;
                }
            }
        });
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

    if TOGGLE_REQUESTED.swap(false, Ordering::AcqRel) {
        window.visible = !window.visible;
        if window.visible {
            window.focused = true;
            *focus_frames = 10;
        }
    }

    // KWin does not restore focus/ABOVE after an X11 unmap. Re-apply both
    // briefly after showing, once the native window has been mapped again.
    if *focus_frames > 0 && window.visible {
        *focus_frames -= 1;
        WINIT_WINDOWS.with_borrow(|winit_windows| {
            if let Some(winit_window) = winit_windows.get_window(entity) {
                winit_window.focus_window();
                winit_window.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
            }
        });
    }

    if window.visible && keys.just_pressed(KeyCode::Escape) && search.query.is_empty() {
        window.visible = false;
    }
}
