# Tasks

## 1. Modo comando (TDD)

- [ ] 1.1 RED: tests de `clean_command` (elimina fences de Markdown y espacios) y de los conflictos de `--cmd`. Verificación: `cargo test` falla.
- [ ] 1.2 GREEN: flag `--cmd` con `conflicts_with_all`, rol de sistema del modo comando y `clean_command`. Verificación: `cargo test`.

## 2. Ejecución segura (TDD)

- [ ] 2.1 RED: tests de `should_execute` (todas las combinaciones: sin `--exec`, con `--yes`, TTY/no-TTY, confirmado/rechazado). Verificación: `cargo test` falla.
- [ ] 2.2 GREEN: flags `--exec`/`--yes` con `requires = "cmd"`, confirmación interactiva y decisión por `should_execute`. Verificación: `cargo test`.
- [ ] 2.3 GREEN: ejecución con `sh -c` y propagación del código de salida. Verificación: `cargo test` (comando que sale con un código conocido).

## 3. Documentación

- [ ] 3.1 Documentar `--cmd`/`--exec`/`--yes` y las advertencias de seguridad en `README.md`. Verificación: revisión.

## 4. Verificación final

- [ ] 4.1 `cargo fmt --check && cargo clippy -- -D warnings && cargo test` en verde.
- [ ] 4.2 `openspec validate command-mode` sin errores.
