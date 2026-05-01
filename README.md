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
3.  `~/.dime.yml` (en el directorio *home* del usuario).
4.  Directorio de configuración del sistema (e.g., `~/.config/dime.yml`).
5.  Directorio de configuración específico para la aplicación (e.g., `~/.config/dime/dime.yml`).

### Estructura de `dime.yml`

El archivo de configuración utiliza formato `YAML` e incluye los parámetros necesarios para la API de OpenAI.

| Parámetro | Valor por Defecto | Descripción |
| :--- | :--- | :--- |
| `log_level` | `"info"` | Nivel de registro para la salida de `tracing`. |
| `base_url` | `"api.openai.com"` | URL base para la API. |
| `endpoint` | `"v1/chat/completions"` | Endpoint para las consultas. |
| `token` | `""` | **Tu clave API de OpenAI.** (Obligatorio) |
| `model` | `"gpt-3.5-turbo"` | El modelo de IA a utilizar. |
| `temperature` | `"1"` | Temperatura de muestreo (controla la creatividad/aleatoriedad). |

> **IMPORTANTE:** Si el campo `token` en la configuración está vacío, el programa terminará con un error.

## 💻 Uso

La aplicación utiliza dos argumentos principales para interactuar con el modelo de IA: las instrucciones para definir el rol de la IA y la pregunta en sí.

### Sintaxis

```bash
dime -i <INSTRUCCIONES> -q <PREGUNTA>
```

### Argumentos

| Argumento | Forma Corta | Descripción |
| :--- | :--- | :--- |
| `--instructions` | `-i` | Instrucciones de comportamiento para el modelo (rol del sistema). |
| `--question` | `-q` | La pregunta o solicitud que deseas realizar. |

### Ejemplo

Define el rol de la IA como un experto en `bash` y pídele un comando:

```bash
dime -i "Eres un experto en shell script y solo respondes con el comando bash que se solicita, sin explicaciones." -q "Comando para buscar archivos .log modificados en los últimos 7 días y comprimirlos en un tar.gz"
```

El modelo de IA procesará la solicitud con el rol definido y devolverá la respuesta directamente a la terminal.

## 🛠️ Tecnologías

**dime** está construido íntegramente en Rust, aprovechando las siguientes librerías:

  * **tokio:** Para la ejecución asíncrona.
  * **hyper** y **hyper-rustls:** Para realizar las peticiones HTTP seguras a la API.
  * **clap:** Para el análisis y gestión de los argumentos de línea de comandos.
  * **serde** y **serde\_yaml/serde\_json:** Para la serialización y deserialización de la configuración y las respuestas de la API.
  * **tracing:** Para la gestión de logs y depuración.
  * **dirs:** Para localizar rutas de configuración y *home* en distintos sistemas operativos.
