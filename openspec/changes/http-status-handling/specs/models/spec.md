# models

## ADDED Requirements

### Requirement: El listado de modelos respeta el código de estado

Al consultar el listado de modelos, el sistema DEBE comprobar el código de estado HTTP y tratar cualquier código no exitoso (`>= 400`) como error tipado, con el mismo criterio que la petición de chat: mensaje del proveedor si el cuerpo contiene `error`, y en su defecto un error que incluya el código. Un cuerpo con forma de listado NO DEBE aceptarse si el estado no es exitoso. Nunca DEBE producirse un *panic*.

#### Scenario: 500 con listado válido

- Given un proveedor que responde `500` con un cuerpo que contiene `data` válida
- When se ejecuta `--list-models`
- Then la operación falla con un error tipado que menciona el código `500`
- And no se imprimen identificadores de modelo

#### Scenario: 401 sin error del proveedor

- Given un proveedor que responde `401` sin el campo `error`
- When se ejecuta `--list-models`
- Then la operación falla con un error tipado que menciona el código `401`
