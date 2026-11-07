# Design

## Context

Ver `proposal.md` y `plans/PLAN-002.md`. Parte del código de `PLAN-001` (`build_body`, `auth_header`, `parse_response`, `IABotError`) se reutiliza.

## Goals / Non-Goals

- Goal: eliminar la dependencia sin mantenimiento y endurecer el tipado.
- Non-Goal: cambiar el formato de salida de las consultas.
- Non-Goal: soportar nuevos proveedores.

## Decisions

### D1. `serde_norway` como sustituto

Se elige `serde_norway 0.9.x` por ser un fork mantenido y API-compatible con `serde_yaml` (mismos `to_string`/`from_str`).

- Alternativa: `serde_yml`. Descartada: está marcada como deprecada/sin mantenimiento.
- Alternativa: `yaml_serde`. Descartada por menor familiaridad y posible divergencia de API.

### D2. Deserializador tolerante para `temperature`

Un `deserialize_with` que acepte número o cadena y produzca `f64`; `default = 1.0`. Errores de parseo se convierten en `IABotError::Config` con mensaje legible.

- Alternativa: mantener `String`. Descartada: traslada el fallo al momento del envío.

### D3. `--list-models` reutiliza la infraestructura

Se reintroduce `models_endpoint` (default `v1/models`) y se añade un flag booleano en `clap`. En modo listado, `question` deja de ser obligatoria (`required_unless_present = "list_models"`). La petición reutiliza la construcción de URL y `auth_header`; el parseo usa un DTO `ModelsResponse { data: Vec<ModelEntry> }` con `ModelEntry { id: String }".

- Alternativa: parsear como `Value`. Descartada por coherencia con `parse_response`.

### D4. Errores

Se añaden variantes al enum existente (`Config`, `Api`, `UnexpectedResponse`) sin romper las llamadas actuales.

## Risks / Trade-offs

- [Tolerancia de `temperature` como cadena perpetúa un formato laxo] → Documentar que el número es el formato preferido.
- [Reintroducir `models_endpoint` reabre un campo antes retirado] → Ahora tiene un uso real (`--list-models`) y valor por defecto.
- [Cambio de crate YAML puede alterar el formato serializado] → Verificar con tests de round-trip.

## Migration Plan

1. Cambiar dependencia y usos.
2. Tipar `temperature`.
3. Añadir `--list-models` + `models_endpoint`.
4. Tests, `fmt`/`clippy`, PR y release `0.3.0`.

Rollback: revert del PR; `dime.yml` sigue siendo compatible.

## Open Questions

- Ninguna.