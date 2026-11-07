# Design

## Context

Ver `proposal.md` y `plans/PLAN-003.md` (Fase 7). `IABot` en `src/ia.rs` guarda `base_url`, `endpoint`, `models_endpoint`, `token`, `use_https` y construye las URLs con `build_url_for`. Sus campos son privados, así que los tests de integración no pueden instanciarlo con una `base_url` arbitraria; en cambio, el binario acepta un `dime.yml` que sí lo permite (patrón ya usado en `tests/scripting_output.rs`).

## Goals / Non-Goals

- Goal: cubrir la ruta real de red (`ask` y `list_models`), incluidas las respuestas de error.
- Non-Goal: tests unitarios adicionales de `build_body`/`parse_response` (ya existen).
- Non-Goal: sustituir los servidores caseros de `tests/scripting_output.rs` (su función es distinta: comprobar la higiene de `stdout`).

## Decisions

### D1. `wiremock` como `[dev-dependencies]`

`wiremock` 0.6 arranca un servidor HTTP en un puerto efímero y permite definir *mocks* con matchers de método, ruta, cabeceras y cuerpo, además de responder códigos y cuerpos arbitrarios. Se elige frente a `httpmock` por su integración directa con `tokio` (ya presente) y por sus matchers de cabecera.

- Alternativa: escribir servidores con `TcpListener` a mano. Descartada para esta fase: no permite afirmar cabeceras con comodidad ni simular varios códigos de estado sin duplicar código.

### D2. Prueba de extremo a extremo mediante el binario

Los tests escriben un `dime.yml` temporal con `base_url` = dirección del mock, `use_https: false`, el `endpoint` y el `token` deseados, y ejecutan el binario con `std::process::Command` (como en `tests/scripting_output.rs`). Así se cubre la ruta real: clap → configuración → `ask` → `reqwest` → render.

Motivo: los campos de `IABot` son privados y exponer *setters* solo para tests ampliaría la API pública sin necesidad.

### D3. Qué se afirma de cada petición

- **Chat**: método `POST`, ruta, `Authorization: Bearer <token>` cuando el token no está vacío y ausencia de la cabecera cuando está vacío, y cuerpo con `model`, `messages[0].role == "system"`, `messages[1].role == "user"` y `temperature`.
- **Modelos**: petición a la ruta del `models_endpoint` y presencia de los identificadores en `stdout`.
- **Errores**: `401` y `500` producen código de salida distinto de cero, mensaje en `stderr` y nada del modelo en `stdout`; un cuerpo sin el campo esperado o que no es JSON también falla de forma legible y sin *panic*.

### D4. Determinismo

Cada test usa su propio `MockServer` (puerto efímero) y su propio directorio temporal con `dime.yml`, limpiado al terminar mediante un guardián con `Drop`. No se toca la red externa.

## Risks / Trade-offs

- [`wiremock` arrastra dependencias de desarrollo] → Solo afectan a `cargo test`; el binario publicado no cambia.
- [Los tests lanzan el binario, no la librería] → Es más lento que un test unitario, pero es el único modo de cubrir la ruta real sin abrir la API de `IABot`.
- [Puertos efímeros] → `MockServer::start()` los gestiona; no hay colisiones fijas.

## Migration Plan

1. Añadir `wiremock` a `[dev-dependencies]`.
2. Crear `tests/http_mock.rs` con los escenarios de chat, modelos y errores.
3. `fmt`/`clippy`/`test`, nota en `README.md`, PR a `development`.

Rollback: borrar el fichero de test y la dependencia.

## Open Questions

- ¿Migrar después `tests/scripting_output.rs` a `wiremock`? Fuera de alcance; se anota como *follow-up*.
