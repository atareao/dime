# Design

## Context

Ver `proposal.md` y `plans/PLAN-003.md` (Fase 4). `Args` se define con `clap` derive y `main` resuelve la configuración, la entrada y (si procede) la consulta. `clap` está en la versión 4.6.

## Goals / Non-Goals

- Goal: imprimir scripts de completado para bash, zsh y fish.
- Non-Goal: instalar los scripts en el sistema.
- Non-Goal: soportar elvish/powershell en esta fase.

## Decisions

### D1. `clap_complete`

Se usa `clap_complete` 4 (compatible con `clap` 4). Se genera con `clap_complete::generate(shell, &mut Args::command(), "dime", &mut std::io::stdout())`.

- Alternativa: escribir los scripts a mano. Descartada: frágil ante cambios de flags.

### D2. Valor de `--completions`

`ValueEnum` propio `CompletionShell { Bash, Zsh, Fish }` (los tres del plan), mapeado a `clap_complete::Shell`. Un valor fuera del conjunto produce un error de uso de `clap` (código ≠ 0).

- Alternativa: reutilizar `clap_complete::Shell` (añade `elvish` y `powershell`). Descartada para ceñirse al plan; revisable en el futuro.

### D3. Ejecución temprana

La rama de `--completions` se ejecuta al inicio de `main`, antes de resolver o crear la configuración y antes de tocar `stdin`. El flag usa `conflicts_with_all = ["question", "question_flag", "list_models"]`.

## Risks / Trade-offs

- [Nueva dependencia] → `clap_complete` es el crate oficial de `clap`; coste y superficie bajos.

## Migration Plan

1. Añadir `clap_complete` y el flag.
2. Generar e imprimir el script.
3. Tests, README, `fmt`/`clippy`, PR a `development`.

Rollback: revert del PR.

## Open Questions

- ¿Restringir a `bash|zsh|fish` o exponer todos los shells de `clap_complete`? Decisión: los tres del plan.
