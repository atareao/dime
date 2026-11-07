# clipboard

## ADDED Requirements

### Requirement: Copiar la respuesta al portapapeles

Con `--copy`, el sistema DEBE copiar la respuesta al portapapeles además de imprimirla. En modo `--json` DEBE copiarse el campo `content`. Si el portapapeles no está disponible, el sistema DEBE advertir por `stderr` y continuar sin fallar (la respuesta ya se ha impreso).

#### Scenario: Copia de la respuesta

- Given la ejecución con `--copy` y una respuesta correcta
- When se procesa la respuesta
- Then el contenido de la respuesta se copia al portapapeles y se imprime en `stdout`

#### Scenario: Copia con --json

- Given la ejecución con `--copy --json`
- When se procesa la respuesta
- Then se copia el campo `content` al portapapeles

#### Scenario: Portapapeles no disponible

- Given un entorno sin portapapeles disponible
- When se ejecuta con `--copy`
- Then se muestra una advertencia por `stderr` y el proceso termina con código de salida cero
