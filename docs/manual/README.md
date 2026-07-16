# Manual normalizado de Borjes

Yo diseñé este manual con la misma receta de mis demás repositorios. El lector puede comprender el producto, instalarlo, navegar, revisar su arquitectura y operar su configuración sin conocer Bevy de antemano.

## Capítulos

| Capítulo | Resultado esperado |
|---|---|
| 01 | entender Borjes y abrirlo de forma segura |
| 02 | navegar, buscar, usar pins, lentes y constelaciones |
| 03 | comprender el motor y mis decisiones |
| 04 | recorrer vistas UML |
| 05 | conocer configuración, archivos de estado y contratos del escritorio |
| 06 | verificar seguridad, rendimiento y calidad |
| 07 | instalar, actualizar, respaldar y recuperar |
| 08 | conocer estado, límites, roadmap y vocabulario |

## Estados

- **Activo:** registrado en el loop principal y accesible al usuario.
- **Mixto:** parte activa con piezas heredadas.
- **Heredado:** permanece en el código, pero no forma el flujo vigente.
- **Planificado:** no está conectado todavía.
- **Pendiente de aceptación:** requiere escritorio/gráficos reales.

## Documentación avanzada

Yo conservo documentos de arquitectura, instalación, configuración, testing, compatibilidad y release status existentes. Este manual actúa como puerta de entrada y no reemplaza el detalle especializado.

## Regla de mantenimiento

Cuando cambio input, render, navegación, paths XDG, assets, atajos, compatibilidad o módulos activos, actualizo este manual y la evidencia de escritorio en el mismo pull request.