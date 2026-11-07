# Proposal

## Why

Follow-ups de `PLAN-001`: `serde_yaml` está sin mantenimiento (versión `+deprecated`); `temperature` es `String` y se parsea tarde, de modo que una configuración inválida sólo falla al enviar la petición; y no existe forma de descubrir los modelos disponibles del proveedor.

## What Changes

- Migrar de `serde_yaml` a `serde_norway` (fork mantenido, API compatible).
- `temperature` pasa a `f64` con deserialización tolerante (acepta número YAML o cadena numérica); un valor no numérico produce un error de configuración legible en la carga.
- **BREAKING** (comportamiento): una `temperature` no numérica falla ahora al cargar la configuración, no al enviar.
- Nuevo comando `--list-models` que consulta el endpoint de modelos y lista los identificadores; se reintroduce el campo `models_endpoint` (por defecto `v1/models`).
- **BREAKING** (config): vuelve `models_endpoint` al modelo; con valor por defecto, los ficheros existentes siguen siendo válidos.

## Capabilities

### New Capabilities
- `models`: listado de modelos disponibles del proveedor.

### Modified Capabilities
- `config`: tipo de `temperature` y reintroducción de `models_endpoint`.
- `chat`: la temperatura se envía ya como número.

## Impact
- `Cargo.toml` (`serde_yaml` → `serde_norway`), `src/ia.rs`, `src/main.rs`, `config.*.example.yml`, `README.md`.