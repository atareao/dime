# Proposal

## Why

`dime` no ofrece scripts de autocompletado, lo que obliga a recordar los flags y sus valores. Proporcionarlos mejora la ergonomía en bash, zsh y fish.

## What Changes

- Nuevo flag `--completions <bash|zsh|fish>` que imprime en `stdout` el script de completado del binario `dime` y termina, sin exigir pregunta, sin leer `stdin` y sin requerir un fichero de configuración (no crea `dime.yml`).
- Nueva dependencia `clap_complete`.
- `--completions` es incompatible con la pregunta y con `--list-models`.

## Capabilities

### New Capabilities
- `completions`: generación de scripts de autocompletado de shell.

### Modified Capabilities
<!-- Ninguna. -->

## Impact
- `Cargo.toml` (`clap_complete`), `src/main.rs` (flag y generación), `README.md`.
