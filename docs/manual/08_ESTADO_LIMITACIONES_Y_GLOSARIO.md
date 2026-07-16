# 08 — Estado, limitaciones, roadmap y glosario

## Estado actual

| Capacidad | Estado |
|---|---|
| overlay visual | activo |
| navegación por niveles | activa |
| breadcrumb | activo |
| búsqueda global | activa |
| constelaciones | activas |
| lentes | activas con configuración heredada parcial |
| pins | activos |
| iconos | activos |
| frecency | activa con path heredado parcial |
| abrir en explorador | activo según desktop adapter |
| cambio de raíz desde UI | no conectado al flujo principal |
| instalación portable completa | depende de la rama/PR fusionado |
| KDE/X11 | entorno principal validado |
| Wayland | aceptación incompleta |
| CI/release | depende del estado de ramas de productización |
| consumo zero-idle | debe medirse en la versión realmente fusionada |

## Limitaciones

- raíz principal fija en `~/Data` en el flujo documentado;
- paths `datatree-rs` heredados;
- assets posiblemente dependientes del checkout;
- foco/overlay específico del compositor;
- compatibilidad Wayland incompleta;
- sin paquetes mantenidos en la rama base documentada;
- código heredado coexistente;
- pruebas gráficas no ejecutables en CI común;
- frecency/pins no sincronizados entre equipos;
- sin operaciones avanzadas de archivos por diseño;
- rendimiento en árboles enormes depende de límites y hardware.

## Roadmap recomendado

### 1. Portabilidad

- raíz configurable conectada y probada;
- unificar XDG bajo `borjes`;
- migración no destructiva;
- assets instalados;
- AppImage/DEB y checksums;
- licencia y release notes coherentes.

### 2. Rendimiento

- loop dirigido por eventos;
- cero polling oculto;
- benchmarks reproducibles;
- scan incremental;
- límites por árbol;
- profiling CPU/GPU/memoria.

### 3. Compatibilidad

- aceptación Wayland;
- GNOME y otros escritorios;
- múltiples monitores;
- HiDPI;
- drivers gráficos;
- fallback de overlay/foco.

### 4. Calidad

- CI completa;
- RustSec;
- tests de migración;
- smoke automatizado donde sea posible;
- matriz de compatibilidad versionada;
- separación/eliminación del código heredado.

### 5. Producto

- editor de raíz/lentes realmente conectado;
- UX de pins inválidos;
- import/export de configuración;
- accesibilidad;
- onboarding visual;
- búsqueda y constelaciones con métricas de utilidad.

## Glosario

**Overlay:** interfaz que aparece sobre el escritorio y se oculta bajo demanda.

**Nodo central:** carpeta que representa el contexto actual.

**Nodo hijo:** subcarpeta o elemento mostrado alrededor del centro.

**Breadcrumb:** ruta navegable que muestra cómo se llegó al lugar actual.

**Constelación:** agrupación visual temática distinta de la jerarquía pura.

**Lente:** filtro o regla que modifica qué se muestra o destaca.

**Pin:** acceso persistente a una carpeta.

**Frecency:** combinación de frecuencia y recencia de uso.

**ECS:** Entity Component System; arquitectura que separa datos en componentes y lógica en sistemas.

**Entity:** identificador de un objeto dentro de la escena.

**Component:** dato asociado a una entidad.

**System:** función que procesa entidades/componentes.

**Bevy:** motor Rust usado para app, ECS, input y render.

**winit:** capa de ventanas/eventos usada por el ecosistema.

**wgpu:** abstracción gráfica multiplataforma.

**X11:** sistema de ventanas clásico de Linux.

**Wayland:** protocolo moderno de composición con reglas distintas de foco/overlay.

**XDG:** estándar de ubicaciones para configuración, datos y caché.

**Desktop entry:** archivo `.desktop` que registra una aplicación.

**systemd user:** servicios ejecutados bajo la sesión del usuario.

**Asset:** recurso de runtime como icono, textura o fuente.

**CARGO_MANIFEST_DIR:** path de compilación que no debería ser la única forma de encontrar assets instalados.

**Smoke gráfico:** prueba manual o automatizada en una sesión gráfica real.

**Zero-idle:** objetivo medible de actividad casi nula cuando la app está oculta.

**Reactive render:** render que despierta por eventos en vez de dibujar continuamente.

**Symlink:** enlace a otro path; requiere política para evitar escapar de la raíz.

**Fallback:** comportamiento alternativo seguro cuando un recurso falta.

## Cierre

Yo considero Borjes una interfaz visual funcional y diferenciada, pero mantengo separados el producto actual, la productización propuesta en otros PR y la aceptación de cada escritorio. La documentación no debe convertir una intención de portabilidad o zero-idle en un hecho no fusionado.