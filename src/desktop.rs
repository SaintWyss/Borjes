use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::{EventLoopProxyWrapper, WinitUserEvent, WINIT_WINDOWS};
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

static TOGGLE_REQUESTS: AtomicUsize = AtomicUsize::new(0);
static SIGNAL_READ_FD: AtomicI32 = AtomicI32::new(-1);
static SIGNAL_WRITE_FD: AtomicI32 = AtomicI32::new(-1);

extern "C" fn handle_sigusr1(_sig: i32) {
    TOGGLE_REQUESTS.fetch_add(1, Ordering::SeqCst);
    let fd = SIGNAL_WRITE_FD.load(Ordering::Relaxed);
    if fd >= 0 {
        let byte = 1_u8;
        // SAFETY: write(2) is async-signal-safe; the non-blocking pipe ensures
        // the handler can never wait behind the render thread.
        unsafe {
            libc::write(fd, (&byte as *const u8).cast(), 1);
        }
    }
}

pub fn install_signal_handler() {
    let mut fds = [-1_i32; 2];
    // SAFETY: fds points to two valid integers. The read side remains blocking;
    // only the signal-handler write side is made non-blocking.
    unsafe {
        if libc::pipe(fds.as_mut_ptr()) == 0 {
            let flags = libc::fcntl(fds[1], libc::F_GETFL);
            if flags >= 0 {
                libc::fcntl(fds[1], libc::F_SETFL, flags | libc::O_NONBLOCK);
            }
            SIGNAL_READ_FD.store(fds[0], Ordering::Release);
            SIGNAL_WRITE_FD.store(fds[1], Ordering::Release);
        }
        libc::signal(
            libc::SIGUSR1,
            handle_sigusr1 as *const () as libc::sighandler_t,
        );
    }
}

/// Bridges Unix signals into winit user events. This lets the hidden overlay use
/// `WinitSettings::desktop_app()` and sleep for real instead of polling at 4 Hz.
pub fn start_signal_wakeup(event_loop: Res<EventLoopProxyWrapper>) {
    let read_fd = SIGNAL_READ_FD.swap(-1, Ordering::AcqRel);
    if read_fd < 0 {
        warn!("signal wake-up pipe unavailable; toggle latency may increase");
        return;
    }

    let event_loop = (*event_loop).clone();
    std::thread::Builder::new()
        .name("borjes-signal-wakeup".to_string())
        .spawn(move || {
            let mut byte = 0_u8;
            loop {
                // SAFETY: read_fd is owned by this thread and byte is a valid
                // one-byte output buffer. The descriptor closes with process exit.
                let read = unsafe { libc::read(read_fd, (&mut byte as *mut u8).cast(), 1) };
                if read == 1 {
                    if event_loop.send_event(WinitUserEvent::WakeUp).is_err() {
                        break;
                    }
                } else if read == 0 {
                    break;
                } else {
                    let err = std::io::Error::last_os_error();
                    if err.kind() != std::io::ErrorKind::Interrupted {
                        break;
                    }
                }
            }
        })
        .expect("failed to start signal wake-up bridge");
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

    if TOGGLE_REQUESTS.swap(0, Ordering::SeqCst) % 2 == 1 {
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
