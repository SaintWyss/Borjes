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

---

# v2 "Desktop mode" — 4 fases · 11 requirements · cobertura 100% ✓

### Phase 6: Ventana Desktop X11
**Goal:** La app corre como capa de escritorio (ventana tipo Desktop X11) revelada con Meta+D, conviviendo con Plasma.
**Mode:** mvp
**Requirements:** DESK-01, DESK-02
**Success Criteria**:
1. La ventana se setea `_NET_WM_WINDOW_TYPE_DESKTOP` (winit X11), sin decoración, fullscreen, debajo de todas las ventanas
2. Meta+D (Mostrar escritorio) revela el grafo; abrir una ventana lo tapa de nuevo
3. No tapa ni es tapado por el wallpaper de Plasma (fondo Plasma plano/transparente documentado)
4. AnyDesk/X11 siguen funcionando (no se rompe el flujo)

**Plans:** 3 plans

Plans:
- [ ] 06-01-PLAN.md — Spike Wave 0: x11rb tag _NET_WM_WINDOW_TYPE_DESKTOP + window config + 8-step KDE acceptance (resolves D1 risk)
- [ ] 06-02-PLAN.md — Finalize DESK-01 desktop window on the spike-chosen path (D1-bend checkpoint if fallback needed)
- [ ] 06-03-PLAN.md — DESK-02 Plasma Plain Color coexistence + Meta+D/AnyDesk verify + sync-control register

### Phase 7: Archivos on-demand
**Goal:** Click en carpeta expande sus archivos como nodos hoja en burst local, doble-click abre.
**Mode:** mvp
**Requirements:** FILE-01, FILE-02, FILE-03, FILE-04, INT-01
**Success Criteria**:
1. Click en una carpeta scanea y muestra SUS archivos (un nivel) como nodos hoja; segundo click los colapsa
2. Los archivos aparecen en burst local radial alrededor de su carpeta sin re-acomodar la galaxia
3. Nodos-archivo visualmente distintos de carpetas
4. Doble-click: carpeta→Dolphin, archivo→xdg-open (app default)
**Plans:** 3 plans
Plans:
- [ ] 07-01-PLAN.md — File scan (scan_files) + shared gray cuboid assets + fibonacci burst geometry
- [ ] 07-02-PLAN.md — FileNode/Expanded + Pointer<Click>.count routing + burst spawn/collapse + open + hover
- [ ] 07-03-PLAN.md — handle_rescan file-node teardown + tactile GUI verification

### Phase 8: Pins + persistencia
**Goal:** Pinear nodos por click-derecho, persistidos y siempre visibles.
**Mode:** mvp
**Requirements:** PIN-01, PIN-02, PIN-03
**Success Criteria**:
1. Click-derecho en un nodo abre menú egui con "Pin"/"Unpin"
2. Los pins se guardan en ~/.config/datatree-rs/pins.json y se cargan al arrancar
3. Los pins sobreviven colapso/reescaneo (siempre visibles)
4. Nodos pinneados con marca visual propia (borde/color dorado)
**Plans:** 3 plans
Plans:
- [ ] 08-01-PLAN.md — PinStore + std-only pins.json persistence (atomic/resilient) + gold PinAssets
- [ ] 08-02-PLAN.md — Right-click egui Pin/Unpin menu + toggle persists + live gold mark
- [ ] 08-03-PLAN.md — apply_pins re-mark on load/rescan (survive restart) + tactile verification

### Phase 9: Autostart + integración escritorio
**Goal:** La app arranca sola al login y queda como escritorio del usuario.
**Mode:** mvp
**Requirements:** DESK-03
**Success Criteria**:
1. Entry de autostart KDE creado (per-máquina) que lanza la app en modo desktop al login
2. Registrado en sync-control (ops de sistema) con OK del usuario
3. Tras reboot/login la app aparece como escritorio sin intervención
