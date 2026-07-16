# 02 — Uso funcional

## Modelo de navegación

Yo represento una carpeta actual como nodo central. Sus elementos cercanos se convierten en nodos alrededor. Al seleccionar una subcarpeta, esa carpeta pasa a ser el nuevo centro.

```text
raíz → carpeta → subcarpeta → destino
```

El breadcrumb conserva el contexto para volver.

## Abrir y ocultar

Borjes está pensado como overlay invocado bajo demanda. La integración exacta depende del instalador/atajo configurado.

En desarrollo:

```bash
cargo run --release
```

En una instalación de escritorio, yo uso un launcher o wrapper que:

- detecta si Borjes está ejecutándose;
- envía la señal/evento soportado para mostrarlo;
- evita crear instancias duplicadas;
- inicia la aplicación si no existe.

## Selección y navegación

Las acciones básicas son:

- mover el foco;
- seleccionar nodo;
- entrar en carpeta;
- volver;
- abrir en explorador;
- cerrar/ocultar.

Los bindings exactos se toman del código y la configuración vigente. Yo no documento una tecla como contrato estable si no está validada por tests o smoke.

## Búsqueda global

La búsqueda recorre el índice/árbol disponible y devuelve coincidencias por nombre o path según la implementación.

Flujo recomendado:

1. abrir búsqueda;
2. escribir una consulta específica;
3. revisar path completo;
4. seleccionar resultado;
5. navegar o abrir.

Yo muestro contexto suficiente para diferenciar carpetas con el mismo nombre.

## Pins

Los pins son accesos persistentes a ubicaciones frecuentes.

Uso:

- fijar una carpeta;
- listar o mostrar pins;
- abrir/navegar al pin;
- eliminarlo si ya no existe.

Un pin puede quedar huérfano si la carpeta se mueve fuera de Borjes. La aplicación debe mostrarlo como inválido o permitir limpieza, no resolverlo a otro path por coincidencia accidental.

## Lentes

Una lente modifica qué nodos se muestran o cómo se ponderan. Yo las uso para reducir ruido en árboles grandes.

Ejemplos conceptuales:

- mostrar solo proyectos;
- destacar documentos recientes;
- ocultar carpetas técnicas;
- agrupar por tema.

Las lentes actuales se cargan desde configuración. Parte del path conserva el nombre `datatree-rs` por compatibilidad heredada.

## Constelaciones

Las constelaciones organizan grupos temáticos para ofrecer una vista distinta del árbol puramente jerárquico.

Yo las uso como capa de navegación, no como una nueva fuente de verdad. El filesystem continúa siendo la autoridad de nombres y paths.

## Frecency

Frecency combina frecuencia y recencia. Una carpeta usada muchas veces o recientemente puede recibir mayor relevancia visual.

Yo no interpreto frecency como importancia absoluta. Es una señal personal y local que puede resetearse.

## Iconos

Borjes puede asociar iconos a carpetas o categorías. Si un asset falta, la aplicación debe usar un fallback y no bloquear la navegación.

## Abrir en explorador

La acción delega al sistema de escritorio la apertura de una carpeta. Yo valido el path y evito construir comandos mediante concatenación insegura.

## Cambio de raíz

La UI heredada puede contener controles relacionados con una raíz configurable, pero el flujo principal documentado sigue usando `~/Data`. Hasta conectar y probar esa función, yo no la presento como activa.

## Estructuras grandes

Para árboles grandes:

- evito escanear mounts innecesarios;
- uso límites visuales deterministas;
- ordeno antes de truncar;
- no sigo symlinks sin una política clara;
- mido tiempo de scan y memoria;
- separo indexación de render cuando sea necesario.

## Errores frecuentes

| Síntoma | Causa probable | Acción |
|---|---|---|
| ventana negra o crash | driver/dependencia gráfica | ejecutar con logs y validar Bevy/wgpu |
| no encuentra `~/Data` | raíz inexistente | crearla o ajustar el código/config soportado |
| faltan iconos/assets | ejecución fuera del checkout | usar instalación con assets completos |
| atajo no abre | launcher/service mal registrado | probar binario y wrapper por separado |
| funciona en X11 pero no Wayland | restricciones de foco/overlay | usar sesión soportada o realizar aceptación Wayland |
| pins desaparecen | path XDG o JSON inválido | revisar permisos y backup |
| alto consumo oculto | loop de render/polling | medir CPU/GPU y revisar rama/runtime |
| lente no carga | JSON/path heredado | validar archivo y ubicación |

## Reset selectivo

Yo respaldo antes de borrar configuración. Puedo resetear por separado:

- pins;
- iconos personalizados;
- lentes;
- frecency;
- cachés.

No elimino todo el directorio de configuración para resolver un único JSON corrupto sin conservar evidencia.

## Regla de uso

Yo trato Borjes como una vista sobre el filesystem. Una animación o nodo visible no debe modificar archivos por sí solo; abrir o navegar es distinto de renombrar, mover o borrar.