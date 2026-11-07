# Proposal

## Why

`IABot::ask` y `IABot::list_models` ignoran el código de estado HTTP: sólo interpretan el cuerpo. Por eso un `500 Internal Server Error` que devuelva un cuerpo de chat válido se trata como éxito, y un error del proveedor sin el campo `error` se degrada al genérico «respuesta inesperada», perdiendo el código. Es un fallo silencioso: el usuario recibe contenido incorrecto con código de salida cero.

## What Changes

- Ambas funciones DEBEN mirar `response.status()` y tratar cualquier código no exitoso (`>= 400`) como error.
- Si el cuerpo trae el campo `error` del proveedor, el error sigue siendo `IABotError::Api` con el mensaje del proveedor (no se pierde información).
- Si no lo trae, se usa una nueva variante `IABotError::Status(u16)` con el código HTTP en el mensaje.
- Sin cambios en las peticiones ni en el formato de las respuestas correctas.

## Capabilities

### New Capabilities
<!-- Ninguna. -->

### Modified Capabilities
- `chat`: la petición de chat distingue éxito de error por código de estado HTTP, no sólo por la forma del cuerpo.
- `models`: el listado de modelos aplica el mismo criterio.

## Impact
- `src/ia.rs` (`ask`, `list_models`, nueva variante de `IABotError`, helper de comprobación de estado), tests unitarios e `tests/http_mock.rs`.
