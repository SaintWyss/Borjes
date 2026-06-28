# Phase 7: Archivos on-demand - Research

**Researched:** 2026-06-28
**Domain:** Bevy 0.19 ECS — picking/observers, on-demand mesh spawning, local FS scan
**Confidence:** HIGH (all claims verified against the locked bevy_picking 0.19.0 source in the cargo registry and against this repo's existing working code)

## Summary

Phase 7 adds file leaf nodes that burst out of a folder on click. Every building block already exists in the codebase and in the locked stack — **no new dependencies are needed**. The work is: (1) a one-level lazy file scan, (2) spawning a radial burst of distinct-looking file meshes around the parent folder's existing position without touching `run_layout`, (3) click/double-click disambiguation, and (4) collapse via despawn.

The single most important discovery: **Bevy 0.19's `Pointer<Click>` event carries a native `count: u8` field** ("number of consecutive clicks, starting at 1") computed against `PickingSettings::multi_click_interval` (default 500ms, configurable). This means the manual `Local<Option<(Instant, Entity)>>` 300ms timer in `interaction.rs` can be replaced by reading `trigger.count` — `count == 1` is a single click, `count >= 2` is a double click. `[VERIFIED: bevy_picking-0.19.0/src/events.rs:311-320, lib.rs:354]`

**Primary recommendation:** Single-click toggles the file burst *immediately* (no timer lag); double-click opens (Dolphin for folders, `open::that` / xdg-open for files). On a real double-click, `count==1` fires first (burst expands instantly) then `count==2` fires (opens) — the burst simply stays open, which is the desired "you opened it, you're working in it" UX. File nodes are spawned as **absolute-positioned siblings** (not Transform children) tagged with a `FileNode { parent: Entity }` component, using **cached shared mesh+material handles** to keep draw calls flat. They carry **no `NodeRef`**, so they are automatically invisible to the existing search/click-folder/edge systems.

## Locked Decisions (from design doc D4–D9 + REQUIREMENTS FILE/INT)

No `CONTEXT.md` exists for this phase yet (standalone research). The following decisions from `docs/superpowers/specs/2026-06-27-desktop-mode-design.md` and `.planning/REQUIREMENTS.md` are already locked and constrain the plan exactly like CONTEXT decisions:

- **D4 / FILE-01:** Click on a folder = toggle: lazy-scan that dir **one level**, show/hide its files as leaf nodes. Second click collapses.
- **D5 / INT-01:** Click = expand (toggle files); double-click = open. Folder double-click → Dolphin; file double-click → `xdg-open` default app. Must coexist with v1's double-click-to-open-folder.
- **D6 / FILE-02:** File node is smaller, distinct shape/color (design suggests grey / diamond) to differ from folders.
- **D8 / FILE-04:** Expansion is a **local radial burst** around the parent folder's current position; the rest of the galaxy stays fixed. **Do NOT re-run `run_layout`.**
- **D9:** On-demand keeps node count low → no Barnes-Hut / culling needed yet.

Out of scope for this phase (do NOT build): pins/right-click menu (PIN-01..03, phase 8), autostart (DESK-03, phase 9), live FS watch (deferred), making files searchable (see Open Questions — flagged NOT in v1).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| One-level file listing | `scan.rs` (FS layer) | — | New `scan_files(dir)` lives beside `scan_folder`; pure FS, no ECS |
| File node spawn + burst placement | `interaction.rs` (on click) + `render.rs` (asset handles) | — | Spawn must react to a click, so it lives in the click observer; shared mesh/material handles are a render concern |
| Radial burst math | `render.rs` or new `burst.rs` helper | `layout.rs` (reuse fibonacci-sphere) | Pure geometry, no force simulation; must NOT enter `run_layout` |
| Click vs double-click routing | `interaction.rs` (observer) | — | Uses native `Pointer<Click>.count` |
| Open file | `interaction.rs` (`open::that` thread) | — | Identical pattern to existing folder open |
| Collapse / expanded state | ECS components (`Expanded`, `FileNode`) | — | Query+despawn, no resource needed |

## Standard Stack

No new crates. Everything required is already a dependency.

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| bevy | 0.19.0 | ECS, picking (`Pointer<Click>` w/ `count`), Mesh3d spawn | Locked; native picking already drives folder nodes `[VERIFIED: Cargo.lock]` |
| open | 5.3.5 | `open::that(path)` → xdg-open default app | Already opens folders in Dolphin (OPEN-01) `[VERIFIED: Cargo.toml, interaction.rs:84]` |
| std::fs | stdlib | `read_dir` one-level file listing | No crate needed for a single-dir, one-level scan |

### Supporting / already present
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| ignore | 0.4 | `WalkBuilder` recursive folder scan | Used by `scan_folder`; **not** reused for the file scan (overkill for one level) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `std::fs::read_dir` for files | `ignore::WalkBuilder` w/ `max_depth(1)` | WalkBuilder honors `.gitignore`/hidden config (consistent w/ folders) but is heavier and async-unfriendly for a per-click scan. For one level, `read_dir` is simpler and fast. Recommend `read_dir` + manual hidden/excluded filter to mirror folder rules. |
| Native `Click.count` | Keep the `Local<Instant>` 300ms timer | The timer works but re-implements what Bevy now gives for free, and 300ms < the 500ms default window can cause missed double-clicks. Recommend migrating to `count`. |
| File nodes as `ChildOf` parent | Absolute sibling + `FileNode{parent}` | ChildOf inherits the **parent folder's scale** (`node_scale` 0.5–3.0), distorting burst radius and file size. Galaxy is static post-layout, so hierarchy buys nothing. Recommend siblings. |

**Installation:** none — `cargo build --release` with the current `Cargo.toml`.

## Package Legitimacy Audit

No external packages are added in this phase. All functionality uses already-vendored, already-audited dependencies (`bevy`, `open`, `ignore`) plus the Rust standard library. Package Legitimacy Gate: **N/A — zero new packages.**

## Architecture Patterns

### System Architecture Diagram

```
                        ┌─────────────────────────────────────┐
   user left-click ───► │ on_node_click (global observer)      │
   on a Mesh3d entity   │  guard: is_pointer_over_egui? ──► no-op
                        │  guard: trigger.button==Primary?     │
                        └───────────────┬──────────────────────┘
                                        │ branch on component
                 ┌──────────────────────┴───────────────────────┐
                 │ entity has NodeRef (FOLDER)                   │ entity has FileNode (FILE)
                 ▼                                               ▼
        trigger.count == 1 ?                            trigger.count == 1 ?
         │            │                                  │            │
        yes          no(>=2)                            yes          no(>=2)
         │            │                                  │            │
   toggle burst    open folder                       (select/      open file
   (expand or       in Dolphin                        nothing)      open::that(path)
    collapse)      open::that(dir)                                  in thread
         │            (thread)
         ▼
  ┌──────────────────────────────┐
  │ EXPAND: scan_files(dir) ─────► one-level read_dir, filter dirs/hidden/excluded
  │   for each file:             │
  │     pos = parent.translation + fibonacci_offset(i,N) * BURST_RADIUS
  │     spawn(Mesh3d(file_mesh), MeshMaterial3d(file_mat),
  │            OriginalMaterial(file_mat), Transform(pos, file_scale),
  │            FileNode{ parent })                                  │
  │   insert Expanded marker on folder entity                      │
  ├──────────────────────────────┤
  │ COLLAPSE: query FileNode, despawn where .parent == folder      │
  │   remove Expanded marker                                       │
  └──────────────────────────────┘

  Hover (unchanged path, query widened): on_node_over/out highlight
  works for BOTH NodeRef and FileNode entities via OriginalMaterial.

  NEVER touched on expand: run_layout, GraphData, draw_edges (folders only).
```

File-to-implementation mapping is in the Architectural Responsibility Map above.

### Recommended structure (surgical — extend existing files)
```
src/
├── scan.rs         # + pub struct FileEntry { path, name }  + fn scan_files(dir) -> Vec<FileEntry>
├── render.rs       # + FileAssets resource (cached mesh+material) + fibonacci_offset() helper
├── interaction.rs  # + FileNode, Expanded components; extend on_node_click; new on_file open path; widen hover queries
└── main.rs         # register FileAssets at startup; add file-node despawn to handle_rescan; register any new observer
```

### Pattern 1: Native double-click via `Pointer<Click>.count`
**What:** Read the consecutive-click counter instead of timing clicks manually.
**When to use:** All click routing in this phase (replaces the `Local<Instant>` timer).
```rust
// Source: bevy_picking-0.19.0/src/events.rs:311-320 (Click { button, hit, duration, count })
//         bevy_picking-0.19.0/src/lib.rs:354 (multi_click_interval default 500ms)
pub fn on_node_click(
    trigger: On<Pointer<Click>>,
    mut contexts: EguiContexts,
    // ... Commands, Assets, queries ...
) -> Result {
    if contexts.ctx_mut()?.is_pointer_over_egui() { return Ok(()); }
    if trigger.button != PointerButton::Primary { return Ok(()); } // reserve Secondary for pins (phase 8)

    let entity = trigger.entity;
    if trigger.count >= 2 {
        // double click → open (folder: Dolphin, file: default app)
    } else {
        // single click → toggle file burst (folders only)
    }
    Ok(())
}
```
Note: `trigger` derefs to the `Click` event, so `trigger.count` / `trigger.button` work directly (same deref the current code uses for `trigger.entity`).

### Pattern 2: Radial burst placement (reuse the layout fibonacci-sphere)
**What:** Even 3D distribution of N points on a sphere around the parent. The exact same golden-angle fibonacci sphere already lives in `layout.rs::seed_clustered_positions` (lines 80–104).
**When to use:** Positioning the file burst (FILE-04). Pure geometry — never feed these into `run_layout`.
```rust
// Source: derived from layout.rs:80-104 (already in this codebase)
const BURST_RADIUS: f32 = 4.0; // tune: large enough to clear the folder's node_scale (max 3.0)
fn fibonacci_offset(i: usize, n: usize) -> Vec3 {
    let golden = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
    let y = if n <= 1 { 0.0 } else { 1.0 - (i as f32 / (n - 1) as f32) * 2.0 };
    let r = (1.0 - y * y).max(0.0).sqrt();
    let theta = i as f32 * golden;
    Vec3::new(theta.cos() * r, y, theta.sin() * r)
}
// spawn position:  parent_transform.translation + fibonacci_offset(i, n) * BURST_RADIUS
```
For a single file (`n == 1`), this places it directly "above" the parent (offset `(0,0,0)` * radius → at center); special-case `n==1` to offset by e.g. `Vec3::Y * BURST_RADIUS` so it doesn't overlap the folder.

### Pattern 3: Cached shared file assets (flat draw calls)
**What:** Create the file mesh + material ONCE at startup, store handles in a resource, clone the handles on every spawn. Mirrors `spawn_graph_nodes`'s shared-`sphere`/per-cluster-material pattern (render.rs:55–83) so files render in the same batched draw call.
```rust
#[derive(Resource)]
pub struct FileAssets { pub mesh: Handle<Mesh>, pub material: Handle<StandardMaterial> }
// at Startup (after Assets exist):
let mesh = meshes.add(Cuboid::new(0.4, 0.4, 0.4));      // distinct shape (cube vs sphere) — FILE-02
let grey = Color::srgb(0.6, 0.6, 0.62);
let material = materials.add(StandardMaterial { base_color: grey, emissive: grey.to_linear(), unlit: true, ..default() });
commands.insert_resource(FileAssets { mesh, material });
```
File scale should be smaller than `MIN_NODE_SCALE` (0.5) — e.g. `Vec3::splat(0.35)` — so files read as "smaller than any folder."

### Pattern 4: Collapse via component query + despawn
```rust
#[derive(Component)] pub struct FileNode { pub parent: Entity }
#[derive(Component)] pub struct Expanded; // marker on the folder entity

// collapse:
for (file_entity, file) in &file_query {            // Query<(Entity, &FileNode)>
    if file.parent == folder_entity { commands.entity(file_entity).despawn(); }
}
commands.entity(folder_entity).remove::<Expanded>();
```

### Anti-Patterns to Avoid
- **Feeding file nodes into `run_layout`:** it is O(N²) over the whole graph and would move all ~4600 folders. Files are positioned by direct geometry only.
- **Giving file nodes a `NodeRef`:** `NodeRef(usize)` indexes `GraphData.0`; files aren't in `GraphData`, so any handler doing `graph.0.get(node_ref.0)` would mis-index or silently drop them. Files use `FileNode` instead.
- **Parenting files to the folder via `ChildOf`:** inherits the folder's 0.5–3.0 scale, distorting the burst. Use absolute sibling transforms.
- **Re-creating mesh/material handles inside the click observer:** churns the asset store and breaks batching. Cache in `FileAssets`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Double-click detection | Manual `Instant` timer per entity | `Pointer<Click>.count` | Bevy tracks consecutive clicks per entity against `multi_click_interval`; handles per-entity reset, the 300ms-vs-500ms window bug, and pointer identity `[VERIFIED: events.rs:933-982]` |
| Even point distribution on a sphere | Random jitter / hand-tuned rings | Golden-angle fibonacci sphere already in `layout.rs:80-104` | Deterministic, even, no clumping; reuse the proven helper |
| Opening files with the right app | Mime lookup / hardcoded apps | `open::that(path)` (xdg-open) | Already used for folders → Dolphin; for files xdg-open resolves the user's default per type |
| Picking/raycast on new meshes | Manual ray-AABB test | `MeshPickingPlugin` (already added) | Any `Mesh3d` entity is auto-pickable — folder nodes prove it (they have no explicit `Pickable` component) `[VERIFIED: main.rs:48 + render.rs:85-92 work today]` |

**Key insight:** This phase is almost entirely *composition* of patterns already in the repo (shared-handle spawn, fibonacci sphere, `open::that` thread, picking observers). The only genuinely new code is `scan_files` (a ~15-line `read_dir` loop).

## Runtime State Inventory

> Not a rename/migration phase, but a brief inventory of ECS/runtime state this phase introduces and must clean up:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — file nodes are ephemeral ECS entities, nothing persisted (D11: only pins.json later) | None |
| Live ECS state | New `FileNode`/`Expanded` entities + components; `FileAssets` resource | `handle_rescan` (main.rs:138) currently despawns only `With<NodeRef>` — **must also despawn `With<FileNode>` and clear `Expanded`**, or stale files survive a reroot |
| OS-registered state | None | None |
| Secrets/env vars | None | None |
| Build artifacts | None — no dependency or `Cargo.toml` change | None |

**The canonical cleanup question:** after a "Reescanear" (reroot), all folder entities are despawned and re-spawned — file burst entities (no `NodeRef`) would be orphaned. The plan must add file-node teardown to `handle_rescan`.

## Common Pitfalls

### Pitfall 1: Rescan orphans file nodes
**What goes wrong:** `handle_rescan` despawns `Query<Entity, With<NodeRef>>` only. File nodes lack `NodeRef`, so after reroot they float with no parent.
**Why:** Files are intentionally excluded from `NodeRef`.
**How to avoid:** Add `Query<Entity, With<FileNode>>` despawn to `handle_rescan` and clear any `Expanded` markers.
**Warning signs:** Grey cubes persist after clicking "Reescanear."

### Pitfall 2: Hover stops working on file nodes
**What goes wrong:** `on_node_over`/`on_node_out` filter `With<NodeRef>` (interaction.rs:21,39). File nodes won't highlight.
**Why:** Existing filter is folder-specific.
**How to avoid:** Widen the filter to `Or<(With<NodeRef>, With<FileNode>)>`. Both `on_node_over` (needs only `MeshMaterial3d`) and `on_node_out` (needs `OriginalMaterial` + optional `SearchMatch`) work unchanged if file nodes also carry `OriginalMaterial(file_material)`. `SearchMatch` is `Option`, so its absence on files is fine.
**Warning signs:** Files don't light up on mouseover; folders still do.

### Pitfall 3: Double-click leaves a flickering / contradictory state
**What goes wrong:** Naive "single-click toggles" means a double-click does expand→collapse, or expand then open while collapsing.
**Why:** A double-click is physically two clicks; both `count==1` and `count==2` events fire.
**How to avoid:** On `count >= 2`, do **only** the open — do not toggle. Net result of a folder double-click: `count==1` expands (instant), `count==2` opens Dolphin and leaves the burst expanded. No collapse, no flicker. Single-click (no follow-up) just expands. This is the least-janky resolution and needs no delay. (See Open Questions Q1 for the alternative if the user dislikes leftover bursts.)
**Warning signs:** Burst blinks open/closed when opening a folder.

### Pitfall 4: Burst radius too small vs. parent node scale
**What goes wrong:** Folders scale up to `MAX_NODE_SCALE = 3.0` (render.rs:16). A `BURST_RADIUS` of 1–2 puts files inside the folder sphere.
**Why:** Folder radius = 0.5 mesh * up-to-3.0 scale = up to 1.5 world units.
**How to avoid:** `BURST_RADIUS >= ~4.0`; optionally scale it by the parent's `node_scale`.
**Warning signs:** Files visually buried in the folder.

### Pitfall 5: Right-click triggers expand
**What goes wrong:** `Click` fires for any `PointerButton`. Current `on_node_click` doesn't check `button`, so right-click also toggles — and phase 8 wants right-click for pins.
**How to avoid:** Gate on `trigger.button == PointerButton::Primary` now, so phase 8's `Secondary` handler won't conflict. `[VERIFIED: events.rs:312 button field; pointer.rs:161 PointerButton::{Primary,Secondary,Middle}]`

### Pitfall 6: Large directory hitches the frame
**What goes wrong:** `scan_files` runs synchronously inside the click observer; a folder with thousands of files blocks one frame and spawns thousands of meshes.
**Why:** No async/LOD in this phase (D9 assumes low counts).
**How to avoid:** For `~/Data` this is fine (folders hold tens of files). If a folder is huge, cap the burst (e.g. first N files) or move the scan to `AsyncComputeTaskPool` like `start_layout_task` (main.rs:109). Recommend: ship synchronous + a soft cap (e.g. 200 files), flag uncapped as future work.
**Warning signs:** Visible stutter expanding a media/download folder.

## Code Examples

### One-level file scan (new, in scan.rs)
```rust
// Source: std::fs::read_dir — mirrors scan_folder's hidden/EXCLUDED policy (scan.rs:9-17,53)
pub struct FileEntry { pub path: PathBuf, pub name: String }

pub fn scan_files(dir: &Path) -> Vec<FileEntry> {
    let Ok(read) = std::fs::read_dir(dir) else { return Vec::new() };
    read.filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false)) // files only, one level
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_str().unwrap_or("");
            !name.starts_with('.')                       // skip dotfiles (matches hidden(true))
                && !EXCLUDED.contains(&name)             // cheap consistency w/ folder rules
        })
        .map(|e| FileEntry { name: e.file_name().to_str().unwrap_or("").to_string(), path: e.path() })
        .collect()
}
```
File-node data needed: `path` (for open), `name` (future labels/search), and the spawning code supplies `parent: Entity`. No `child_count`, no `root_group`, no layout `pos` (computed at spawn). Symlinked files: `read_dir` + `is_file()` follows the link's type via `file_type()`; keeping them is fine and simplest. Recommend NOT special-casing symlinked files in v1.

### Open file (reuse existing folder pattern)
```rust
// Source: interaction.rs:82-86 (identical to folder open)
let path = file.path.clone();
std::thread::spawn(move || { let _ = open::that(path); });
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual `Instant`-based double-click timing | `Pointer<Click>.count` + `multi_click_interval` | Present in bevy_picking 0.19 | Delete the `Local<Option<(Instant,Entity)>>` timer; cleaner, matches OS double-click window |
| `despawn_recursive()` vs `despawn()` | `despawn()` is recursive on children by default in modern Bevy | Bevy 0.16+ | File nodes are siblings (no children), so per-entity `despawn()` is exact; folder despawn won't accidentally take files (they aren't children) |

**Deprecated/outdated:** none affecting this phase. (Note `egui::TopBottomPanel::show` deprecation is already handled with an `#[allow(deprecated)]` in ui.rs:65 and is unrelated.)

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `~/Data` folders hold few enough files that a synchronous `read_dir` + spawn doesn't hitch | Pitfall 6 | A media folder could stutter on expand; mitigated by a soft cap |
| A2 | Reusing `EXCLUDED` (dir names) as a file filter is harmless (rarely matches files) | scan_files | Negligible — at worst a file literally named `target` is hidden |
| A3 | User accepts that a folder double-click leaves its file burst expanded | Pitfall 3, Q1 | If disliked, switch to "double-click collapses then opens" (one extra line) |
| A4 | Files are NOT searchable in v1 (search indexes `GraphData`/`NodeRef` only) | Open Questions Q5 | If user expects to search files, a separate index is needed (out of this phase's scope) |

## Open Questions

1. **Folder double-click leaves files expanded — acceptable?**
   - What we know: `count==1` expands instantly, `count==2` opens; suppressing the toggle on `count>=2` leaves the burst open (recommended, no flicker).
   - What's unclear: whether the user prefers a clean folder (auto-collapse on open).
   - Recommendation: ship "leave expanded" (matches "you're working in it"); trivially switchable. Surface in `/gsd:discuss-phase`.

2. **Visual encoding specifics (FILE-02 / D6).**
   - What we know: smaller + distinct shape/color reads as "file." D6 suggests grey/diamond.
   - What's unclear: cube vs. small sphere vs. octahedron ("diamond"); flat grey vs. color-by-extension.
   - Recommendation: v1 = grey `Cuboid` at scale ~0.35 (cheapest distinct read, one shared material). Color-by-extension is a later polish; flag, don't build now.

3. **Edges from folder to its files?**
   - What we know: `draw_edges` (render.rs:96) draws only `GraphData` parent links; files aren't in `GraphData`.
   - Recommendation: optional. A small gizmo system iterating `FileNode` + parent `Transform` can draw faint lines. Treat as polish, not a requirement (FILE-01..04 don't mention edges).

4. **Soft cap on files per burst?**
   - Recommendation: cap at ~200 with a note, to keep D9's "low count" guarantee even for a stray huge folder. Confirm the number in discuss.

5. **Should files be searchable / selectable?**
   - What we know: search (`ui.rs`) and `Selected` index `GraphData`. Files are outside it by design.
   - Recommendation: **No** in v1 (matches REQUIREMENTS "Mostrar archivos como nodos hoja" being a focused feature, and the deferred LOD note). Clicking a file should not write `Selected.0` (which is a folder index). Document as intentional.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| xdg-open (via `open` crate) | FILE-03 open file | ✓ | open 5.3.5 | — (already opens folders in Dolphin, OPEN-01 validated) |
| MeshPickingPlugin | picking file nodes | ✓ | bevy 0.19 | — (folder picking proves it works) |
| Filesystem read access to `~/Data` | FILE-01 scan | ✓ | — | — (scan_folder already reads it) |

**Missing dependencies with no fallback:** none.
**Missing dependencies with fallback:** none.

## Security Domain

Local single-user native desktop app; no network, no auth, no untrusted external input. The only security-relevant surface:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes (mild) | Paths come from the local FS (`read_dir`), not user text; `open::that` receives a `PathBuf` from a real dir entry. No shell string interpolation. |
| V6 Cryptography | no | — |
| V2/V3/V4 Auth/Session/Access | no | — |

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| `open::that` on an arbitrary file launches its default app (a `.desktop`/script could execute) | Elevation/Tampering | Files come only from the user's own `~/Data` scan, opened by explicit double-click — same trust level as the file manager. No new exposure beyond v1's folder-open. No mitigation needed beyond not auto-opening. |

No new security controls required for this phase.

## Sources

### Primary (HIGH confidence)
- `bevy_picking-0.19.0/src/events.rs` (extracted from cargo cache) — `Click { button, hit, duration, count }` struct (L311-320); click-count computation against `multi_click_interval` (L913-985)
- `bevy_picking-0.19.0/src/lib.rs` — `PickingSettings::multi_click_interval` default `Duration::from_millis(500)` (L354), configurable
- `bevy_picking-0.19.0/src/pointer.rs` — `PointerButton::{Primary, Secondary, Middle}` (L161)
- Repo source (working code): `src/interaction.rs`, `src/render.rs`, `src/scan.rs`, `src/main.rs`, `src/layout.rs`, `src/ui.rs`, `src/desktop.rs`
- `Cargo.toml` / `Cargo.lock` — bevy 0.19.0, bevy_picking 0.19.0, open 5.3.5, ignore 0.4
- `docs/superpowers/specs/2026-06-27-desktop-mode-design.md` — D4–D9 locked decisions
- `.planning/REQUIREMENTS.md` — FILE-01..04, INT-01

### Secondary / Tertiary
- None needed — all claims verified against locked source or running code.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new packages; everything verified in Cargo.lock + working code
- Picking / double-click API: HIGH — read directly from the locked bevy_picking 0.19.0 source
- Architecture (burst, components, cleanup): HIGH — composed from existing repo patterns
- Visual encoding specifics: MEDIUM — exact shape/color is a design choice (Open Q2)

**Research date:** 2026-06-28
**Valid until:** stable until the bevy/bevy_egui/bevy_panorbit_camera versions in Cargo.toml change (locked stack). ~60 days.

## RESEARCH COMPLETE

**Phase:** 7 - Archivos on-demand
**Confidence:** HIGH

### Key Findings
- **No new dependencies.** Entire phase composes existing repo patterns + stdlib `read_dir`.
- **Native double-click:** `Pointer<Click>.count` (500ms `multi_click_interval`, configurable) replaces the manual 300ms `Local<Instant>` timer — cleaner INT-01, no laggy delay. `count==1` → toggle burst (instant); `count>=2` → open.
- **File nodes = absolute siblings with `FileNode{parent}` + `Expanded` marker, NO `NodeRef`.** This auto-excludes them from search/edges/Selected and avoids `GraphData` mis-indexing; ChildOf is rejected (inherits folder scale).
- **Burst math already exists** as the golden-angle fibonacci sphere in `layout.rs:80-104`; reuse it with `BURST_RADIUS >= 4.0`. Files never enter `run_layout`.
- **Three must-do cleanups/guards:** (1) add `With<FileNode>` despawn to `handle_rescan`; (2) widen hover queries to `Or<(With<NodeRef>, With<FileNode>)>`; (3) gate clicks on `PointerButton::Primary` to keep phase-8 right-click pins conflict-free.

### File Created
`/home/saintwyss/dev/datatree-rs/.planning/phases/07-archivos-on-demand/07-RESEARCH.md`

### Confidence Assessment
| Area | Level | Reason |
|------|-------|--------|
| Standard Stack | HIGH | Zero new packages; verified Cargo.lock + working code |
| Architecture | HIGH | Composed from existing, proven repo patterns + locked picking source |
| Pitfalls | HIGH | Each traced to a specific line in the current codebase |

### Open Questions (for /gsd:discuss-phase)
- Folder double-click: leave burst expanded (recommended) or auto-collapse?
- File visual: grey cuboid (recommended) — confirm shape/color; color-by-extension deferred?
- Soft cap on files per burst (~200)?
- Confirm files are NOT searchable in v1 (recommended).

### Ready for Planning
Research complete. Planner can create PLAN.md tasks: (1) `scan_files` in scan.rs, (2) `FileAssets` + fibonacci helper in render.rs, (3) `FileNode`/`Expanded` + extended `on_node_click` + file open + widened hover in interaction.rs, (4) `handle_rescan` file-node teardown + startup registration in main.rs.
