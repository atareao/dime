# Proposal

## Why

Para probar otro modelo, otra temperatura u otro proveedor hay que editar `dime.yml` y luego revertirlo. No existe forma de ajustar la configuración para una sola invocación.

## What Changes

- Nuevos flags que sobreescriben la configuración **sólo para la invocación en curso**, sin modificar `dime.yml`: `--model`, `--temperature` y `--provider`.
- Alias `--system` para `-i/--instructions`.
- Validación: `--temperature` debe ser numérica (la valida `clap`); `--provider` debe ser `openai` u `ollama` (case-insensitive), con error legible en caso contrario.
- Sin cambios incompatibles en `dime.yml`.

## Capabilities

### New Capabilities
<!-- Ninguna capacidad nueva. -->

### Modified Capabilities
- `config`: los overrides por CLI no se persisten ni modifican el fichero.
- `chat`: la petición usa los valores efectivos (override si se pasa, si no el de la configuración).

## Impact
- `src/main.rs` (flags de `clap`), `src/ia.rs` (aplicación de overrides a los valores efectivos), `README.md`.
- Sin dependencias nuevas.
