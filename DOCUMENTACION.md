# Documentación de Borjes

Yo organicé la documentación de Borjes para que una persona sin conocimientos de Rust, Bevy, gráficos 3D o KDE pueda entender el producto y usarlo sin depender de rutas personales de mi computadora.

## Recorrido recomendado

1. [Producto y primeros pasos](docs/manual/01_PRODUCTO_Y_PRIMEROS_PASOS.md)
2. [Uso funcional](docs/manual/02_USO_FUNCIONAL.md)
3. [Arquitectura y decisiones](docs/manual/03_ARQUITECTURA_Y_DECISIONES.md)
4. [Diagramas UML](docs/manual/04_UML.md)
5. [Configuración, datos y APIs](docs/manual/05_CONFIGURACION_DATOS_Y_API.md)
6. [Seguridad, pruebas y calidad](docs/manual/06_SEGURIDAD_PRUEBAS_Y_CALIDAD.md)
7. [Operación, despliegue y recuperación](docs/manual/07_OPERACION_DESPLIEGUE_Y_RECUPERACION.md)
8. [Estado, limitaciones, roadmap y glosario](docs/manual/08_ESTADO_LIMITACIONES_Y_GLOSARIO.md)

## Criterio editorial

- Escribí todo en español y en primera persona.
- Explico primero la experiencia visual y después el motor Rust/Bevy.
- Diferencio funciones activas, código heredado y propuestas de otros PR abiertos.
- No uso rutas absolutas de una máquina concreta.
- Documento compatibilidad y consumo como propiedades que deben medirse.

## Fuente de verdad

1. código y pruebas de la rama base;
2. este manual normalizado;
3. documentación técnica vigente;
4. `.planning/` y notas históricas;
5. otros PR abiertos, que no se consideran parte de `main` hasta fusionarse.

Los archivos personales del usuario, pins y hábitos de navegación no deben incluirse en el repositorio.