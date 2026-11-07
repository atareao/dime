## ADDED Requirements

### Requirement: Entrada desde stdin

El sistema DEBE obtener la pregunta desde `stdin` cuando no se aporta `-q`/posicional y `stdin` no es un TTY. El valor `-` como pregunta DEBE forzar la lectura desde `stdin`. Cuando se aporte pregunta y `stdin` no sea un TTY y contenga datos, el contenido de `stdin` DEBE añadirse como contexto al mensaje de usuario. Si `stdin` no es un TTY, no contiene datos y no hay pregunta, el sistema DEBE mostrar un error legible y terminar con código distinto de cero, sin realizar ninguna petición.

#### Scenario: Pregunta sólo desde stdin

- Given que no se aporta `-q`/posicional y `stdin` recibe `hola` por tubería
- When se ejecuta `dime`
- Then la pregunta es `hola`

#### Scenario: Marcador `-`

- Given que `stdin` recibe `hola` por tubería
- When se ejecuta `dime -`
- Then la pregunta es `hola`

#### Scenario: Pregunta más contexto por stdin

- Given la pregunta `resume` y `stdin` con el contenido de un diff
- When se ejecuta `dime "resume"" con `stdin` canalizado
- Then el mensaje de usuario contiene la pregunta y el contexto de `stdin`

#### Scenario: stdin sin datos ni pregunta

- Given que `stdin` no es un TTY, no aporta datos y no hay pregunta
- When se ejecuta `dime`
- Then se muestra un error legible y el proceso termina con código distinto de cero

#### Scenario: Uso interactivo sin pregunta

- Given que `stdin` es un TTY y no se aporta pregunta
- When se ejecuta `dime`
- Then se muestra el error de uso y el proceso termina con código distinto de cero, sin leer de `stdin`