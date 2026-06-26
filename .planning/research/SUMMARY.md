# Research Summary — datatree-rs

**Dominio:** Visualizador 3D nativo (Rust + Bevy) — galaxy-style graph del FS
**Fecha:** 2026-06-26 · **Confianza general:** HIGH (verificado contra crates.io / docs oficiales)

## Stack (versiones verificadas)

| Crate | Versión | Bevy compat | Rol |
|-------|---------|-------------|-----|
| `bevy` | **0.19.0** | — | Engine (latest stable, 2026-06-18) |
| `bevy_panorbit_camera` | **0.35.0** | 0.19 | Cámara orbital (pan/orbit/zoom) |
| `bevy_egui` | **0.40.1** | 0.19 | UI overlay (search bar, sidebar) |
| `open` | **5.3.5** | — | Abrir carpeta en Dolphin via xdg-open |
| `ignore` | 0.4.x | — | FS scan con exclusiones tipo .gitignore |
| `unicode-normalization` | latest | — | Búsqueda accent-insensitive (NFD) |

**Qué NO usar:** `bevy_mod_picking` (archivado 2025-03, picking ya es nativo `MeshPickingPlugin` desde 0.15) · `bevy_mod_billboard` (muerto en 0.14) · `bevy_polyline` (gizmos nativos bastan hasta ~220k líneas) · `petgraph` (overhead para 604 nodos; usar `Vec<FolderNode>` + `parent_idx`) · `Text2D` por nodo (nunca 604 entidades de texto) · `fdg-sim`/`forceatlas2` (AGPL + complejidad; FR manual mejor).

## Arquitectura — 5 módulos

```
src/
├── main.rs        — App::new() + plugins
├── scan.rs        — FS traversal → Vec<FolderNode>
├── layout.rs      — force-directed 3D (background thread)
├── render.rs      — spawn spheres + gizmos edges
├── interaction.rs — picking, double-click, abrir Dolphin
└── ui.rs          — bevy_egui: search bar + node info
```

### Modelo de datos
```rust
pub struct FolderNode {
    pub id: usize,
    pub path: PathBuf,
    pub name: String,
    pub parent_id: Option<usize>,
    pub root_group: usize,   // qué carpeta de ~/Data/* es el ancestro
    pub depth: usize,
    pub child_count: usize,
    pub is_symlink: bool,
    pub pos: Vec3,           // llenado por layout
}
```

### scan.rs
`ignore::WalkBuilder` con `follow_links(false)`. Excluir vía `filter_entry`: node_modules, .git, .cache, Steam, target, __pycache__, .Trash. **Symlinks (01_Now → 20_Knowledge crítico):** resolver manualmente con `canonicalize()` + `HashSet<PathBuf>` de caminos visitados, trackear por inode (`MetadataExt::ino()`) para cortar ciclos. Marcar `is_symlink: true`.

### layout.rs — Force-Directed 3D (galaxia)
Escribir Fruchterman-Reingold custom (~150 líneas). 604 nodos → O(N²) ≈ 110M ops/300 iter < 200ms, sin Barnes-Hut. Constantes: `K=3.0` (distancia ideal), `CLUSTER_K=0.05` (atracción al centroide del cúmulo raíz = el efecto galaxia). Seed inicial separando clusters en el espacio. Correr en `AsyncComputeTaskPool::get().spawn()` al startup (una sola vez, queda fijo); mostrar "Computing layout…" en egui mientras tanto; recibir vía channel/Mutex.

### render.rs
Un solo `Handle<Mesh>` `Sphere::new(0.5)` clonado a los 604 nodos. Un `StandardMaterial{emissive, unlit:true}` por cluster (≤10 draw calls; Bevy 0.16+ batchea solo). Edges: `gizmos.line(a,b, srgba(1,1,1,0.15))` immediate-mode cada frame (≤3000 edges trivial). Cámara: `PanOrbitCameraPlugin` + spawn `(Camera3d, PanOrbitCamera::default())`.

### interaction.rs
`MeshPickingPlugin` en DefaultPlugins. Observers `On<Pointer<Click>>` / `On<Pointer<Over>>` por nodo. Double-click NO existe nativo → implementar con `Local<Option<(Instant, Entity)>>` y umbral 300ms. Abrir: `std::thread::spawn(|| open::that(path))` (no bloquear frame) → xdg-open → Dolphin.

### ui.rs
bevy_egui 0.40.1. **Antes** de procesar picking: `if contexts.ctx_mut().is_pointer_over_area() { return; }`. Búsqueda: normalizar NFD + strip combining marks. Animar cámara al nodo: lerp manual de `PanOrbitCamera.focus` (no hay "animate to" built-in).

## Pitfalls (detección + prevención)

1. **CRÍTICO — `MeshPickingPlugin` fuera de DefaultPlugins:** observers nunca disparan, sin error. → siempre `add_plugins((DefaultPlugins, MeshPickingPlugin))`.
2. **CRÍTICO — Symlink loops:** `follow_links(true)` + 01_Now cuelga el scan. → `follow_links(false)` + canonicalize + HashSet.
3. **CRÍTICO — bevy_egui consume mouse:** click bajo UI hace doble acción. → `is_pointer_over_area()` antes del picking.
4. **MODERADO — Layout bloquea startup:** ventana congelada ~200ms. → AsyncComputeTaskPool.
5. **MODERADO — 604 Text2D labels:** draw calls + ilegible. → labels solo en hover via egui sidebar.
6. **LEVE — handles de mesh distintos:** N draw calls. → un mesh handle clonado, un material por cluster.
7. **LEVE — `Pointer<DoubleClick>` no existe:** → implementar manual con Local+Instant.

## Build Order (alimenta el roadmap)

1. **Scanner** (Rust puro, sin Bevy): `scan.rs` + `FolderNode`, debug print 604 nodos, test symlink 01_Now. `cargo test`.
2. **Render estático**: app Bevy mínima, 604 esferas en posiciones random, edges gizmos, cámara orbital.
3. **Layout** FR en AsyncComputeTaskPool, ajustar constantes hasta ver clusters separados (el "aha" visual).
4. **Interacción**: click→sidebar, hover→highlight emissive, double-click→Dolphin, verificar egui no interfiere.
5. **Buscador**: egui top panel, filtro accent-insensitive, animación de cámara al nodo.
6. **Polish**: tamaño por child_count, opacidad de edges por depth, symlinks visual diferenciado.
