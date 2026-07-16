# 06 — Seguridad, pruebas y calidad

## Activos que protejo

- privacidad de nombres y paths;
- integridad del filesystem;
- configuración personal;
- pins y hábitos de navegación;
- estabilidad de la sesión gráfica;
- consumo de CPU/GPU/memoria;
- cadena de suministro del binario y assets.

## Amenazas principales

| Amenaza | Ejemplo | Control |
|---|---|---|
| ejecución de path malicioso | nombre con caracteres de shell | argumentos separados, sin concatenación |
| traversal/symlink | salir de raíz | normalización y política explícita |
| config corrupta | JSON inválido | validación, fallback y backup |
| exposición de paths | logs completos | logging mínimo y local |
| instancia duplicada | dos overlays/locks | launcher y control de proceso |
| consumo oculto | loop render continuo | modo reactivo y medición |
| asset reemplazado | icono/fuente no confiable | instalación controlada y paths fijos |
| dependencia vulnerable | crate comprometido | lockfile y auditoría RustSec |
| crash gráfico | driver/backend | logs, fallback y matriz de compatibilidad |
| borrado accidental | gesto ambiguo | producto de lectura/apertura, sin delete implícito |

## Principio de menor privilegio

Yo ejecuto Borjes como el usuario normal. No requiere root para escanear sus propias carpetas, abrir el explorador o registrar un desktop entry de usuario.

## Filesystem

- no sigo symlinks sin política explícita;
- no interpreto nombres como comandos;
- manejo permisos denegados sin abortar todo el árbol;
- limito profundidad/cantidad cuando corresponde;
- no escribo dentro de `~/Data` para mantener estado de UI;
- no registro contenidos sensibles.

## Pruebas Rust

```bash
cargo fmt --all -- --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
```

Las pruebas deben cubrir:

- scan y orden determinista;
- paths y nombres extraños;
- navegación y breadcrumb;
- búsqueda;
- pins;
- lentes;
- frecency;
- migración de config;
- resolución de assets;
- comandos de escritorio construidos de forma segura.

## Pruebas de arquitectura

Yo verifico que:

- módulos de dominio/scan no dependan del render innecesariamente;
- la integración de escritorio esté aislada;
- configuración no esté hardcodeada a un usuario;
- código heredado no se presente como activo;
- las operaciones de lectura no muten el filesystem.

## Smoke de escritorio

Un CI sin sesión gráfica no alcanza. Yo pruebo en el escritorio objetivo:

1. arranque;
2. abrir/cerrar por atajo;
3. foco correcto;
4. múltiples monitores;
5. HiDPI;
6. búsqueda;
7. navegación;
8. apertura externa;
9. persistencia;
10. oculto durante varios minutos.

## Rendimiento

Métricas mínimas:

- tiempo de arranque frío/caliente;
- tiempo de scan por cantidad de nodos;
- memoria después de abrir y ocultar;
- CPU visible e invisible;
- GPU visible e invisible;
- FPS durante animación;
- latencia input→frame;
- tiempo de búsqueda;
- número de entidades.

Yo comparo contra una baseline, no uso “se siente rápido” como evidencia.

## Cero consumo en reposo

“Zero-idle” debe significar un criterio medible:

- sin polling periódico innecesario;
- render bajo demanda cuando está oculto;
- CPU cercana al ruido del sistema;
- GPU sin actividad sostenida;
- ausencia de wakeups frecuentes.

Si una optimización existe solo en otro PR, no la atribuyo a `main`.

## CI

Un workflow ideal valida:

- fmt;
- clippy con warnings como errores;
- tests;
- build release;
- auditoría RustSec;
- packaging;
- enlaces y Mermaid;
- ausencia de paths personales.

## Supply chain

- `Cargo.lock` versionado para aplicación;
- dependencias revisadas;
- licencia definida;
- SBOM/checksums para releases;
- binarios desde CI controlado;
- assets incluidos y verificables;
- acciones de GitHub fijadas según política.

## Accesibilidad y UX

Yo evalúo:

- contraste;
- tamaño de texto;
- navegación por teclado;
- reduced motion;
- labels y contexto de resultados;
- foco visible;
- colores no usados como única señal;
- tolerancia a daltonismo;
- escalado HiDPI.

## Definición de terminado

Una feature visual requiere:

1. propósito de navegación;
2. estado conectado;
3. tests de lógica;
4. smoke gráfico;
5. medición de rendimiento;
6. fallback;
7. documentación;
8. compatibilidad y rollback.

## Regla de calidad

Yo no acepto una mejora visual que haga ilegible el mapa, incremente el consumo oculto sin justificación o dependa de la ruta del checkout.