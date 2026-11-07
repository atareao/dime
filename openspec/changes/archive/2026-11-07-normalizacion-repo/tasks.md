# Tasks

## 1. Dependencias y base

- [x] 1.1 Añadir `thiserror` a `Cargo.toml`. Verificación: `cargo build` compila.
- [x] 1.2 Crear `HANDOFF.md` en la raíz. Verificación: el fichero existe con estado y tarea pendiente.

## 2. Configuración (`src/ia.rs`, `src/main.rs`)

- [x] 2.1 Eliminar `models_endpoint` y los métodos muertos (`get_base_url`, `set_token`, `save`). Verificación: `cargo clippy -- -D warnings` sin avisos de `dead_code`.
- [x] 2.2 `read_content` devuelve `Result<IABot, IABotError>` en lugar de terminar el proceso; corregir el mensaje `config.yml` → `dime.yml`. Verificación: test unitario con YAML inválido devuelve `Err` con la ruta correcta.
- [x] 2.3 Test de valores por defecto con YAML parcial. Verificación: `cargo test`.

## 3. Petición y respuesta (`src/ia.rs`)

- [x] 3.1 Extraer `build_body` con temperatura numérica y `num_ctx` por proveedor. Verificación: tests para `openai` y `ollama`.
- [x] 3.2 Tests de construcción: URL https/http, cabecera `Authorization` sólo con token, `num_ctx` en Ollama. Verificación: `cargo test`.
- [x] 3.3 DTOs `serde` y función de parseo `parse_response` (respuesta correcta, error del proveedor, respuesta inesperada). Verificación: tests unitarios.
- [x] 3.4 Temperatura inválida devuelve error legible. Verificación: test unitario.
- [x] 3.5 El valor del token no aparece en los logs. Verificación: test de que la cadena de log no contiene el token.

## 4. CLI (`src/main.rs`)

- [x] 4.1 `main` trata los `Result` de configuración y consulta con `process::exit(1)` en error, sin `unwrap`. Verificación: `cargo clippy -- -D warnings` sin warnings.

## 5. CI, repo y documentación

- [x] 5.1 `.gitignore` con `dime.yml`, `.dime.yml` y `config.yml`. Verificación: `git check-ignore -v dime.yml`.
- [x] 5.2 `rust.yml` en `main` y `development` con `fmt`, `clippy -D warnings`, `build` y `test`. Verificación: revisión del YAML y acción en verde en el PR.
- [x] 5.3 `release.yml` con `softprops/action-gh-release@v2`. Verificación: revisión del YAML.
- [x] 5.4 `.github/dependabot.yml` para `cargo` y `github-actions`. Verificación: revisión del YAML.
- [x] 5.5 Crear `docs/normalizacion-repo.md` (qué es, ficheros implicados, funcionamiento, configuración y follow-ups). Verificación: el fichero existe y cubre esos puntos.

## 6. Verificación final

- [x] 6.1 `cargo fmt --check && cargo clippy -- -D warnings && cargo test` en verde.
- [x] 6.2 `openspec validate normalizacion-repo` sin errores.