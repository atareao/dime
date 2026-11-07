# Tasks

## 1. Adjuntos de fichero (TDD)

- [ ] 1.1 RED: tests de que `--file` incluye el contenido en el mensaje de usuario, respeta el orden y falla con fichero inexistente. Verificación: `cargo test` falla.
- [ ] 1.2 GREEN: flag `--file` (repetible) y composición del contexto en `src/ia.rs`/`src/main.rs`. Verificación: `cargo test`.

## 2. Imágenes multimodales (TDD)

- [ ] 2.1 RED: tests de `openai` (content parts con data URL `data:image/png;base64,`) y de `ollama` (`images` como base64, `content` texto); imagen inexistente → error. Verificación: `cargo test` falla.
- [ ] 2.2 GREEN: codificación base64 + MIME por extensión, flag `--image` (repetible) y construcción del cuerpo por proveedor. Verificación: `cargo test`.

## 3. Portapapeles (TDD)

- [ ] 3.1 RED: tests de que `--copy` copia el contenido (y `content` en `--json`) y de que un portapapeles no disponible advierte sin fallar. Verificación: `cargo test` falla.
- [ ] 3.2 GREEN: `--copy` con `arboard` y `copy_to_clipboard` aislada. Verificación: `cargo test`.

## 4. Documentación

- [ ] 4.1 Documentar `--file`, `--image` y `--copy` en `README.md` con ejemplos. Verificación: revisión.

## 5. Verificación final

- [ ] 5.1 `cargo fmt --check && cargo clippy -- -D warnings && cargo test` en verde.
- [ ] 5.2 `openspec validate context-input` sin errores.
