# Proposal

## Why

La ruta real de red de `dime` no está cubierta por tests. Los tests existentes validan `build_body` y `parse_response` de forma pura, y `tests/scripting_output.rs` cubre el proceso completo, pero **nadie afirma** qué petición se envía realmente (método, ruta, cabeceras, cuerpo) ni cómo se comporta el cliente ante `4xx`/`5xx` y respuestas con forma inesperada. Una regresión en esas rutas pasaría desapercibida.

## What Changes

- Añadir `wiremock` como **dependencia de desarrollo** (no de runtime).
- Nuevo test de integración `tests/http_mock.rs` que ejecuta el binario `dime` contra un servidor mock local y verifica:
  - la petición de chat: método, ruta, cabecera `Authorization` y cuerpo;
  - la petición de listado de modelos;
  - las respuestas `4xx`/`5xx` y los cuerpos con forma inesperada (error legible, sin *panic*).
- Sin cambios de comportamiento en el producto.

## Capabilities

### New Capabilities
- `http-testing`: verificación de la ruta real de red contra un servidor HTTP local.

### Modified Capabilities
<!-- Ninguna. -->

## Impact
- `Cargo.toml` (`wiremock` en `[dev-dependencies]`), `tests/http_mock.rs` (nuevo), `README.md` (nota de desarrollo).
