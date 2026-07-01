# Borjes

`Borjes` es una aplicacion de escritorio nativa escrita en Rust que visualiza un arbol de carpetas como una interfaz 3D navegable. Su objetivo principal es permitir encontrar y abrir carpetas profundas de `~/Data` rapidamente, sin tener que recorrer manualmente niveles de directorios en el explorador de archivos.

El concepto de producto es **Borjes**: un overlay visual tipo mapa neuronal/galaxia para navegar informacion local. La version actual ya funciona como overlay fullscreen on-demand y muestra un nivel del arbol por vez: la carpeta actual aparece como nodo central y sus subcarpetas alrededor.

## Para que sirve

- Navegar visualmente una estructura grande de carpetas.
- Reducir el tiempo para llegar a carpetas profundas y repetitivas.
- Usar un overlay invocable por atajo global, sin mantener una ventana normal abierta.
- Explorar `~/Data` con una interfaz radial: hub central, hijos alrededor y breadcrumb superior.

El caso de uso esperado es una carpeta personal de conocimiento/trabajo, por ejemplo:

```text
~/Data/
  00_Inbox/
  10_Workspace/
  20_Knowledge/
  30_Cybersecurity/
```

## Estado actual

La aplicacion arranca oculta y queda esperando una senal `SIGUSR1` para mostrarse u ocultarse. Cuando esta visible usa una ventana borderless fullscreen, siempre arriba, pensada para integrarse con un atajo global del entorno de escritorio.

Funcionalidades activas:

- Overlay fullscreen sin decoraciones, siempre arriba.
- Raiz por defecto: `~/Data`.
- Escaneo recursivo de carpetas con exclusion de ruido como `node_modules`, `.git`, `.cache`, `target`, `__pycache__`, `Steam` y `.Trash`.
- Navegacion por nivel:
  - nodo central: carpeta actual;
  - nodos alrededor: subcarpetas directas;
  - nodo fantasma superior: carpeta padre.
- Hover sostenido durante `0.5s` para entrar a una subcarpeta o subir al padre.
- `Backspace` para volver a la carpeta padre.
- Breadcrumb superior con el path actual.
- Labels renderizados con `egui` sobre los nodos.
- Representaciones visuales por carpeta: esfera default, PNG, emoji rasterizado o modelo GLB/GLTF.
- Animaciones de entrada, glow/emissive materials y camara que se ajusta al tamano del nivel.

Hay modulos heredados o parcialmente preparados para etapas anteriores/siguientes:

- `layout.rs`: layout global force-directed para renderizar todo el grafo a la vez.
- `render.rs`: infraestructura para nodos globales, colores por cluster y file bursts.
- `ui.rs`: buscador, resaltado y panel de reescaneo, actualmente no conectado al loop principal de Borjes.
- `pins.rs`: persistencia de carpetas pineadas en JSON, tambien preparada para integracion futura.

## Tecnologias

- **Rust 2021**: lenguaje principal.
- **Bevy 0.19**: motor de app, ECS, ventana, render 3D, input y camara.
- **bevy_egui 0.40.1**: UI overlay para breadcrumb, labels y componentes de interfaz.
- **bevy_panorbit_camera 0.35**: dependencia de camara usada por la arquitectura anterior y utilidades existentes.
- **ignore 0.4**: recorrido de directorios respetando reglas de ignorado y filtrado.
- **libc 0.2**: manejo de `SIGUSR1` para mostrar/ocultar el overlay.
- **open 5.3.5**: apertura de rutas con la aplicacion del sistema.
- **rand 0.8**: posiciones aleatorias en codigo de layout heredado.
- **unicode-normalization 0.1**: normalizacion de texto para busqueda sin acentos.

## Estructura del repo

```text
src/
  main.rs          # bootstrap Bevy, ventana, recursos y sistemas activos
  desktop.rs      # overlay on-demand: SIGUSR1, visibilidad y Escape
  scan.rs         # escaneo de carpetas y archivos
  nav.rs          # arbol en memoria y layout radial por nivel
  drill.rs        # experiencia Borjes actual: hub, hijos, hover-dwell, labels
  render.rs       # render de grafo global y assets reutilizables
  interaction.rs  # clicks, hover, apertura de archivos/carpetas y pin visual
  ui.rs           # buscador/panel/pin menu heredados
  pins.rs         # persistencia de pins en config local
  layout.rs       # layout force-directed global
  root.rs         # configuracion de raiz y expansion de ~

docs/
  desktop-setup.md                         # setup de overlay y atajo KDE
  superpowers/specs/*                      # decisiones de producto/diseno

assets/icons/
  *.png                                    # iconos disponibles para UI futura
```

## Requisitos

- Rust estable con Cargo.
- Linux desktop. El setup documentado esta orientado a KDE Plasma.
- Dependencias del sistema necesarias para compilar Bevy en Linux, por ejemplo toolchain C/C++, `pkg-config`, bibliotecas X11/Wayland, ALSA y udev segun la distribucion.
- Una carpeta `~/Data` si se quiere usar la raiz por defecto.

## Ejecutar

Compilar y ejecutar en modo release:

```bash
cargo run --release
```

La ventana inicia oculta. Para alternar visibilidad:

```bash
pkill -USR1 -x borjes
```

Para ocultar cuando esta visible:

```text
Esc
```

Para cerrar el proceso:

```bash
pkill -x borjes
```

En KDE se puede bindear `pkill -USR1 -x borjes` a un atajo global, por ejemplo `Meta+1`. Ver [docs/desktop-setup.md](docs/desktop-setup.md) para el setup de autostart y atajo.

## Controles

- **Mostrar/ocultar overlay**: enviar `SIGUSR1` al proceso.
- **Entrar a una subcarpeta**: mantener el cursor sobre un nodo hijo hasta completar el anillo de progreso.
- **Subir al padre**: mantener el cursor sobre el nodo fantasma superior.
- **Subir al padre con teclado**: `Backspace`.
- **Personalizar una carpeta**: click derecho sobre un nodo y elegir icono, emoji, PNG, GLB o reset.
- **Ocultar overlay**: `Esc`.

## Tests

Ejecutar la suite:

```bash
cargo test
```

Los tests cubren, entre otras cosas:

- expansion de `~`;
- escaneo de carpetas y archivos;
- exclusion de archivos ocultos o directorios de ruido;
- limite de 200 archivos por burst;
- layout radial determinista;
- layout force-directed sin NaN/infinitos;
- normalizacion de busqueda sin acentos;
- serializacion de pins.

## Persistencia local

Los pins se guardan en:

```text
$XDG_CONFIG_HOME/borjes/pins.json
```

Si `XDG_CONFIG_HOME` no esta definido:

```text
~/.config/borjes/pins.json
```

En el estado actual, la infraestructura de pins existe en el codigo pero no forma parte del flujo principal de NEURA.

## Iconos y modelos personalizados

Las representaciones por carpeta se guardan en:

```text
$XDG_CONFIG_HOME/borjes/icons.json
```

Si `XDG_CONFIG_HOME` no esta definido:

```text
~/.config/borjes/icons.json
```

Formato:

```json
{
  "folders": {
    "/home/user/Data/30_Cybersecurity": {
      "kind": "png",
      "asset": "icons/shield.png",
      "scale": 1.0
    },
    "20_Knowledge": {
      "kind": "emoji",
      "emoji": "📚",
      "scale": 1.1
    },
    "Robotics": {
      "kind": "model",
      "asset": "custom/robot.glb",
      "scale": 0.8
    }
  }
}
```

`asset` puede apuntar a `assets/` del repo, por ejemplo `icons/folder.png` o `custom/robot.glb`, o a un archivo absoluto bajo `~/.config/borjes/assets/`. El formato viejo `{ "FolderName": "shield" }` sigue funcionando como alias a `assets/icons/shield.png`.

## Notas de arquitectura

El repo conserva dos enfoques:

1. **Grafo global tipo galaxia**: escanea todas las carpetas, calcula posiciones 3D con un layout force-directed y renderiza clusters por raiz. Esa ruta existe en `layout.rs`/`render.rs`, pero no es el flujo principal actual.
2. **Drill-down por nivel**: carga el arbol completo en memoria y renderiza solo la carpeta actual con sus hijos. Esta es la experiencia activa, implementada principalmente en `nav.rs` y `drill.rs`.

El segundo enfoque reduce ruido visual y evita mostrar miles de nodos a la vez. La aplicacion mantiene el arbol completo indexado para navegar sin reescanear en cada nivel.

## Roadmap sugerido

- Reconectar buscador como teletransporte a carpetas profundas.
- Integrar pins como accesos rapidos navegables.
- Permitir configurar la raiz desde UI activa o argumento CLI.
- Unificar o retirar codigo heredado del grafo global cuando Borjes este completo.
- Agregar instrucciones por distribucion para dependencias nativas de Bevy.
