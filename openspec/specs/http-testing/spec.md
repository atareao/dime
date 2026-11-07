# http-testing Specification

## Purpose
TBD - created by archiving change http-mock-tests. Update Purpose after archive.

## Requirements

### Requirement: Petición de chat verificada contra servidor local

Los tests de integración DEBEN ejecutar el binario `dime` contra un servidor HTTP local y verificar la petición de chat realmente enviada: método `POST`, ruta del `endpoint` configurado, cabecera `Authorization: Bearer <token>` cuando el token no está vacío, y cuerpo JSON con `model`, `messages` (`system` y `user`) y la `temperature`. La respuesta del servidor DEBE aparecer en `stdout` junto con el código de salida cero.

#### Scenario: Petición de chat con autorización

- Given un servidor mock local y un `dime.yml` con `base_url` apuntando a él, `use_https: false` y un token no vacío
- When se ejecuta `dime -q "hola"`
- Then el servidor recibe un `POST` en la ruta del `endpoint` con `Authorization: Bearer <token>`
- And el cuerpo contiene el `model` configurado y los mensajes `system` y `user`
- And `stdout` contiene la respuesta simulada y el proceso termina con código 0

#### Scenario: Sin token no se envía autorización

- Given un servidor mock local y un `dime.yml` con token vacío
- When se ejecuta `dime -q "hola"`
- Then la petición no incluye la cabecera `Authorization`

### Requirement: Listado de modelos verificado contra servidor local

Los tests DEBEN verificar que `--list-models` consulta la ruta del `models_endpoint` configurado y presenta los identificadores devueltos por el servidor.

#### Scenario: Listado de modelos

- Given un servidor mock local que responde con una lista de modelos
- When se ejecuta `dime --list-models`
- Then el servidor recibe una petición en la ruta del `models_endpoint`
- And `stdout` contiene los identificadores de modelo devueltos
- And el proceso termina con código 0

### Requirement: Errores HTTP y respuestas inesperadas

Los tests DEBEN verificar que una respuesta `4xx`/`5xx` o un cuerpo con forma inesperada producen un error legible por `stderr`, código de salida distinto de cero y ningún *panic*.

#### Scenario: Respuesta 401

- Given un servidor mock que responde `401 Unauthorized`
- When se ejecuta `dime -q "hola"`
- Then el proceso termina con código distinto de cero
- And `stderr` contiene un mensaje de error legible
- And `stdout` no contiene la respuesta del modelo

#### Scenario: Respuesta 500

- Given un servidor mock que responde `500 Internal Server Error`
- When se ejecuta `dime -q "hola"`
- Then el proceso termina con código distinto de cero y sin *panic*

#### Scenario: Cuerpo sin el campo esperado

- Given un servidor mock que responde `200 OK` con un JSON sin el contenido esperado
- When se ejecuta `dime -q "hola"`
- Then el proceso termina con código distinto de cero y `stderr` describe el problema

#### Scenario: Cuerpo que no es JSON

- Given un servidor mock que responde `200 OK` con texto plano
- When se ejecuta `dime -q "hola"`
- Then el proceso termina con código distinto de cero y sin *panic*

### Requirement: Sin dependencias de runtime añadidas

`wiremock` DEBE declararse únicamente en `[dev-dependencies]`, de modo que el binario publicado no incorpore código de simulación y los tests no dependan de la red externa.

#### Scenario: Solo dependencia de desarrollo

- Given el `Cargo.toml` del proyecto
- When se inspecciona la sección de dependencias
- Then `wiremock` figura en `[dev-dependencies]` y no en `[dependencies]`
