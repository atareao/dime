# dime: Tu Asistente de IA en la Línea de Comandos

**dime** es una herramienta simple y eficiente, diseñada para desarrolladores y usuarios avanzados de Linux, que te permite enviar consultas rápidas a modelos de inteligencia artificial (como ChatGPT) configurando el comportamiento del sistema y la pregunta, todo desde la terminal.

Ideal para automatizar tareas, obtener comandos rápidos o recibir resúmenes contextualizados sin salir de tu flujo de trabajo.

## 🚀 Instalación

Dado que **dime** está desarrollado en **Rust**, la forma más sencilla de instalarlo es usando `cargo`:

```bash
cargo install dime
```

Alternativamente, puedes compilarlo desde el código fuente:

```bash
git clone https://github.com/atareao/dime.git
cd dime
cargo build --release
# El ejecutable se encontrará en target/release/dime
```

## ⚙️ Configuración

Antes de usar `dime`, debes configurar tu clave API en el archivo de configuración, típicamente llamado `dime.yml` o `.dime.yml`.

Al ejecutar `dime` por primera vez sin un archivo de configuración, se intentará crear uno por defecto llamado `dime.yml` en el directorio actual.

**dime** busca su archivo de configuración (`dime.yml`) en el siguiente orden:

1.  Directorio actual.
2.  Directorio del ejecutable.
3.  `~/.dime.yml` (en el directorio _home_ del usuario).
4.  Directorio de configuración del sistema (e.g., `~/.config/dime.yml`).
5.  Directorio de configuración específico para la aplicación (e.g., `~/.config/dime/dime.yml`).

### Estructura de `dime.yml`

El archivo de configuración utiliza formato `YAML` e incluye los parámetros necesarios para la API de OpenAI.

| Parámetro     | Valor por Defecto       | Descripción                                                          |
| :------------ | :---------------------- | :------------------------------------------------------------------- |
| `log_level`       | `"info"`                | Nivel de registro para la salida de `tracing`.                       |
| `base_url`        | `"api.openai.com"`      | URL base para la API.                                                |
| `endpoint`        | `"v1/chat/completions"` | Endpoint para las consultas.                                         |
| `models_endpoint` | `"v1/models"`           | Endpoint para listar los modelos disponibles con `--list-models`.    |
| `token`           | `""`                    | **Tu clave API de OpenAI.** (Obligatorio)                            |
| `model`           | `"gpt-3.5-turbo"`       | El modelo de IA a utilizar.                                          |
| `temperature`     | `1.0`                   | Temperatura de muestreo; número o cadena numérica (ej. `0.2`).       |
| `num_ctx`         | `32000`                 | Ventana de contexto enviada en `options.num_ctx` cuando usas Ollama. |

> **Nota sobre `temperature`:** el formato preferido es un número (`temperature: 0.2`),
> pero también se acepta una cadena numérica (`temperature: "0.2"`). Un valor no
> numérico produce un error de configuración legible al cargar `dime.yml`.

> **IMPORTANTE:** Si el campo `token` en la configuración está vacío, el programa terminará con un error.

## 💻 Uso

La pregunta puede pasarse como argumento posicional, con `--question` o por `stdin` (tubería/redirección). El rol de la IA es opcional: si no se pasa `--instructions`, se usa `system_role` desde la configuración.

### Sintaxis

```bash
dime -q <PREGUNTA>
dime <PREGUNTA>
dime -i <INSTRUCCIONES> -q <PREGUNTA>
dime -i <INSTRUCCIONES> <PREGUNTA>
dime --list-models
```

### Entrada por tubería o redirección

Cuando no se aporta pregunta y `stdin` no es un terminal (TTY), **dime** lee la
pregunta de `stdin`. También puedes forzarlo con `-` como pregunta. Si aportas
la pregunta y además canalizas datos, el contenido de `stdin` se añade como
**contexto** al mensaje de usuario (el rol del sistema no cambia).

```bash
# La pregunta viene de stdin
echo "A que huelen las nubes" | dime

# Marcador `-` para forzar la lectura desde stdin
dime - < pregunta.txt

# Pregunta + contexto canalizado (el diff se añade como contexto)
git diff | dime "resume estos cambios"

# Redirección desde fichero como contexto
dime "explica" < README.md
```

Consideraciones:

- En uso **interactivo** (TTY) sin pregunta, se muestra un error de uso y el
  proceso termina con código distinto de cero, sin leer de `stdin`.
- Si `stdin` no es un TTY, no aporta datos (EOF inmediato) y no hay pregunta,
  se muestra un error legible y el proceso termina con código distinto de cero.
- `--list-models` **ignora** `stdin`: nunca se bloquea esperando datos.

### Argumentos

| Argumento        | Forma Corta | Descripción                                                                                                                                     |
| :--------------- | :---------- | :---------------------------------------------------------------------------------------------------------------------------------------------- |
| `--instructions` | `-i`        | Instrucciones de comportamiento para el modelo (rol del sistema). Opcional; si no se indica, se usa `system_role` del archivo de configuración. |
| `--question`     | `-q`        | La pregunta o solicitud que deseas realizar. Opcional si se pasa como argumento posicional o si se lee de `stdin`.                                    |
| `--list-models`  |             | Lista los modelos disponibles del proveedor (uno por línea) y termina. No requiere pregunta e ignora `stdin`.                                     |
| `QUESTION`       |             | La pregunta o solicitud que deseas realizar como argumento posicional. Usa `-` para forzar la lectura desde `stdin`.                            |

### Ejemplo

Define el rol de la IA como un experto en `bash` y pídele un comando:

```bash
dime -i "Eres un experto en shell script y solo respondes con el comando bash que se solicita, sin explicaciones." -q "Comando para buscar archivos .log modificados en los últimos 7 días y comprimirlos en un tar.gz"
```

Tambien puedes invocarlo sin `-q`:

```bash
dime "A que huelen las nubes"
```

Si prefieres no pasar el rol por línea de comandos, puedes fijarlo en `system_role` dentro de tu archivo `dime.yml`.

El modelo de IA procesará la solicitud con el rol definido y devolverá la respuesta directamente a la terminal.

Cuando el proveedor es `ollama`, `dime` envía `num_ctx` dentro de `options`, por ejemplo:

```json
{
  "options": {
    "num_ctx": 32000
  }
}
```

Si no defines `num_ctx` en tu `dime.yml`, se usará `32000` por defecto.

### Listar los modelos disponibles

Con la opción `--list-models` puedes consultar los modelos disponibles y verlos,
uno por línea, sin hacer ninguna pregunta:

```bash
dime --list-models
```

La consulta se realiza a `{protocolo}://{base_url}/{models_endpoint}` (por ejemplo,
`https://api.openai.com/v1/models` para OpenAI o `http://localhost:11434/api/tags`
para Ollama), incluyendo la cabecera `Authorization: Bearer <token>` cuando el
proveedor la requiere. El listado funciona tanto con el formato de OpenAI
(`v1/models`, respuesta `{"data":[{"id": ...}]}`) como con el de Ollama nativo
(`api/tags`, respuesta `{"models":[{"name": ...}]}`). Si se produce un error de
red o de API, se muestra un mensaje legible y el proceso termina con código
distinto de cero.

Puedes personalizar el endpoint con `models_endpoint` en tu `dime.yml`.

## 🛠️ Tecnologías

**dime** está construido íntegramente en Rust, aprovechando las siguientes librerías:

- **tokio:** Para la ejecución asíncrona.
- **reqwest:** Para realizar las peticiones HTTP seguras a la API.
- **clap:** Para el análisis y gestión de los argumentos de línea de comandos.
- **serde** y **serde_norway/serde_json:** Para la serialización y deserialización de la configuración y las respuestas de la API.
- **tracing:** Para la gestión de logs y depuración.
- **dirs:** Para localizar rutas de configuración y _home_ en distintos sistemas operativos.
