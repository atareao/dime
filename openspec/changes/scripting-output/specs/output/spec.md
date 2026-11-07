# output

## ADDED Requirements

### Requirement: Salida JSON

Con `--json`, el sistema DEBE escribir en `stdout` exactamente un objeto JSON válido con los campos `content` (cadena con la respuesta), `model` (cadena con el modelo usado) y `usage` (el objeto de uso del proveedor, o `null` si no se aporta). No DEBE escribir ningún otro texto en `stdout`.

#### Scenario: Respuesta con uso

- Given una respuesta con contenido `Hola`, modelo `gpt-4o` y un objeto `usage`
- When se ejecuta con `--json`
- Then `stdout` es un JSON válido con `content = "Hola"`, `model = "g1pt-4o"` y `usage` con los datos del proveedor

#### Scenario: Respuesta sin uso

- Given una respuesta sin campo `usage`
- When se ejecuta con `--json`
- Then el JSON tiene `usage` igual a `null`

#### Scenario: Error en modo JSON

- Given un error de API
- When se ejecuta con `--json`
- Then `stdout` no contiene JSON y `stderr` muestra un mensaje legible con código de salida distinto de cero

### Requirement: Control de color

El sistema NO DEBE emitir secuencias de escape ANSI cuando `stdout` no es un TTY o cuando se usa `--no-color`. Cuando `stdout` es un TTY y no se pasa `--no-color`, el sistema PUEDE usar color.

#### Scenario: `--no-color`

- Given la ejecución con `--no-color`
- When se imprime la respuesta
- Then la salida no contiene códigos ANSI

#### Scenario: Salida canalizada

- Given `stdout` redirigido a un fichero o a una tubería
- When se ejecuta `dime`
- Then la salida no contiene códigos ANSI