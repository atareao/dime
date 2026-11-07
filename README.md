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
dime --json <PREGUNTA>
dime --no-color <PREGUNTA>
dime --model <MODELO> <PREGUNTA>
dime --temperature <NÚMERO> <PREGUNTA>
dime --provider <openai|ollama> <PREGUNTA>
dime --system <INSTRUCCIONES> <PREGUNTA>
dime --file <RUTA> <PREGUNTA>
dime --image <RUTA> <PREGUNTA>
dime --copy <PREGUNTA>
dime --completions <bash|zsh|fish>
dime --cmd <OBJETIVO>
dime --cmd <OBJETIVO> --exec
dime --cmd <OBJETIVO> --exec --yes
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

### Adjuntar contexto e imágenes

`--file <RUTA>` (repetible) añade el contenido de uno o más ficheros como
contexto del mensaje de usuario, **después** del contexto de `stdin` y
respetando el orden indicado. Cada fichero se precede de una cabecera con su
ruta:

```bash
# Un fichero como contexto
dime --file README.md "resume este fichero"

# Varios ficheros, en orden
dime --file src/main.rs --file Cargo.toml "revisa esta configuración"
```

`--image <RUTA>` (repetible) adjunta imágenes multimodales. El tipo MIME se
deduce de la extensión (`png`, `jpg`/`jpeg`, `gif`, `webp`). Con `openai` la
imagen se envía como *content part* con un *data URL*
(`data:<mime>;base64,<datos>`); con `ollama` se envía en el campo `images` del
mensaje:

```bash
# Analizar una captura de pantalla
dime --image captura.png "explica qué muestra esta captura"

# Varias imágenes
dime --image antes.png --image despues.png "describe las diferencias"
```

Un fichero inexistente/no UTF-8 o una imagen ilegible/con extensión no
soportada muestran un error legible y terminan con código distinto de cero,
**sin** realizar la petición.

### Copiar la respuesta al portapapeles

Con `--copy`, la respuesta se copia al portapapeles además de imprimirse. Es
*best-effort*: si el portapapeles no está disponible (por ejemplo en un entorno
*headless*), se advierte por `stderr` y el comando termina con código `0`. En
modo `--json` se copia el campo `content`.

```bash
# Copiar la respuesta para pegarla en otro sitio
dime --copy "escribe un correo de disculpa"

# Con salida JSON: se copia el campo `content`
dime --copy --json "resume este texto" | jq -r '.content'
```

> **Limitación en X11:** en X11 **sin** un *clipboard manager* (por ejemplo,
> `clipman`, `klipper`, `CopyQ`…), el propietario del contenido es el propio
> proceso; al salir `dime`, el texto no persiste en el portapapeles. Es una
> limitación de X11, no de `dime`, y no afecta a Wayland, macOS ni Windows, ni a
> X11 con un *clipboard manager* en ejecución. Mientras `dime` está vivo, el
> contenido sí es accesible (se conserva el handle del portapapeles hasta el
> final del programa).
>
> Nota: `arboard` puede emitir por `stderr` un aviso (`Clipboard was dropped
> very quickly after writing`) en algunos gestores; es inofensivo.

### Modo comando y ejecución segura

Con `--cmd <OBJETIVO>`, **dime** pide al modelo *únicamente* el comando de shell
que cumple el objetivo (rol de sistema específico, sin explicaciones) e imprime
el comando limpio: elimina los bloques de código Markdown (```` ```bash ````,
```` ``` ````) y los espacios sobrantes. En este modo `stdin` **no** se usa como
contexto, para no chocar con la confirmación de `--exec`.

```bash
# Sólo imprime el comando: NUNCA lo ejecuta
dime --cmd "listar los ficheros .log modificados hoy"

# Lo imprime y pide confirmación interactiva antes de ejecutarlo
dime --cmd "listar los ficheros .log modificados hoy" --exec

# Lo imprime y lo ejecuta sin preguntar (sin TTY, para scripts)
dime --cmd "listar los ficheros .log modificados hoy" --exec --yes
```

> ⚠️ **Seguridad:** ejecutar comandos generados por un modelo es peligroso.
> `dime` aplica estas garantías:
>
> - **Sin `--exec` no se ejecuta nada**: sólo se imprime el comando.
> - El comando **siempre se muestra** antes de cualquier ejecución.
> - La confirmación interactiva tiene **respuesta por defecto negativa**:
>   vacío, `n` o `no` no ejecutan (sólo `y`/`yes`/`s`/`si`/`sí` confirman).
> - Si `stdin` **no es un TTY** y no se pasa `--yes`, `dime` se niega a ejecutar
>   y termina con código distinto de cero.
> - `--exec` y `--yes` **requieren** `--cmd`; `--cmd` es incompatible con la
>   pregunta (`-q`/posicional), `--list-models` y `--completions`.
> - **Prompt injection:** combinar `--file`/`--image` con `--exec --yes` permite
>   que el contenido adjunto incluya instrucciones que el modelo traduzca en un
>   comando: `dime` ejecutará ese comando **sin preguntar**. No uses
>   `--exec --yes` con contenido no confiable (diffs, páginas web, documentos de
>   terceros); revisa siempre el comando impreso.
> - No hay *sandboxing*: el comando se ejecuta con `sh -c` con tus permisos. El
>   usuario es responsable de lo que ejecuta.

En modo `--cmd` la salida es siempre el comando limpio (se conservan los saltos
de línea internos de los comandos multilínea); `--json` no se aplica.

Con `--exec`, el comando se ejecuta con `sh -c` heredando `stdout`/`stderr`, y su
código de salida se propaga como código de salida de `dime`:

```bash
dime --cmd "comprueba si existe /etc/hosts" --exec --yes
echo $?   # código de salida del comando ejecutado
```

### Argumentos

| Argumento        | Forma Corta | Descripción                                                                                                                                     |
| :--------------- | :---------- | :---------------------------------------------------------------------------------------------------------------------------------------------- |
| `--instructions` | `-i`        | Instrucciones de comportamiento para el modelo (rol del sistema). Opcional; si no se indica, se usa `system_role` del archivo de configuración. Alias: `--system`. |
| `--system`       |             | Alias de `-i/--instructions` (mismo argumento).                                                                                                  |
| `--question`     | `-q`        | La pregunta o solicitud que deseas realizar. Opcional si se pasa como argumento posicional o si se lee de `stdin`.                                    |
| `--list-models`  |             | Lista los modelos disponibles del proveedor (uno por línea) y termina. No requiere pregunta e ignora `stdin`.                                     |
| `--json`         |             | Emite la respuesta como un único objeto JSON en `stdout` con `content`, `model` y `usage` (o `null`).                                            |
| `--no-color`     |             | Garantiza que la salida no contiene secuencias de escape ANSI.                                                                                   |
| `--file`         |             | Adjunta el contenido de un fichero como contexto del mensaje (repetible; se respeta el orden). Un fichero inexistente o no UTF-8 produce un error legible y código de salida distinto de cero. |
| `--image`        |             | Adjunta una imagen multimodal (repetible). Formatos: `png`, `jpg`/`jpeg`, `gif`, `webp`. Con `openai` se envían como *content parts* (`data:<mime>;base64,...`); con `ollama` en el campo `images`. Una imagen ilegible o con extensión no soportada produce un error legible y código distinto de cero. |
| `--copy`         |             | Copia la respuesta al portapapeles además de imprimirla. Es *best-effort*: si el portapapeles no está disponible (p. ej. entorno *headless*), se advierte por `stderr` y el proceso termina con código `0`. En modo `--json` se copia el campo `content`. |
| `--completions`  |             | Imprime en `stdout` el script de autocompletado del shell indicado (`bash`, `zsh` o `fish`) y termina. No requiere pregunta, no lee `stdin` ni crea `dime.yml`. Incompatible con la pregunta y con `--list-models`. |
| `--cmd`          |             | Pide al modelo **únicamente** el comando que cumple el objetivo e imprime el comando limpio (sin fences de Markdown). **Nunca lo ejecuta** salvo con `--exec`. No usa `stdin` como contexto. Incompatible con la pregunta, `--list-models` y `--completions`. |
| `--exec`         |             | Ejecuta el comando de `--cmd` con `sh -c` tras una confirmación interactiva (por defecto **negativa**) y propaga su código de salida. Requiere `--cmd`. Sin `--exec` no se ejecuta nada. En un `stdin` que no es TTY exige `--yes`. |
| `--yes`          |             | Con `--exec`, ejecuta sin pedir confirmación. Requiere `--cmd`. Sin `--exec` no ejecuta nada. |
| `--model`        |             | Sobreescribe `model` de `dime.yml` **sólo para esta invocación**. No reescribe el fichero.                                                        |
| `--temperature`  |             | Sobreescribe `temperature` de `dime.yml` **sólo para esta invocación**. Debe ser numérica; en caso contrario `clap` da un error de uso.           |
| `--provider`     |             | Sobreescribe `provider` de `dime.yml` **sólo para esta invocación**. Valores válidos: `openai`, `ollama` (insensible a mayúsculas).               |
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

### Sobreescribir la configuración para una invocación

Los flags `--model`, `--temperature` y `--provider` ajustan la configuración
**sólo para la invocación en curso, sin modificar ni reescribir `dime.yml`**. Si
no se pasa un flag, se usa el valor del fichero (valores efectivos):

```bash
# Probar otro modelo puntualmente
dime --model gpt-4o "A que huelen las nubes"

# Bajar la temperatura para una respuesta más determinista
dime --temperature 0.2 -q "dame solo el comando bash"

# Usar Ollama sin tocar dime.yml (no requiere token)
dime --provider ollama "A que huelen las nubes"
```

Notas:

- `--temperature` debe ser numérica; un valor no numérico (`--temperature alta`)
  produce un error de uso y el proceso termina con código distinto de cero, sin
  realizar ninguna petición.
- `--provider` sólo admite `openai` u `ollama` (case-insensitive). Un valor
  desconocido produce un error legible y termina con código distinto de cero.
- Con `--provider ollama` el cuerpo de la petición usa `options.num_ctx` y no
  incluye la cabecera `Authorization`; además no se exige token.
- Los overrides **no** cambian el rol del sistema: eso sólo lo hacen
  `-i/--instructions` o su alias `--system`.

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

### Salida para scripting

Con `--json`, `dime` escribe en `stdout` **exactamente** un objeto JSON válido
con las claves `content` (la respuesta), `model` (el modelo usado) y `usage` (el
objeto de uso del proveedor, o `null` si no se aporta). `model` es siempre una
cadena: si el proveedor no informa el modelo, se usa el configurado en
`dime.yml`. No se imprime ningún otro texto en `stdout` (los logs de `tracing`
van a `stderr`), de modo que puedes encadenarlo con `jq`:

```bash
dime --json "A que huelen las nubes" | jq -r '.content'
dime --json -q "resume este error" | jq '.usage.total_tokens'
```

`--json` no es compatible con `--list-models`.

En modo `--json`, si se produce un error, el mensaje legible se escribe en
`stderr` y el proceso termina con un código de salida distinto de cero; `stdout`
queda sin JSON parcial.

Con `--no-color` se garantiza que la salida no contiene secuencias de escape
ANSI. El color de los logs se aplica automáticamente solo si `stderr` es un TTY:
al canalizar o redirigir la salida nunca se emiten códigos ANSI.

### Autocompletado de shell

Con `--completions` puedes generar el script de autocompletado de **dime** para
`bash`, `zsh` o `fish`. El script se imprime en `stdout` y el proceso termina con
código 0; no requiere pregunta, no lee `stdin` y no crea ni necesita `dime.yml`.
Por ello es ideal para instalarlo en el arranque del shell:

```bash
# Bash (sesión actual)
. <(dime --completions bash)

# Bash (instalación permanente, requiere permisos)
dime --completions bash | sudo tee /etc/bash_completion.d/dime > /dev/null

# Zsh
dime --completions zsh > ~/.zfunc/_dime

# Fish
dime --completions fish > ~/.config/fish/completions/dime.fish
```

`--completions` no es compatible con la pregunta (posicional o `-q`) ni con
`--list-models`: combinarlos produce un error de uso. Un shell no soportado
(p. ej. `--completions elvish`) también produce un error de uso.

## 🛠️ Tecnologías

**dime** está construido íntegramente en Rust, aprovechando las siguientes librerías:

- **tokio:** Para la ejecución asíncrona.
- **reqwest:** Para realizar las peticiones HTTP seguras a la API.
- **clap:** Para el análisis y gestión de los argumentos de línea de comandos.
- **serde** y **serde_norway/serde_json:** Para la serialización y deserialización de la configuración y las respuestas de la API.
- **tracing:** Para la gestión de logs y depuración.
- **dirs:** Para localizar rutas de configuración y _home_ en distintos sistemas operativos.

## 🧪 Desarrollo y pruebas

La suite se ejecuta con `cargo test`. Además de los tests unitarios, los tests de
integración lanzan el binario completo contra servidores HTTP locales en un
puerto efímero, así que **no necesitan red externa** ni credenciales.

- **`wiremock`** es una dependencia **de desarrollo** (sólo `[dev-dependencies]`):
  no se compila en el binario publicado. `tests/http_mock.rs` la usa para
  verificar la ruta real de red: método, ruta, cabecera `Authorization` y cuerpo
  de la petición de chat, la petición de `--list-models`, y el comportamiento
  ante `401`, `500`, cuerpos sin el campo esperado y cuerpos que no son JSON.
- Para filtrar sólo esas pruebas: `cargo test --test http_mock`.

Verificación completa antes de abrir un PR:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
