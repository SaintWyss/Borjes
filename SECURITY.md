# Seguridad

## Alcance

Borjes es una aplicación local sin servidor, puertos de red ni contenido remoto automático. Lee nombres de carpetas y carga assets que el propio usuario instala o selecciona explícitamente.

## Advisories transitivos aceptados temporalmente

La versión 1.0 mantiene tres advisories ignorados de forma explícita en CI:

- `RUSTSEC-2026-0204`, `crossbeam-epoch 0.9.18`: afecta el formateo `Display` de punteros internos inválidos. Borjes no construye ni formatea esos punteros desde datos del usuario.
- `RUSTSEC-2026-0195`, `quick-xml 0.39.4`: agotamiento de memoria mediante declaraciones XML maliciosas.
- `RUSTSEC-2026-0194`, `quick-xml 0.39.4`: complejidad cuadrática mediante atributos XML maliciosos.

`quick-xml` llega transitivamente mediante el soporte GLTF de Bevy. Borjes no descarga ni analiza XML remoto; solamente carga modelos locales configurados por el usuario. El riesgo residual requiere que el usuario seleccione deliberadamente un asset malicioso.

Estas excepciones deben eliminarse cuando Bevy permita `quick-xml >= 0.41.0` y `crossbeam-epoch >= 0.9.20`. Cualquier advisory adicional continúa bloqueando CI.

## Reporte

Los problemas de seguridad deben reportarse de forma privada al propietario del repositorio, evitando publicar pruebas explotables en un issue público.
