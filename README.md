# Borjes

Borjes es un navegador visual de carpetas nativo para Linux, escrito en Rust y Bevy. Funciona como un overlay invocable: permite buscar, recorrer y abrir bibliotecas profundas sin reemplazar al explorador de archivos.

## Funciones

- Búsqueda global en vivo, insensible a acentos.
- Constelación visual de resultados agrupados por rama.
- Navegación radial con breadcrumb, historial y previews.
- Lentes temáticas transversales, por ejemplo `@seguridad`.
- Pins rápidos mediante `Alt+1` a `Alt+9`.
- Ranking por frecencia: lo utilizado aparece primero y adquiere mayor peso visual.
- Iconos, PNG, emojis y modelos GLB personalizados.
- Configuración de raíz y lentes desde la propia aplicación con `F10`.
- Overlay fullscreen para KDE/Linux.
- Arquitectura zero-idle: oculto duerme hasta recibir el evento del atajo.

## Controles

| Acción | Control |
|---|---|
| Mostrar u ocultar | Atajo global asociado a `borjes-toggle` |
| Buscar | Escribir directamente |
| Seleccionar resultado | Flechas o `Tab` |
| Abrir | `Enter` |
| Entrar en carpeta | Click o `Tab` sobre el nodo |
| Volver | `Backspace` o nodo violeta |
| Pin rápido | `Alt+1..9` |
| Configuración | `F10` o botón ⚙ |
| Ocultar | `Esc` |

## Instalación

Requisitos: Linux, Rust estable, Cargo y las bibliotecas nativas requeridas por Bevy.

```bash
git clone https://github.com/SaintWyss/Borjes.git
cd Borjes
./scripts/install.sh
```

El instalador:

- compila el binario release;
- instala Borjes en `~/.local/bin`;
- instala sus assets en `~/.local/share/borjes`;
- registra `borjes.service` como servicio de usuario;
- instala el lanzador de escritorio.

Después se debe asociar el atajo global deseado, por ejemplo `Meta+1`, al comando:

```text
~/.local/bin/borjes-toggle
```

Para desinstalar:

```bash
./scripts/uninstall.sh
```

La desinstalación conserva la configuración personal.

## Configuración

Todos los datos viven en:

```text
$XDG_CONFIG_HOME/borjes/
```

o, si la variable no existe:

```text
~/.config/borjes/
```

Borjes migra automáticamente datos anteriores de `datatree-rs` sin sobrescribir configuración nueva.

## Rendimiento

Borjes utiliza render adaptativo:

- oculto: event loop dormido, sin polling periódico;
- visible e interactuando: frames continuos durante la transición;
- visible en reposo: vuelve automáticamente al loop reactivo;
- escaneo y render tienen límites defensivos para carpetas grandes.

Los presupuestos y el procedimiento de medición están en [docs/PERFORMANCE.md](docs/PERFORMANCE.md).

## Desarrollo

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo build --locked --release
```

GitHub Actions ejecuta formato, Clippy, tests, build release y auditoría RustSec en cada PR.

## Arquitectura

La arquitectura activa es drill-down por nivel. El árbol completo permanece indexado para búsqueda, constelaciones y lentes, mientras el render solamente monta el contexto visible.

Más detalles en [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Licencia

MIT.

