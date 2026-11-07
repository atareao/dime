# Tasks

## 1. Flag y generación (TDD)

- [ ] 1.1 RED: tests de que `--completions <bash|zsh|fish>` produce un script no vacío y exit 0; shell inválido → error de uso; incompatible con pregunta y `--list-models`. Verificación: `cargo test` falla.
- [ ] 1.2 GREEN: añadir `clap_complete` y el `ValueEnum` `CompletionShell`; flag `--completions` con `conflicts_with_all`. Verificación: `cargo test`.
- [ ] 1.3 GREEN: generar el script con `clap_complete::generate` al inicio de `main`, antes de resolver/crear configuración. Verificación: test de que no se crea `dime.yml`.

## 2. Documentación

- [ ] 2.1 Documentar `--completions` y el uso (`. <(dime --completions bash)`) en `README.md`. Verificación: revisión.

## 3. Verificación final

- [ ] 3.1 `cargo fmt --check && cargo clippy -- -D warnings && cargo test` en verde.
- [ ] 3.2 `openspec validate shell-completions` sin errores.
