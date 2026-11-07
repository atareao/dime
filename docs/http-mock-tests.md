# Feature: tests de integración HTTP con servidor mock

## Qué es

Cobertura de la **ruta real de red** de `dime`: `tests/http_mock.rs` arranca un servidor `wiremock` local, escribe un `dime.yml` temporal que apunta a él y ejecuta el binario completo para verificar qué petición se envía de verdad (método, ruta, cabecera `Authorization` y cuerpo) y cómo se comporta ante respuestas `4xx`/`5xx` o formas inesperadas. Corresponde a la **Fase 7** de `PLAN-003` (cambio OpenSpec `http-mock-tests`, PR #37 squash → `development`). Añade la spec `http-testing` (4 requisitos). No cambia el comportamiento del producto: `wiremock` es una dependencia **solo de desarrollo**.

## Ficheros implicados

- `tests/http_mock.rs` — 11 tests de integración sobre el binario real (`CARGO_BIN_EXE_dime`) con un `MockServer` en un puerto efímero.
- `Cargo.toml` — `wiremock = "0.6.5"` únicamente en `[dev-dependencies]` (no en `[dependencies]`).
- `README.md` — nota de desarrollo sobre la suite de tests.
- Spec: `openspec/specs/http-testing` (4 requisitos).

## Cómo funciona

- **Arranque:** cada test crea un `MockServer::start()` (puerto efímero) y monta la respuesta esperada con `Mock::given(method(...)).and(path(...))`.
- **Configuración temporal:** `write_config` escribe en un directorio temporal un `dime.yml` con `base_url` = `host:puerto` del mock, `use_https: false`, `endpoint`/`models_endpoint`, `temperature`, `system_role` y `log_level: error`. El directorio se limpia con un guardián `TempDir` (`Drop`).
- **Ejecución:** `run_dime` lanza el binario con `std::process::Command`, `current_dir` en el directorio temporal y `stdin` cerrado (`Stdio::null()`, nunca un TTY).
- **Escenarios cubiertos:**
  - **Chat (openai):** `POST` a la ruta del `endpoint`, `Authorization: Bearer <token>`, cuerpo con `model`, `messages[0]` (`system`), `messages[1]` (`user`) y `temperature`; la respuesta aparece en `stdout` y el proceso sale con 0.
  - **Chat sin token (ollama):** `provider: ollama` con token vacío sigue haciendo la petición y **no** envía `Authorization`.
  - **Forma del cuerpo ollama:** `options.num_ctx`, `options.temperature`, ausencia de `temperature` en la raíz y `content` del mensaje `user` como cadena.
  - **`--list-models`:** `GET` al `models_endpoint`, misma autenticación, ids en `stdout` y salida 0; con `401`, error legible y **ningún id** en `stdout`.
  - **Errores:** `401`, `500`, `200` sin `choices` y `200` con cuerpo no JSON → código distinto de cero, `stderr` legible, sin *panic* y con la petición anclada mediante `verify()`/`assert_single_chat_request` (evita que un `404` por defecto haga pasar el test sin ejercitar el escenario).
  - **Validación local:** `provider: openai` con token vacío aborta **antes** de la red (el mock no recibe ninguna petición).
  - **Solo dev-dependency:** un test inspecciona `Cargo.toml` y exige `wiremock` en `[dev-dependencies]` y no en `[dependencies]`.
- **Helpers:** `assert_readable_error` (exige el prefijo `Error:` y la ausencia de `panicked`, sin acoplarse al texto exacto) y `assert_single_chat_request` (método `POST` y ruta esperada).

### Hueco que cierra

- Los tests unitarios de `src/ia.rs` validan `build_body` y `parse_response` de forma **pura**: comprueban la lógica, pero no que el binario use esos valores al hablar con la red.
- `tests/scripting_output.rs` cubre el **proceso completo** (incluido un mock), pero no afirma de forma exhaustiva método, ruta, cabecera, cuerpo exacto ni la matriz de errores `4xx`/`5xx`/cuerpo inesperado.
- `tests/http_mock.rs` cubre esa franja: la petición y la respuesta **reales en el cable**, con el binario y su `clap`/config/`reqwest`/render, sin exponer la API interna de `IABot` y sin tocar la red externa.

## Configuración

- No requiere variables de entorno ni claves nuevas: los tests generan su propio `dime.yml` temporal.
- Campos usados en ese fichero: `provider`, `model`, `token`, `base_url`, `use_https`, `endpoint`, `models_endpoint`, `temperature`, `system_role`, `log_level`.
- `base_url` apunta a `127.0.0.1:<puerto efímero>` y `use_https: false` evita TLS.

## Cómo habilitar / desplegar

No hay activación en el producto: solo afecta a la suite de tests.

```bash
# Toda la suite (incluye los tests HTTP mock)
cargo test

# Solo esta suite
cargo test --test http_mock

# Un escenario concreto
cargo test --test http_mock chat_request_has_method_path_token_and_expected_body
```

- `wiremock` está en `[dev-dependencies]`, por lo que `cargo build --release` y el binario publicado no lo incorporan (`cargo tree -e normal` no lo muestra).
- Los tests no dependen de la red externa ni de puertos fijos (el servidor usa un puerto efímero).

## Decisiones

- **Ejecutar el binario**, no instanciar `IABot`: sus campos son privados y el binario acepta un `dime.yml` con `base_url` arbitraria (patrón ya usado en `tests/scripting_output.rs`).
- **`wiremock` como dev-dependency**: simulación local sin añadir peso ni superficie al binario.
- **Afirmar la petición completa** (método, ruta, cabecera, cuerpo) en lugar de solo el resultado: detecta regresiones de protocolo que un test de salida no vería.
- **Anclar los tests de error** con `assert_single_chat_request`/`verify()`: un `404` por defecto del mock también daría código distinto de cero, así que se exige que la petición haya llegado a la ruta correcta.
- **Mensajes de error sin acoplar la redacción**: `assert_readable_error` solo exige el prefijo `Error:` y la ausencia de `panic`.
- **Directorio temporal con guardián `Drop`**: aislamiento por test y limpieza automática.

## Follow-ups

- **`ask()` no inspecciona `response.status()`**: hoy un `500` con un cuerpo de chat válido se trataría como éxito. Los tests actuales usan cuerpos de error coherentes; conviene endurecer `ask()` y añadir el caso «500 con cuerpo válido».
- Valorar un escenario de timeout/red caída (hoy no cubierto).
- Valorar `--list-models` con respuesta malformada (más allá del `401`).
- Reutilizar los helpers (`write_config`, `run_dime`, `assert_readable_error`) si se añaden nuevas suites de integración.
