use bevy::prelude::*;
use bevy::window::{PrimaryWindow, RawHandleWrapper};
use raw_window_handle::RawWindowHandle;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt as XProtoConnectionExt, PropMode};
use x11rb::wrapper::ConnectionExt as X11WrapperConnectionExt;

#[derive(Resource, Default)]
pub struct DesktopModeApplied(pub bool);

pub fn apply_desktop_window_type(
    mut applied: ResMut<DesktopModeApplied>,
    q: Query<&RawHandleWrapper, With<PrimaryWindow>>,
) {
    if applied.0 {
        return;
    }

    let Ok(wrapper) = q.single() else {
        return;
    };

    let xid = match wrapper.get_window_handle() {
        RawWindowHandle::Xlib(handle) => handle.window as u32,
        RawWindowHandle::Xcb(handle) => handle.window.get(),
        other => {
            warn!("not an X11 window handle ({other:?}); desktop mode is X11-only");
            applied.0 = true;
            return;
        }
    };

    match set_desktop_window_type(xid) {
        Ok(()) => {
            info!("set _NET_WM_WINDOW_TYPE_DESKTOP on XID {xid:#x}");
            applied.0 = true;
        }
        Err(err) => {
            warn!("failed to set desktop window type on XID {xid:#x}: {err}");
            applied.0 = true;
        }
    }
}

fn set_desktop_window_type(xid: u32) -> Result<(), Box<dyn std::error::Error>> {
    let (conn, _screen) = x11rb::connect(None)?;

    let wm_type_atom = intern_atom(&conn, b"_NET_WM_WINDOW_TYPE")?;
    let desktop_atom = intern_atom(&conn, b"_NET_WM_WINDOW_TYPE_DESKTOP")?;
    let wm_state_atom = intern_atom(&conn, b"_NET_WM_STATE")?;
    let below_atom = intern_atom(&conn, b"_NET_WM_STATE_BELOW")?;
    let skip_taskbar_atom = intern_atom(&conn, b"_NET_WM_STATE_SKIP_TASKBAR")?;
    let skip_pager_atom = intern_atom(&conn, b"_NET_WM_STATE_SKIP_PAGER")?;

    conn.change_property32(
        PropMode::REPLACE,
        xid,
        wm_type_atom,
        AtomEnum::ATOM,
        &[desktop_atom],
    )?
    .check()?;

    conn.change_property32(
        PropMode::REPLACE,
        xid,
        wm_state_atom,
        AtomEnum::ATOM,
        &[below_atom, skip_taskbar_atom, skip_pager_atom],
    )?
    .check()?;

    conn.flush()?;

    Ok(())
}

fn intern_atom<C>(conn: &C, name: &[u8]) -> Result<Atom, Box<dyn std::error::Error>>
where
    C: XProtoConnectionExt,
{
    Ok(conn.intern_atom(false, name)?.reply()?.atom)
}
