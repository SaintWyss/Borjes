# Presupuesto de rendimiento

Borjes es una aplicación 3D y no pretende igualar la memoria de un explorador 2D. Sí debe comportarse como una aplicación de escritorio cuando no existe interacción.

## Objetivos

| Estado | CPU | GPU | Latencia |
|---|---:|---:|---:|
| Oculto durante 60 s | < 0,1 % promedio | 0 % de render | toggle < 150 ms |
| Visible en reposo | < 1 % promedio | mínima | entrada < 50 ms |
| Búsqueda | — | — | < 2 ms por tecla con 5000 carpetas |
| Navegación | — | activa temporalmente | sin frame-drop perceptible |

## Diseño

El handler de `SIGUSR1` escribe un byte en un self-pipe no bloqueante. Un thread dormido en el extremo de lectura transforma ese byte en un `WinitUserEvent::WakeUp`. Esto permite usar `WinitSettings::desktop_app()` sin polling.

El gobernador de render solicita redraws durante 900 ms después de:

- movimiento o click del mouse;
- entrada de teclado;
- navegación;
- hover;
- búsqueda;
- cambio de visibilidad.

Cuando termina esa ventana, winit vuelve a esperar eventos.

## Medición manual

```bash
pidstat -p "$(pgrep -x borjes)" 1 60
ps -o pid,rss,%cpu,cmd -p "$(pgrep -x borjes)"
nvidia-smi dmon -s u
```

La medición debe hacerse después de 10 segundos de estabilización, primero oculto y luego visible sin mover el mouse.

## Regla de aceptación

Una función visual nueva no se acepta si mantiene redraws permanentes en reposo. Toda animación debe ser acotada o depender del gobernador.

