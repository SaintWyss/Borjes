# Rendimiento y presupuesto energético

| Estado | CPU objetivo | GPU objetivo |
|---|---:|---:|
| Oculto | < 0,1% promedio | sin frames |
| Visible estable | event-driven | sin frames periódicos |
| Interacción | limitado por VSync | activo temporalmente |

Los valores finales deben medirse en el equipo objetivo; CI valida lógica y compilación, no consumo real del escritorio.

## Zero-idle

1. `SIGUSR1` se bloquea antes de crear los threads de Bevy.
2. Un thread duerme dentro de `sigwait`.
3. La señal marca el toggle y envía `WinitUserEvent::WakeUp`.
4. El loop procesa el cambio y vuelve a dormir al estabilizarse.

El gobernador mantiene render continuo durante input, hover, búsqueda o transición. Tras 48 frames sin actividad cambia a `UpdateMode::reactive(Duration::MAX)`.

## Medición

```bash
pidstat -p "$(pgrep -x borjes)" 1
systemd-cgtop --user
nvidia-smi dmon
```

Medir al menos 60 segundos en cada estado.
