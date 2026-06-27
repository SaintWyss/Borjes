# Phase 6: Ventana Desktop X11 - Research

**Researched:** 2026-06-27
**Domain:** X11/EWMH window-type control, winit 0.30 ↔ Bevy 0.19 integration, KDE Plasma (KWin + plasmashell) desktop-layer coexistence
**Confidence:** MEDIUM — the *mechanism* to set the type is HIGH confidence; the *coexistence with plasmashell* is a genuine runtime risk that must be validated by a spike, not assumed.

## Summary

Two halves to this phase, with very different confidence:

1. **Setting `_NET_WM_WINDOW_TYPE_DESKTOP` (DESK-01) — solvable, HIGH confidence on the "how".** winit 0.30.13 (the version Bevy 0.19 pins) *does* expose `WindowAttributesExtX11::with_x11_window_type(vec![WindowType::Desktop])`. **But Bevy 0.19's `bevy_winit` gives you no hook to inject those X11 attributes at window-creation time** — it builds `WindowAttributes` internally and never passes through `with_x11_window_type`. So the native winit path is *unreachable through Bevy*. The concrete escape hatch is: after Bevy creates the window, grab the X11 window ID from the `RawHandleWrapper` component and set the `_NET_WM_WINDOW_TYPE` property yourself with **x11rb** (already in your lockfile as a transitive dep, v0.13.2).

2. **Coexisting with Plasma's wallpaper (DESK-02) — THE risk spike, MEDIUM confidence, may not work as the spec assumes.** plasmashell *is itself* a `_NET_WM_WINDOW_TYPE_DESKTOP` application. You are putting a second desktop-type window into the same KWin "desktop layer". The well-documented conky experience on KDE (conky #145, #2306, KDE bug 348987) is that a third-party `own_window_type desktop` window **gets covered by plasmashell's wallpaper the moment the user clicks the desktop** — KWin raises plasmashell within the desktop layer. The conky community's proven fix is to abandon `desktop` type and use a `normal` window with `below + skip_taskbar + skip_pager + sticky` hints instead. This directly tensions with locked decision **D1 ("NO always-below")**. This must be validated first and surfaced to the user, because the locked approach has a real chance of failing on KDE.

**Primary recommendation:** Plan this phase **spike-first**. Wave 0 = a throwaway runtime spike that (a) sets `_NET_WM_WINDOW_TYPE_DESKTOP` via x11rb on the live Bevy window and (b) checks with `xprop` whether KWin actually reclassifies it, whether it survives a desktop-click, whether Meta+D reveals it, and whether the egui search box can still receive keyboard focus. Branch the rest of the plan on the spike's outcome, with the conky `normal+below` recipe as the documented fallback. Do **not** write the full integration as if `DESKTOP` type is guaranteed to behave.

---

<user_constraints>
## User Constraints (from design spec + REQUIREMENTS.md)

> No CONTEXT.md exists for this phase (discuss was skipped). The locked decisions below come from
> the APPROVED design spec `docs/superpowers/specs/2026-06-27-desktop-mode-design.md` and
> `.planning/REQUIREMENTS.md`. Treat them as locked.

### Locked Decisions
- **D1 — Ventana tipo Desktop X11:** winit sets `_NET_WM_WINDOW_TYPE_DESKTOP` natively; KDE treats it as the desktop, it stays below everything, Meta+D reveals it. **"NO xwinwrap, NO always-below."**
- **D2 — Revelado:** Show Desktop (Meta+D) / minimizing windows. No overlay/hotkey-to-front.
- **DESK-01:** runs as Desktop X11 window, no decoration, fullscreen, below all windows.
- **DESK-02:** revealed by Meta+D / minimize; coexists with Plasma wallpaper (Plasma background set flat/transparent so it doesn't fight the app).
- Stack is locked: Rust, bevy 0.19.0, bevy_panorbit_camera 0.35, bevy_egui 0.40.1, winit 0.30.x. Target: CachyOS + KDE Plasma on **X11** (NOT Wayland — chosen because of AnyDesk).

### Claude's Discretion
- The concrete escape-hatch mechanism (x11rb vs xcb vs x11-dl) to set the atom.
- Whether to also set `_NET_WM_STATE_BELOW` / `_NET_WM_STATE_SKIP_TASKBAR` / `_NET_WM_STATE_SKIP_PAGER` as belt-and-suspenders.
- Spike design and fallback strategy.

### Deferred Ideas (OUT OF SCOPE for Phase 6)
- Files on-demand, pins, persistence (Phases 7–8).
- **Autostart at login (DESK-03) is Phase 9 — NOT this phase.** Do not add autostart entries here.
- Wayland support, multiplatform, replacing the WM.

> ⚠️ **Research flag against D1:** The "NO always-below" constraint may be incompatible with making
> the window reliably visible on KDE. The conky evidence (see Common Pitfalls) shows pure DESKTOP
> type gets covered by plasmashell on desktop-click, and the documented working fix *is* an
> always-below normal window. The planner should treat D1 as "preferred, pending spike" and route
> a fallback decision back to the user if the spike shows DESKTOP type is unworkable.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DESK-01 | Desktop X11 window (`_NET_WM_WINDOW_TYPE_DESKTOP`), no decoration, fullscreen, below all windows | winit 0.30 API confirmed but unreachable via Bevy → x11rb escape hatch (atom-set on raw handle). Borderless fullscreen + decorations via Bevy `Window` component. Stacking handled by KWin "desktop layer" for DESKTOP type (no manual `_NET_WM_STATE_BELOW` needed *if* type sticks). |
| DESK-02 | Revealed by Meta+D / minimize; coexists with Plasma wallpaper (flat/transparent Plasma background) | EWMH: DESKTOP/DOCK windows are exempt from "show desktop" → reveal works. Coexistence is the spike: plasmashell is also a DESKTOP window; stacking between the two is the unknown. Plasma "Plain Color" wallpaper documented as cosmetic mitigation; true plasmashell transparency not supported. |
</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Window creation, fullscreen, no decorations | Bevy `Window` component (winit) | — | Bevy owns the winit window; `decorations` + `mode` are first-class Bevy fields. |
| Setting `_NET_WM_WINDOW_TYPE_DESKTOP` | x11rb (direct X server) | — | Bevy/winit don't expose it post-creation; must talk to X11 directly via the window XID. |
| Stacking below all windows | KWin (window manager) | x11rb (`_NET_WM_STATE_BELOW` fallback) | KWin auto-places DESKTOP type in the bottom layer per EWMH; manual state only if type doesn't stick. |
| Reveal on Meta+D | KWin (`_NET_SHOWING_DESKTOP`) | — | WM-level; DESKTOP windows exempt by EWMH spec. No app code. |
| Wallpaper coexistence | plasmashell config (user) | x11rb stacking | A Plasma-side config change ("Plain Color" / transparent), per-machine, with user OK. App cannot force plasmashell's layer. |
| Keep rendering while occluded | Bevy `WinitSettings` (`unfocused_mode`) | — | A desktop window is never "focused"; must keep updating. |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| bevy | 0.19.0 | App/render (already in use) | Locked. |
| winit (via bevy_winit) | 0.30.13 | Windowing backend Bevy wraps | Pinned transitively; exposes X11 ext (but unreachable through Bevy). `[VERIFIED: Cargo.lock]` |
| raw-window-handle | 0.6.2 | Extract the X11 window XID from Bevy's `RawHandleWrapper` | Already in tree; the canonical handle bridge. `[VERIFIED: Cargo.lock]` |
| x11rb | 0.13.2 | Set `_NET_WM_WINDOW_TYPE` / `_NET_WM_STATE` atoms on the live window | Pure-Rust X11 client, already a transitive dep (pulled via winit); no new C linkage. `[VERIFIED: Cargo.lock present]` |

### Supporting (verification tooling, not deps)
| Tool | Purpose | When to Use |
|------|---------|-------------|
| `xprop` | Inspect `_NET_WM_WINDOW_TYPE`, `_NET_WM_STATE` on the live window | Spike + every verification step. Present at `/usr/bin/xprop` v1.2.8. `[VERIFIED: which xprop]` |
| `xwininfo` | Confirm geometry/override-redirect | Spike if geometry wrong. |
| `spectacle -b -n -a -o shot.png` | Headless screenshot for visual verification | Per project STATE.md convention. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| x11rb | `x11-dl` (Xlib FFI, also in lock) | x11-dl is unsafe FFI; x11rb is safe, idiomatic, async-capable, and already used by winit. Prefer x11rb. |
| x11rb own connection | Reuse winit's `Display*` from `RawDisplayHandle` (Xlib `XCBConnection::from_raw_xcb_connection`) | Reusing the display avoids a 2nd connection but couples to winit internals and risks thread-safety issues with Bevy. A fresh `x11rb::connect(None)` is simpler and the XID is server-global. Recommend a fresh connection. `[ASSUMED]` |
| DESKTOP type (D1) | Normal window + `_NET_WM_STATE_BELOW` + skip_taskbar/pager + sticky | The conky-proven KDE recipe. Contradicts D1 but is the documented working path; keep as spike fallback. |

**Installation (add x11rb + raw-window-handle as DIRECT deps):**
```bash
cargo add x11rb@0.13 raw-window-handle@0.6
```
> Both are already in the resolved tree transitively, so this pins them as direct deps without changing the build graph. No new system libraries (x11rb speaks the X protocol over a socket; no libX11 link required for the `rust-connection` default feature).

**Version verification:** `winit 0.30.13`, `raw-window-handle 0.6.2`, `x11rb 0.13.2`, `bevy_winit 0.19.0` all confirmed present in `/home/saintwyss/dev/datatree-rs/Cargo.lock`. crates.io API was unreachable from the sandbox (no outbound net), so "latest published" was not re-confirmed — but the lockfile versions are what will build, which is what matters here.

## Package Legitimacy Audit

> slopcheck could not be run: `pip install slopcheck` is blocked by the user's standing
> no-install-without-permission policy (denied by sandbox). Per the graceful-degradation rule,
> new direct deps are tagged `[ASSUMED]` — **but** note that both packages below are **already
> present in the lockfile as transitive dependencies of winit/bevy**, so promoting them to direct
> deps adds **zero new attack surface**. This materially lowers the risk versus a genuinely new package.

| Package | Registry | Status in tree | Source Repo | slopcheck | Disposition |
|---------|----------|----------------|-------------|-----------|-------------|
| x11rb | crates.io | Already transitive (0.13.2) via winit | github.com/psychon/x11rb | not run | Approved — pre-existing dep, well-known (psychon), used by winit/smithay. `[ASSUMED]` (slopcheck unavailable) |
| raw-window-handle | crates.io | Already transitive (0.6.2) via bevy/winit | github.com/rust-windowing/raw-window-handle | not run | Approved — rust-windowing org, the ecosystem-standard handle crate. `[ASSUMED]` |

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none
**Planner note:** Because slopcheck was unavailable, gate the `cargo add` behind a `checkpoint:human-verify` task. The check is trivial: both crates are already in `Cargo.lock` — confirm the version strings match before adding. No registry round-trip needed.

---

## Architecture Patterns

### System Architecture Diagram

```
            ┌─────────────────────────────────────────────────────────┐
            │  Bevy App (datatree-rs)                                   │
            │                                                           │
  Startup   │  DefaultPlugins → bevy_winit creates winit Window        │
  ────────► │      with Window { decorations:false,                    │
            │                    mode: BorderlessFullscreen(...) }      │
            │                         │                                 │
            │                         ▼                                 │
            │   Window entity gains  RawHandleWrapper  component        │
            │                         │                                 │
            │   ┌─────────────────────┴──────────────────────┐         │
            │   │ Startup-after-window system (run ONCE,      │         │
            │   │ gated until RawHandleWrapper exists):       │         │
            │   │   1. get_window_handle() → RawWindowHandle  │         │
            │   │   2. match Xlib{window} / Xcb{window}       │         │
            │   │      → XID (u32)                            │         │
            │   │   3. x11rb::connect(None)                   │         │
            │   │   4. intern _NET_WM_WINDOW_TYPE,            │         │
            │   │      _NET_WM_WINDOW_TYPE_DESKTOP            │         │
            │   │   5. change_property32(Replace, XID, ...)   │         │
            │   │   6. (fallback) set _NET_WM_STATE_BELOW,    │         │
            │   │      SKIP_TASKBAR, SKIP_PAGER               │         │
            │   │   7. conn.flush()                           │         │
            │   └─────────────────────┬──────────────────────┘         │
            └─────────────────────────┼────────────────────────────────┘
                                      │  X protocol (socket)
                                      ▼
            ┌─────────────────────────────────────────────────────────┐
            │  X Server (:0)                                            │
            │     window XID now has _NET_WM_WINDOW_TYPE = DESKTOP      │
            └─────────────────────────┬───────────────────────────────┘
                                      │  property-notify
                                      ▼
            ┌─────────────────────────────────────────────────────────┐
            │  KWin (kwin_x11)                                         │
            │   • places DESKTOP-type windows in the bottom            │
            │     "desktop layer"  ── shares layer with plasmashell    │
            │   • Meta+D (_NET_SHOWING_DESKTOP): hides NORMAL windows,  │
            │     DESKTOP windows stay visible  ← reveal mechanism      │
            │   • ⚠ on desktop-click may raise plasmashell over us      │
            └─────────────────────────┬───────────────────────────────┘
                                      │  shares desktop layer with
                                      ▼
            ┌─────────────────────────────────────────────────────────┐
            │  plasmashell (ALSO _NET_WM_WINDOW_TYPE_DESKTOP)          │
            │   draws the Plasma wallpaper → set to "Plain Color"      │
            │   black (per-machine, user OK) so it doesn't fight us     │
            └─────────────────────────────────────────────────────────┘
```

### Recommended Code Structure
```
src/
├── main.rs        # add Window config + WinitSettings + the one-shot desktop-mode system
└── desktop.rs     # NEW: set_desktop_window_type system + x11rb atom helpers + xprop-able state
```

### Pattern 1: One-shot "apply desktop window type after creation"
**What:** A system that runs every Update but no-ops after it has successfully tagged the window. Gate it on the `RawHandleWrapper` component existing (it appears a frame or two after startup) and a `Local<bool>`/resource "done" flag.
**When to use:** Always — Bevy creates the window asynchronously; you cannot set the atom in `Startup` reliably because the handle may not exist yet.
**Example:**
```rust
// src/desktop.rs
// Source: x11rb docs.rs change_property + raw-window-handle 0.6 + bevy 0.19 RawHandleWrapper
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, RawHandleWrapper};
use raw_window_handle::RawWindowHandle;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, AtomEnum, ConnectionExt, PropMode};

#[derive(Resource, Default)]
pub struct DesktopModeApplied(pub bool);

pub fn apply_desktop_window_type(
    mut applied: ResMut<DesktopModeApplied>,
    q: Query<&RawHandleWrapper, With<PrimaryWindow>>,
) {
    if applied.0 {
        return;
    }
    let Ok(wrapper) = q.single() else { return }; // handle not ready yet → retry next frame

    // Extract the X11 window id (XID). On X11 the handle is Xlib or Xcb.
    let xid: u32 = match wrapper.get_window_handle() {
        RawWindowHandle::Xlib(h) => h.window as u32,
        RawWindowHandle::Xcb(h) => h.window.get(),
        other => {
            warn!("not an X11 window handle ({other:?}); desktop mode is X11-only");
            applied.0 = true; // give up cleanly on non-X11
            return;
        }
    };

    if let Err(e) = set_desktop_type(xid) {
        warn!("failed to set desktop window type: {e}");
        // do NOT set applied=true on transient errors if you want retries;
        // for a hard error, set it to avoid spamming.
    } else {
        info!("set _NET_WM_WINDOW_TYPE_DESKTOP on XID {xid:#x}");
        applied.0 = true;
    }
}

fn set_desktop_type(xid: u32) -> Result<(), Box<dyn std::error::Error>> {
    let (conn, _screen) = x11rb::connect(None)?; // fresh connection; XID is server-global

    let wm_type = conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE")?.reply()?.atom;
    let desktop = conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_DESKTOP")?.reply()?.atom;

    // _NET_WM_WINDOW_TYPE is a list of ATOMs (format 32).
    conn.change_property32(
        PropMode::REPLACE,
        xid,
        wm_type,
        AtomEnum::ATOM,
        &[desktop],
    )?;

    // Belt-and-suspenders stacking/skip hints (see spike: may be unnecessary if type sticks,
    // or may be the actual working path if it doesn't).
    let wm_state    = conn.intern_atom(false, b"_NET_WM_STATE")?.reply()?.atom;
    let below       = conn.intern_atom(false, b"_NET_WM_STATE_BELOW")?.reply()?.atom;
    let skip_tb     = conn.intern_atom(false, b"_NET_WM_STATE_SKIP_TASKBAR")?.reply()?.atom;
    let skip_pager  = conn.intern_atom(false, b"_NET_WM_STATE_SKIP_PAGER")?.reply()?.atom;
    conn.change_property32(
        PropMode::REPLACE,
        xid,
        wm_state,
        AtomEnum::ATOM,
        &[below, skip_tb, skip_pager],
    )?;

    conn.flush()?;
    Ok(())
}
```
> ⚠️ **The `change_property32` / `get_window_handle()` / `XcbWindowHandle.window.get()` exact
> signatures must be confirmed at implementation time against the locked crate versions** — they
> are accurate to the documented APIs but `[ASSUMED]` in the exact spelling. `xprop -id <XID>` is
> the ground-truth check.

### Pattern 2: Bevy window + WinitSettings config
**What:** Borderless fullscreen, no decorations, and keep rendering while never focused.
**Example:**
```rust
// src/main.rs — replace DefaultPlugins with a configured WindowPlugin
use bevy::window::{Window, WindowMode, WindowPlugin, MonitorSelection};
use bevy::winit::{WinitSettings, UpdateMode};

App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            decorations: false,
            // BorderlessFullscreen takes a MonitorSelection in Bevy 0.19:
            mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            // Avoid grabbing focus on spawn so we don't fight KWin focus-stealing prevention:
            focused: false,
            ..default()
        }),
        ..default()
    }))
    // A desktop window is *never* focused → without this Bevy may throttle/stop updating.
    .insert_resource(WinitSettings {
        focused_mode: UpdateMode::Continuous,
        unfocused_mode: UpdateMode::Continuous, // smooth orbit even though "unfocused"
    })
    .insert_resource(desktop::DesktopModeApplied::default())
    .add_systems(Update, desktop::apply_desktop_window_type)
    // ...existing setup...
    ;
```
> Power note: if idle battery/CPU matters, `unfocused_mode: UpdateMode::reactive_low_power(...)`
> wakes on input instead of running at full FPS. For a desktop that's usually hidden behind
> windows, that's the kinder default — but verify orbit/search animation stays smooth on
> interaction. `[ASSUMED]` exact `UpdateMode` constructor name for 0.19.

### Anti-Patterns to Avoid
- **Setting the atom in `Startup`:** the `RawHandleWrapper` may not exist yet → query returns empty → silently never tagged. Use an Update system gated on the handle.
- **Assuming KWin reclassifies after map:** EWMH says window type is read at map time. KWin *may* re-read on property change, but this is the #1 thing the spike must prove. If it doesn't, you need unmap→set→remap (which loses `_NET_WM_STATE`, see pitfalls) or a different approach.
- **Manually forcing stacking with `XRaiseWindow`/`XLowerWindow` in a loop:** fights KWin; jittery. Use window *type*/*state*, let the WM stack.
- **Adding autostart here:** that's DESK-03 / Phase 9. Out of scope.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Talking to X11 | Raw socket / hand-rolled XCB | x11rb | Protocol encoding, atom interning, reply handling — all solved, safe, already in tree. |
| Getting the window XID | Parsing `xwininfo` output | `RawHandleWrapper::get_window_handle()` | Direct, typed, no subprocess. |
| Stacking below everything | `XLowerWindow` polling loop | `_NET_WM_WINDOW_TYPE_DESKTOP` (+ `_NET_WM_STATE_BELOW` fallback) | The WM owns stacking; fight it and you get flicker. |
| Reveal on Meta+D | Global hotkey listener that raises the window | Nothing — EWMH exempts DESKTOP windows from "show desktop" | KWin already does this; a hotkey listener would duplicate and conflict (and D2 forbids overlay/hotkey-to-front). |

**Key insight:** Almost everything here is the *window manager's* job. Your code's only real job is to correctly label the window as a desktop and then get out of KWin's way. The hard part isn't writing code — it's discovering whether KWin + plasmashell will cooperate, which is empirical.

## Runtime State Inventory

Not a rename/refactor phase — but there IS per-machine OS/desktop state to touch (DESK-02 mitigation). Documented for the planner:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — verified, no datastore touched by this phase. | none |
| Live service config | **Plasma wallpaper config** (`~/.config/plasma-org.kde.plasma.desktop-appletsrc`): the wallpaper plugin must be set to "Plain Color" (black) per-machine so it doesn't visually fight the app. This is a UI-set config, **not in git**, **per-machine**, requires user OK (and per the user's standing rules, register in sync-control). | Manual via System Settings → Wallpaper → "Plain Color", OR scripted edit of appletsrc (riskier). Reversible. |
| OS-registered state | None this phase. (Autostart = Phase 9.) | none |
| Secrets/env vars | None. | none |
| Build artifacts | New direct deps x11rb/raw-window-handle (already in lock) → `cargo build` regenerates nothing stale. | none |

**Per-machine + reversible + user-OK:** the Plasma wallpaper change matches the user's documented "Plasma per-máquina / confirmar antes de tocar" rule. Plan a `checkpoint:human-verify` before applying it, and a note to register the change in `~/Data/00_System/sync-control`.

## Common Pitfalls

### Pitfall 1: plasmashell covers the app when the user clicks the desktop (THE risk)
**What goes wrong:** A third-party `_NET_WM_WINDOW_TYPE_DESKTOP` window is in the same KWin "desktop layer" as plasmashell. Clicking empty desktop raises plasmashell's wallpaper window over your app → app vanishes (process keeps running).
**Why it happens:** KWin treats plasmashell as *the* desktop and raises it within the desktop layer on activation; documented for conky across KDE bug 348987, conky #145, #2306.
**How to avoid:** (1) Set Plasma wallpaper to Plain Color so even if plasmashell is on top there's nothing to "fight" visually — **but this does NOT make plasmashell transparent**, so if it's above your app you still see black, not your graph. (2) The conky-proven fix is the fallback: use a `normal` window with `below + skip_taskbar + skip_pager + sticky` instead of `desktop` type. (3) **Mitigating factor unique to this app:** your interaction model is "click the 3D graph" not "click the empty desktop" — clicks land on *your* window, which may keep it raised and never trigger the plasmashell raise. This is plausibly why it could work where conky struggles. **Must be proven by the spike.**
**Warning signs:** App visible right after launch, then disappears the first time you click blank space; `xprop -id <XID>` still shows DESKTOP but the window is occluded.

### Pitfall 2: KWin doesn't reclassify after the window is already mapped
**What goes wrong:** You set `_NET_WM_WINDOW_TYPE` after Bevy mapped the window; KWin ignores it because EWMH says type is read at map time.
**Why it happens:** Bevy/winit maps the window on creation; you have no pre-map hook through Bevy 0.19.
**How to avoid:** Spike it first. If KWin honors the late property change → done. If not, options: (a) `conn.unmap_window(xid)` → set property → `conn.map_window(xid)` → re-assert `_NET_WM_STATE` afterward (state is lost on unmap, per wm-spec-list); (b) patch/fork `bevy_winit` to call `with_x11_window_type` pre-creation (heavier, but the "correct" fix); (c) fall back to `normal + below` which *can* be applied late via `_NET_WM_STATE`.
**Warning signs:** `xprop` shows the atom set but the window still has a titlebar/normal stacking.

### Pitfall 3: egui search box can't receive keyboard focus
**What goes wrong:** DESKTOP-type windows are typically non-activatable; KWin may refuse to give keyboard focus, so the egui search field (SEARCH-01, carried from v1) won't accept typing.
**Why it happens:** WMs treat desktop/dock windows as non-focusable by design.
**How to avoid:** Spike: click the window, try typing in the search box. If broken, options: set `_NET_WM_STATE` without making it non-focusable, add an `_NET_WM_USER_TIME`/input hint, or (fallback) use the `normal + below` window which *is* focusable. Mouse picking/orbit is unaffected (pointer events go to the window under the cursor regardless of focus).
**Warning signs:** Orbit/hover works, but typing in search does nothing.

### Pitfall 4: Multi-monitor — BorderlessFullscreen covers one monitor only
**What goes wrong:** `BorderlessFullscreen(MonitorSelection::Current)` fills a single monitor; the user's other monitor keeps the plain Plasma wallpaper.
**Why it happens:** One winit window = one fullscreen monitor.
**How to avoid:** For Phase 6 MVP, single-monitor is fine (spec doesn't require multi-mon). If needed later: a window sized to the bounding box of all monitors, or one window per monitor (out of scope now — flag, don't build).
**Warning signs:** Graph only on the primary screen. The user's setup (Santiago-PC has a TV + main panel per memory) means this *will* be visible — note it but keep MVP single-monitor.

### Pitfall 5: AnyDesk / X11 interaction
**What goes wrong:** Concern that a fullscreen desktop-layer GL app interferes with AnyDesk.
**Why it's likely fine:** AnyDesk on X11 captures the whole X screen; a DESKTOP-type window below everything is just part of the screen content. No special API interaction expected. **Low risk, but list as a spike acceptance check** (success criterion #4 in ROADMAP): start AnyDesk after the app, confirm remote view + input still work.

## Code Examples

### Verify the live window with xprop (the ground-truth check)
```bash
# Find the window and confirm the type stuck:
DISPLAY=:0 xprop -id "$(DISPLAY=:0 xdotool search --class datatree 2>/dev/null | head -1)" \
  _NET_WM_WINDOW_TYPE _NET_WM_STATE
# Expect: _NET_WM_WINDOW_TYPE(ATOM) = _NET_WM_WINDOW_TYPE_DESKTOP
#         _NET_WM_STATE(ATOM) = _NET_WM_STATE_BELOW, _NET_WM_STATE_SKIP_TASKBAR, ...
```
> If `xdotool` isn't installed, click-select with `DISPLAY=:0 xprop _NET_WM_WINDOW_TYPE` and click the window. (See Environment Availability.)

### Spike acceptance script (manual, Wave 0)
```
1. Launch:  DISPLAY=:0 ./target/release/datatree-rs
2. xprop → confirm _NET_WM_WINDOW_TYPE_DESKTOP is set.            [DESK-01]
3. Open Dolphin → it should cover the graph.                      [DESK-01 below-all]
4. Meta+D → graph reappears.                                      [DESK-02 reveal]
5. Click empty desktop area → does the graph survive or vanish?   [Pitfall 1 — KEY]
6. Click the graph + type in the search box → does it focus?      [Pitfall 3]
7. Set Plasma wallpaper Plain Color → no fighting/flicker.        [DESK-02 coexist]
8. Start AnyDesk → remote view + input still work.                [success #4]
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `xwinwrap` / reparent into root window | Native `_NET_WM_WINDOW_TYPE_DESKTOP` via the toolkit | EWMH-era; xwinwrap is a hack | Spec D1 explicitly chose native over xwinwrap. Correct in principle; KDE coexistence is the catch. |
| winit `XWindowType` (≤0.29) | `winit::platform::x11::WindowType` + `with_x11_window_type` (0.30) | winit 0.30 | API exists; just not surfaced by Bevy 0.19. |
| `own_window_type desktop` (conky default) | conky community moved to `own_window_type normal` + below hints on KDE | ongoing KDE bug | Direct evidence the "pure desktop type" path is fragile on KDE. |

**Deprecated/outdated:**
- Reading old conky configs that use `own_window_type desktop` on KDE — they're the *broken* config; the `normal+below` config is the working one.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | KWin will reclassify the window when `_NET_WM_WINDOW_TYPE` is set *after* mapping | Pitfall 2 | High — if false, the whole DESK-01 mechanism needs unmap/remap or a bevy_winit patch. **Spike resolves this.** |
| A2 | The app's "click the graph" model avoids the plasmashell raise-on-click that breaks conky | Pitfall 1 | High — if false, D1 (DESKTOP type) is unworkable and the `normal+below` fallback (contradicting "NO always-below") is required; route to user. **Spike resolves this.** |
| A3 | A DESKTOP-type window can receive keyboard focus for the egui search box on KWin | Pitfall 3 | Medium — if false, search is broken until fallback applied. **Spike resolves this.** |
| A4 | Exact APIs: `RawHandleWrapper::get_window_handle()`, `change_property32`, `XcbWindowHandle.window.get()`, `WindowMode::BorderlessFullscreen(MonitorSelection)`, `UpdateMode` constructor names | Code Examples / Pattern 2 | Low — spellings may differ slightly across 0.19/0.6/0.13; compiler + xprop catch these fast. |
| A5 | A fresh `x11rb::connect(None)` operating on the winit XID is safe & sufficient (no need to reuse winit's display) | Standard Stack | Low — XID is server-global; standard practice. |
| A6 | Setting Plasma wallpaper to "Plain Color" is the realistic coexistence mitigation; plasmashell cannot be made truly transparent for our window to show through | DESK-02 | Medium — if plasmashell ends up *above* our app, Plain Color shows black, not the graph; the real fix is stacking order, not wallpaper color. **Spike clarifies.** |

## Open Questions

1. **Does KWin honor a post-map `_NET_WM_WINDOW_TYPE` change?**
   - Known: EWMH says type is read at map time; KWin sometimes re-reads on property change.
   - Unclear: actual KWin (kwin_x11) behavior on this CachyOS install.
   - Recommendation: Spike step 2; if it fails, try unmap/remap, else fall back.

2. **Does the graph survive a desktop-area click (vs. conky's failure)?**
   - Known: conky's `desktop` type gets covered; our interaction differs.
   - Recommendation: Spike step 5 is the go/no-go for D1. If it fails, surface to user that "NO always-below" can't hold.

3. **Can the egui search box get keyboard focus as a DESKTOP window?**
   - Recommendation: Spike step 6; fallback is `normal+below`.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| X11 session | Whole phase | ✓ | `XDG_SESSION_TYPE=x11` | — (Wayland out of scope) |
| KWin (X11) | Stacking/reveal | ✓ | `/usr/bin/kwin_x11` | — |
| plasmashell | Coexistence target | ✓ | `/usr/bin/plasmashell` | — |
| xprop | Verification | ✓ | 1.2.8 (`/usr/bin/xprop`) | — |
| x11rb crate | Atom setting | ✓ | 0.13.2 (in Cargo.lock) | x11-dl 2.21.0 (also in lock) |
| raw-window-handle crate | XID extraction | ✓ | 0.6.2 (in Cargo.lock) | — |
| xdotool | Convenience in xprop one-liner | ? (not checked) | — | Click-select with bare `xprop` |
| Outbound network | crates.io re-verify | ✗ (sandbox) | — | Lockfile versions are authoritative for the build |

**Missing dependencies with no fallback:** none.
**Missing dependencies with fallback:** `xdotool` (optional) → use interactive `xprop` click-select.

## Security Domain

> `security_enforcement` is not set in `.planning/config.json` (treated as enabled). This phase has
> almost no security surface: a local, single-user desktop app with **no network, no auth, no
> crypto, no untrusted input** in scope. (xdg-open of arbitrary files = Phase 7, not here.)

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | minimal | The only external input is X server replies (trusted local socket). x11rb handles protocol parsing safely. |
| V6 Cryptography | no | — |

### Known Threat Patterns for {Rust + X11 local desktop app}
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed X reply / panic on `.reply()` | DoS (local) | Handle `Result` from x11rb (don't `unwrap`); fail the system gracefully and log. |
| Per-machine Plasma config edit corrupts user desktop | Tampering (user data) | Make the wallpaper change via System Settings UI (reversible) or a backed-up appletsrc edit; gate behind user OK + sync-control register. |

**Bottom line:** security work here is "don't `unwrap()` x11rb results" and "don't clobber the user's Plasma config without a backup and OK." No ASVS-heavy controls needed.

## Sources

### Primary (HIGH confidence)
- `/home/saintwyss/dev/datatree-rs/Cargo.lock` — confirmed winit 0.30.13, bevy_winit 0.19.0, raw-window-handle 0.6.2, x11rb 0.13.2, x11-dl 2.21.0.
- Local env probes — `XDG_SESSION_TYPE=x11`, `kwin_x11`, `plasmashell`, `xprop 1.2.8` all present.
- winit `WindowAttributesExtX11` docs (doc.servo.org / smithay winit docs) — `with_x11_window_type(Vec<WindowType>)`, `with_override_redirect(bool)`; `XWindowType`→`WindowType` rename in 0.30.
- raw-window-handle 0.6.2 `XlibWindowHandle` docs — `window: c_ulong`, `visual_id: c_ulong`.
- bevy 0.19 `RawHandleWrapper` docs — `get_window_handle()`, `get_display_handle()`, is a `Component`.
- bevy 0.19 `bevy_winit` docs — no WindowAttributes customization hook exposed.
- x11rb `change_property` / `intern_atom` / `atom_manager` docs (docs.rs).

### Secondary (MEDIUM confidence)
- conky issues #145, #2306 and KDE bug 348987 — `own_window_type desktop` gets covered by plasmashell on desktop-click; community fix = `normal` + `below,sticky,skip_taskbar,skip_pager`.
- Martin Gräßlin (KWin maintainer) blog "What is a Desktop?" — plasmashell *is* a `_NET_WM_WINDOW_TYPE_DESKTOP` app; "showing the desktop" is a separate `showingDesktop` mode.
- EWMH/Wikipedia — DESKTOP/DOCK windows exempt from "show desktop"; `_NET_WM_STATE` lost on unmap.

### Tertiary (LOW confidence — validate in spike)
- Exact KWin reclassify-after-map behavior on this install.
- Keyboard-focus behavior of DESKTOP-type windows on KWin.

## Metadata

**Confidence breakdown:**
- Standard stack / mechanism to set the atom: HIGH — all crate versions confirmed in lockfile; APIs documented.
- "Bevy can't do it natively, need x11rb escape hatch": HIGH — confirmed bevy_winit exposes no hook.
- Plasma coexistence / stacking / focus: MEDIUM — strong documented evidence of risk (conky), but app-specific outcome is genuinely unknown → spike-gated.
- Post-map reclassify: LOW-MEDIUM — must be empirically tested.

**Research date:** 2026-06-27
**Valid until:** ~2026-07-27 (stable crates; KDE/KWin behavior is the volatile part — re-verify only if Plasma is updated)
