# Roadmap — datatree-rs

5 fases · 10 requirements v1 · cobertura 100% ✓
Stack: Rust + Bevy 0.19 · granularity: coarse · mode: mvp

### Phase 1: Scanner (núcleo Rust, sin Bevy)
**Goal:** Recorrer `~/Data` y producir `Vec<FolderNode>` correcto, con symlinks resueltos sin loops.
**Mode:** mvp
**Requirements:** SCAN-01, SCAN-02
**Plans:** 1 plan
**Success Criteria**:
1. `cargo test` valida que escanea ~/Data y excluye node_modules/.git/Steam/cache
2. Imprime ~4600 nodos con su root_group y depth correctos (el ~604 inicial era un preview capeado a depth 5; el árbol real a profundidad completa ronda 4607)
3. `01_Now` (symlink a 20_Knowledge) aparece una vez, marcado is_symlink, sin colgarse
4. Cada nodo tiene parent_id, child_count y root_group bien asignados

Plans:
- [ ] 01-01-PLAN.md — Cargo scaffold + FolderNode struct + scan_folder con exclusiones, symlinks y test suite

### Phase 2: Render estático + cámara
**Goal:** App Bevy que dibuja los nodos como esferas y los edges, navegable con cámara orbital.
**Mode:** mvp
**Requirements:** GRAPH-01, CAM-01
**Plans:** 1 plan
**Success Criteria**:
1. La ventana abre y muestra ~4600 esferas (posiciones random por ahora) + edges padre→hijo
2. Cámara orbital funciona: rotar, zoom, pan con mouse
3. Corre fluido (≥60 fps) con MeshPickingPlugin ya cargado
4. Un solo mesh handle compartido + material por cluster (≤10 draw calls de nodos)

Plans:
- [ ] 02-01-PLAN.md — Cargo deps (bevy/panorbit/rand) + render.rs (spawn esferas mesh compartido + material por cluster + edges gizmos) + main.rs (App con MeshPickingPlugin + cámara orbital)

### Phase 3: Layout galaxia (force-directed)
**Goal:** Las posiciones random se reemplazan por un layout que separa visualmente cada área-raíz en su cúmulo.
**Mode:** mvp
**Requirements:** GRAPH-02
**Success Criteria**:
1. Fruchterman-Reingold + cluster-pull corre en AsyncComputeTaskPool al startup (no congela la ventana)
2. Se ve "Computing layout…" mientras calcula y luego se acomoda
3. Los cúmulos por carpeta-raíz quedan claramente separados en el espacio (efecto galaxia)
4. Layout estable para ~4600 nodos (OJO PERF: a 4607 nodos, Fruchterman-Reingold O(N²) = ~21M pares/iter × 300 ≈ 6.3B ops/pasada, ya NO "trivial" como asumió el research para 604 → evaluar Barnes-Hut o reducir iteraciones; sigue siendo cálculo único en AsyncComputeTaskPool)

**Plans:** 1 plan

Plans:
- [ ] 03-01-PLAN.md — layout.rs run_layout (FR O(N²) + cluster-pull + seed de centroides) + test de clustering; integración AsyncComputeTaskPool one-shot + indicador "Computing layout…" + aplicar posiciones por NodeRef

### Phase 4: Interacción + búsqueda
**Goal:** Llegar a una carpeta: buscarla, centrarla y abrirla en Dolphin.
**Mode:** mvp
**Requirements:** OPEN-01, SEARCH-01
**Success Criteria**:
1. Hover resalta el nodo (emissive); click lo selecciona y muestra su ruta
2. Doble-click abre la carpeta en Dolphin (xdg-open en thread), sin bloquear el frame
3. Buscador egui (top panel) filtra acento-insensible y la cámara se anima hacia el nodo
4. egui no dispara picking por debajo (`is_pointer_over_area` chequeado)

**Plans:** 2 plans

Plans:
- [ ] 04-01-PLAN.md — interaction.rs: hover highlight + click selecciona/loggea ruta + doble-click abre Dolphin en thread (open 5.3.5) + guard is_pointer_over_area; deps egui/open/unicode-normalization
- [ ] 04-02-PLAN.md — ui.rs: panel egui buscador + normalize() acento-insensible (unit test) + resaltar matches + SearchTarget + lerp de cámara al primer match

### Phase 5: Lectura visual + raíz configurable (polish)
**Goal:** El grafo comunica de un vistazo y se puede apuntar a otra raíz.
**Mode:** mvp
**Requirements:** VIS-01, VIS-02, ROOT-01
**Success Criteria**:
1. Color de nodo por área raíz (paleta por cluster)
2. Tamaño de nodo escala con child_count
3. Symlinks con marca visual diferenciada
4. Se puede cambiar la raíz escaneada desde la app y el grafo se reconstruye (default ~/Data)

**Plans:** 2 plans

Plans:
- [ ] 05-01-PLAN.md — VIS-02 tamaño por child_count (node_scale) + symlinks material distinto + revisión paleta VIS-01 (render.rs)
- [ ] 05-02-PLAN.md — ROOT-01 raíz configurable: input + botón Reescanear (egui) + reconstrucción async (despawn viejos → re-scan → re-layout) + ruta inválida sin panic (main.rs, ui.rs)
