# Design

## Context

Ver `proposal.md` y `plans/PLAN-003.md` (Fase 2). El CLI usa `clap`; hoy la pregunta es obligatoria por `required_unless_present_any = ["question_flag", "list_models"]` en `src/main.rs`.

## Goals / Non-Goals

- Goal: permitir la entrada por tubería/redirección sin romper el uso interactivo.
- Non-Goal: modo conversación o historial.
- Non-Goal: heurísticas de TTY multiplataforma más allá de lo estándar.

## Decisions

### D1. Detección de TTY con `std::io::IsTerminal`

Se usa `std::io::stdin().is_terminal()` (Rust ≥ 1.70). Sin dependencias nuevas.

- Alternativa: crate `atty`. Descartada: deprecada.
- Alternativa: crate `is-terminal`. Descartada: innecesaria con `IsTerminal` en std.

### D2. Relajar la obligatoriedad en `clap` y validar en runtime

La pregunta deja de ser obligatoria en `clap`; se valida en runtime: si stdin es un TTY, no hay pregunta y no se usa `--list-models`, se emite el error de uso (código distinto de cero) sin leer de stdin.

### D3. Marcador `-`

Si la pregunta (por `-q` o posicional) es exactamente `-`, se fuerza la lectura de la pregunta desde stdin.

### D4. Contexto desde stdin

Si hay pregunta y stdin no es un TTY con datos, se lee stdin y se concatena al mensaje `user` como contexto (pregunta + separador + contexto). El mensaje `system` no cambia.

### D5. Errores

Se añade una variante tipada en `IABotError` (p. ej. `Input`) con mensaje legible; nunca se entra en *panic*.

## Risks / Trade-offs

- [Bloqueo si se intentara leer un stdin interactivo] → Sólo se lee stdin cuando NO es un TTY; nunca se lee en modo interactivo.
- [Cambiar la obligatoriedad de `cl,ap` altera la ayuda y los mensajes] → Mantener el error legible y cubrirlo con tests.
- [Scripts que pasan pregunta y además canalizan algo] → El contenido de stdin se usa como contexto; queda documentado en el README.

## Migration Plan

1. `is_terminal` y resolución de la pregunta.
2. Marcador `-".
3. Contexto desde stdin.
4. Tests, `fmt`/`clippy`, PR a `development`.

Rollback: revert del PR; el uso interactivo no cambia.

## Open Questions

- ¿`--list-models` debe ignorar stdin? Decisión: sí, lo ignora.