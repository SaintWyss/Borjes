# 03 — Arquitectura y decisiones de ingeniería

## Explicación simple

Yo construí Borjes como una aplicación nativa en Rust sobre Bevy. Bevy aporta un motor basado en ECS: los datos viven en componentes y los sistemas procesan esos componentes.

Esto es diferente de una GUI clásica donde cada botón o árbol administra su propio estado. En Borjes, navegación, input, nodos, cámara, búsqueda y render se coordinan mediante recursos, eventos y sistemas.

## Arquitectura general

| Área | Responsabilidad |
|---|---|
| bootstrap/main | configura app, plugins, recursos y sistemas |
| scanner/root | descubre la estructura de archivos |
| navegación | cambia carpeta central y breadcrumb |
| overlay/drill | experiencia visual principal |
| búsqueda | consulta global y selección de resultados |
| pins | accesos persistentes |
| icons | resolución de iconos y fallback |
| lens | filtros y vistas temáticas |
| frecency | relevancia por uso/recencia |
| desktop | integración con sistema, foco y apertura externa |
| assets | fuentes, texturas e iconografía |

## Por qué esta arquitectura es adecuada

### Render visual y animaciones

Yo elegí Bevy porque el producto necesita posicionar, animar y actualizar nodos como una escena, no solo dibujar una lista jerárquica.

### Rendimiento y seguridad de memoria

Rust evita garbage collector y muchas clases de errores de memoria. No garantiza rendimiento automáticamente, pero permite control preciso.

### ECS

El ECS desacopla datos de comportamiento. Un nodo puede adquirir componentes de selección, animación o relevancia sin crear una jerarquía de clases profunda.

### Aplicación nativa

Yo no necesito un servidor ni un navegador para explorar archivos locales. Una aplicación nativa puede integrar atajos, foco, apertura y GPU directamente.

## Decisiones principales

### D-01 — Rust

**Yo elegí:** Rust estable.

**Porque:** seguridad de memoria, binario nativo y buen control de recursos.

**Costo:** compilación más lenta y mayor curva de aprendizaje.

### D-02 — Bevy/ECS

**Yo elegí:** Bevy para escena, input y sistemas.

**Porque:** la metáfora neuronal/galaxia se beneficia de un motor de entidades y animación.

**Descarté:** una UI de widgets pura, porque limitaría la libertad visual.

### D-03 — Un nivel visible por vez

**Yo elegí:** carpeta central + hijos alrededor.

**Porque:** mostrar todo un árbol grande simultáneamente produce ruido y carga.

**Costo:** la persona necesita navegar por niveles, compensado con breadcrumb, búsqueda y pins.

### D-04 — Filesystem como fuente de verdad

**Yo elegí:** derivar el mapa de carpetas reales.

**Porque:** Borjes no debe mantener una estructura paralela que se desincronice.

### D-05 — Configuración XDG

**Yo elegí:** archivos bajo `~/.config` y datos/caché en ubicaciones de usuario.

**Porque:** evita paths absolutos y permite backup/desinstalación.

**Deuda:** algunos archivos conservan `datatree-rs` por historia y deben migrarse con compatibilidad.

### D-06 — Búsqueda global separada de la vista

**Yo elegí:** no limitar búsqueda al nivel visible.

**Porque:** el objetivo principal es llegar rápido a carpetas profundas.

### D-07 — Pins y frecency locales

**Yo elegí:** personalización sin servidor.

**Porque:** son hábitos privados del usuario y no necesitan sincronización remota en el alcance actual.

### D-08 — Orden determinista antes del límite

**Yo elegí:** ordenar y luego truncar nodos visibles.

**Porque:** evita que el resultado cambie según el orden arbitrario del filesystem.

### D-09 — Overlay bajo demanda

**Yo elegí:** experiencia invocable por atajo.

**Porque:** Borjes debe estar disponible sin ocupar espacio como una ventana tradicional.

**Riesgo:** foco, composición y Wayland varían por escritorio.

### D-10 — Assets como parte del producto

**Yo elegí:** tratar fuentes/iconos como dependencias de runtime.

**Porque:** un binario sin assets no reproduce la experiencia.

**Deuda:** el estado documentado puede resolver assets desde el checkout; una instalación final debe usar paths estables.

### D-11 — Compatibilidad KDE/X11 primero

**Yo elegí:** validar primero el entorno real del owner.

**Porque:** foco y overlay dependen del compositor.

**No significa:** incompatibilidad definitiva con otros escritorios, sino falta de aceptación equivalente.

### D-12 — Cero modificaciones de archivos desde la navegación

**Yo elegí:** el mapa es principalmente de lectura/apertura.

**Porque:** reduce riesgo. Las operaciones destructivas no deben estar implícitas en gestos visuales.

## ECS frente a MVC/MVVM

| Enfoque | Fortalezas | Diferencia en Borjes |
|---|---|---|
| MVC/MVVM | formularios, widgets y estado de pantalla | más natural para apps de negocio |
| ECS | muchas entidades, animaciones y sistemas independientes | más natural para nodos visuales y escena |
| árbol de widgets | jerarquía de UI directa | puede acoplar visual y dominio |

Yo uso ECS porque el producto se comporta más como una escena interactiva que como un formulario.

## Alternativas descartadas

- Electron: desarrollo web rápido, pero mayor consumo base y peor integración nativa para el objetivo.
- Tauri: buena alternativa para UI tradicional; menos adecuada para la escena 3D/animada actual.
- Qt/GTK: integración madura, pero la metáfora visual requeriría más render custom.
- WebGL local: portable, pero agrega navegador/servidor o empaquetado web.
- escanear todo en cada frame: simple, pero inaceptable para rendimiento.

## Atributos priorizados

1. respuesta visual;
2. consumo bajo cuando está oculto;
3. navegación rápida;
4. estabilidad;
5. portabilidad de configuración;
6. mantenibilidad;
7. compatibilidad multi-escritorio.

## Deuda reconocida

- raíz fija en el flujo principal;
- paths heredados;
- assets ligados al checkout según la versión;
- aceptación Wayland pendiente;
- CI/distribución depende de qué PR se fusione;
- módulos heredados mezclados con activos;
- benchmarks de consumo no automatizados.

## Regla de evolución

Yo no agrego efectos visuales si empeoran legibilidad, tiempo de apertura o consumo oculto. Cada cambio visual debe tener una razón de navegación, no solo decoración.