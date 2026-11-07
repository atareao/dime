# Design

## Context

Ver `proposal.md` y `plans/PLAN-003.md` (Fase 1). `IABot` guarda la configuración (`model`, `temperature`, `provider`, `system_role`, ...) y `main.rs` la carga con `read_content`. `is_ollama()` y `build_body` deciden en función de `provider`, y `validate()` exige token cuando el proveedor es `openai`.

## Goals / Non-Goals

- Goal: overrides de una sola invocación sin tocar el fichero.
- Non-Goal: persistir overrides o generar un fichero nuevo.
- Non-Goal: añadir flags para el resto de campos de `dime.yml`.

## Decisions

### D1. Flags en `clap`

`--model <String>`, `--temperature <f64>` y `--provider <String>` como `Option<...>`. `--system` se añade como `alias` de `-i/--instructions`.

### D2. Aplicación de overrides sobre `IABot`

Se aplican tras `read_content` y ANTES de `validate()` y de la consulta, mediante un método p. ej. `apply_overrides(&mut self, model, temperature, provider)` que muta los campos del `IABot`. El fichero nunca se reescribe.

- Alternativa: un struct de overrides separado. Descartada: `IABot` ya es el portador de la configuración efectiva.

### D3. Validación

`--temperature` no numérica la rechaza `clap` (código ≠ 0, error de uso). `--provider` se valida contra `{openai, ollama}` (case-insensitive); un valor inválido produce un error de configuración legible (`IABotError`, sin *panic*).

- Nota: el override de proveedor afecta a `validate()`; con `ollama` no se exige token.

### D4. `--system`

Alias de `-i/--instructions`. Cuando no se pasa ninguno, se sigue usando `system_role` del fichero.

## Risks / Trade-offs

- [El orden importa: overrides antes de `validate()`] → Aplicar antes de validar; cubrir con tests (proveedor `ollama` sin token).
- [Mutabilidad de `IABot`] → Método `apply_overrides` documentado y testeado.

## Migration Plan

1. Flags en `clap`.
2. `apply_overrides` y validación.
3. Tests.
4. README, `fmt`/`clippy`, PR a `development`.

Rollback: revert del PR; `dime.yml` sigue siendo compatible.

## Open Questions

- ¿`--provider` debería inferir también `use_https`/endpoint? Decisión: no en esta fase; sólo cambia el proveedor.
