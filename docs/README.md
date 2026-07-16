# Documentación de Borjes

Este directorio contiene la documentación canónica del producto y de su operación. Los documentos históricos de `.planning/` y `docs/superpowers/` explican decisiones y fases anteriores, pero no sustituyen esta fuente de verdad.

## Mapa documental

| Documento | Propósito | Audiencia |
|---|---|---|
| [README raíz](../README.md) | Entrada rápida, capacidades y comandos básicos | Usuarios y colaboradores |
| [Arquitectura](ARCHITECTURE.md) | Componentes, flujo de datos, módulos activos y heredados | Desarrollo y arquitectura |
| [Instalación](INSTALLATION.md) | Preparación del sistema, compilación y actualización | Usuarios y desarrollo |
| [Configuración](CONFIGURATION.md) | Raíz, persistencia XDG, pins, iconos, lentes y frecency | Usuarios avanzados y soporte |
| [Integración de escritorio](desktop-setup.md) | Autostart, atajo global, systemd user y troubleshooting | Usuarios Linux |
| [Testing](TESTING.md) | Validaciones automáticas y aceptación manual | Desarrollo y QA |
| [Estado de release](RELEASE_STATUS.md) | Estado implementado, límites y gates pendientes | Producto y release |

## Precedencia

Cuando exista una contradicción, se usa este orden:

1. Código y pruebas de la rama o commit inspeccionado.
2. README raíz y documentos canónicos de `docs/`.
3. Especificaciones históricas y planes de `.planning/` o `docs/superpowers/`.

## Convenciones de estado

- **Implementado**: conectado al loop principal y verificable en el código actual.
- **Parcial**: existe código real, pero falta integración, aceptación o distribución.
- **Heredado**: permanece en el repositorio, pero no forma parte del flujo principal.
- **Objetivo**: dirección futura sin implementación completa.

## Regla de actualización

Todo cambio que afecte controles, rutas de configuración, módulos activos, compatibilidad de escritorio o procedimiento de instalación debe actualizar la documentación canónica en el mismo pull request.