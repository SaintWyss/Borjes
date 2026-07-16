# 01 — Producto y primeros pasos

## Explicación simple

Yo construí Borjes como una aplicación de escritorio que convierte una estructura de carpetas en un mapa visual navegable. La carpeta actual aparece como centro y sus subcarpetas o elementos se distribuyen alrededor.

El objetivo no es reemplazar todas las funciones del explorador de archivos. Yo lo uso para llegar rápido a carpetas profundas, reconocer áreas por posición y explorar información como si fuera un mapa.

## Ejemplo cotidiano

Un explorador clásico se parece a una lista de calles. Borjes se parece a un mapa de subte: la ubicación, tamaño y relación de cada nodo ayudan a recordar el camino.

## Cómo lo explicaría yo

> Yo uso Borjes como un overlay visual para navegar mis carpetas. Abro el mapa con un atajo, elijo una rama, busco o fijo accesos y vuelvo al escritorio sin mantener una ventana tradicional abierta.

## Capacidades activas

- overlay visual;
- navegación por niveles;
- breadcrumb;
- búsqueda global;
- constelaciones temáticas;
- lentes/filtros;
- pins;
- iconos;
- ponderación por frecuencia/recencia;
- apertura de carpetas;
- integración de escritorio según el entorno.

## Lo que no debo asumir

- El cambio de raíz desde UI no está conectado en el flujo principal actual.
- Wayland no tiene la misma aceptación que KDE/X11.
- Copiar solo el binario puede dejar assets sin resolver en el estado documentado.
- Configuraciones de lentes y frecency conservan nombres heredados en algunas rutas.
- Un PR de productización abierto no forma parte de `main` hasta fusionarse.

## Requisitos

- Linux;
- entorno gráfico;
- Rust estable y Cargo para compilar;
- dependencias del sistema requeridas por Bevy/winit;
- KDE/X11 como entorno mejor validado;
- una raíz de datos, actualmente `~/Data` en el flujo principal documentado.

## Compilación

```bash
cargo build --release
```

Antes:

```bash
cargo fmt --all -- --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## Primera ejecución

```bash
cargo run --release
```

Yo comienzo con una estructura de prueba:

```text
~/Data/
  Estudios/
  Proyectos/
  Personal/
```

No pruebo inicialmente sobre una raíz enorme o un montaje lento.

## Primer recorrido

1. abrir el overlay;
2. identificar nodo central;
3. navegar a una subcarpeta;
4. volver con breadcrumb o acción de retroceso;
5. buscar un nombre;
6. fijar un pin;
7. activar una lente;
8. abrir una carpeta en el explorador;
9. cerrar/ocultar el overlay;
10. revisar consumo en reposo.

## Qué debería observar

- inicio sin panic;
- nodos legibles;
- navegación determinista;
- búsqueda con resultados correctos;
- pins persistentes;
- ausencia de escrituras fuera de configuración;
- retorno rápido al escritorio;
- consumo reducido cuando está oculto, según la rama/runtime probado.

## Diferencia entre uso y desarrollo

En desarrollo, los assets pueden resolverse desde el checkout. Una instalación distribuible debe copiar binario y assets a rutas estables, registrar desktop entry y evitar depender de `CARGO_MANIFEST_DIR`.

## Criterio para avanzar

Yo considero la aplicación utilizable cuando:

- la raíz correcta se escanea;
- el atajo no colisiona;
- el overlay abre/cierra de forma estable;
- los paths XDG son escribibles;
- el consumo oculto fue medido;
- existe rollback de configuración;
- la sesión gráfica objetivo fue probada.