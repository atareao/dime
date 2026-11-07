use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use std::path::{Path, PathBuf};
use tokio::fs::read_to_string;
use tracing::{debug, error, info};

/// Errores tipados del cliente `IABot`.
///
/// Sustituye al antiguo `Box<dyn Error>` y a los múltiples `unwrap()` que
/// provocaban *panics* ante respuestas o ficheros inesperados.
#[derive(Debug, thiserror::Error)]
pub enum IABotError {
    /// El fichero de configuración no se pudo leer o no es YAML válido.
    #[error("Error con el fichero de configuración `dime.yml`: {0}")]
    Config(String),
    /// Fallo de red al contactar con el proveedor.
    #[error("Error de red al contactar con el proveedor: {0}")]
    Http(String),
    /// El proveedor devolvió un campo `error`.
    #[error("Error devuelto por el proveedor: {0}")]
    Api(String),
    /// La respuesta no tiene ni `choices` ni `error`.
    #[error("Respuesta inesperada del proveedor: no contiene `choices` ni `error`")]
    UnexpectedResponse,
    /// La entrada del usuario no es utilizable: falta la pregunta o `stdin` no
    /// aporta datos cuando se esperaban.
    #[error("Entrada no válida: {0}")]
    Input(String),
    /// El proveedor indicado por CLI (`--provider`) no es válido.
    ///
    /// Es un error del flag, no del fichero, por eso tiene un mensaje propio y
    /// no se usa [`IABotError::Config`].
    #[error("proveedor no soportado: `{0}`; valores válidos: `openai`, `ollama`")]
    Provider(String),
    /// Un adjunto (`--file` o `--image`) no se pudo leer o no es válido.
    ///
    /// Cubre ficheros inexistentes o no UTF-8, imágenes ilegibles y extensiones
    /// de imagen no soportadas.
    #[error("Error con el adjunto: {0}")]
    Attachment(String),
    /// No se pudo escribir en el portapapeles (`--copy`).
    ///
    /// Es *best-effort*: el llamante advierte y continúa con código de salida
    /// cero, por lo que nunca interrumpe el comando.
    #[error("No se pudo copiar al portapapeles: {0}")]
    Clipboard(String),
}

/// Compone el mensaje de usuario concatenando la pregunta con un contexto
/// opcional (por ejemplo, el contenido leído de `stdin`).
///
/// Si `context` es `None` o sólo contiene espacios en blanco, devuelve la
/// pregunta tal cual. En caso contrario, usa la pregunta seguida de una línea
/// en blanco y el contexto recortado por la derecha.
pub fn compose_user_message(question: &str, context: Option<&str>) -> String {
    match context.map(str::trim_end) {
        Some(context) if !context.is_empty() => format!("{question}\n\n{context}"),
        _ => question.to_string(),
    }
}

/// Un adjunto de imagen listo para enviar al proveedor.
///
/// `data` es la imagen ya codificada en base64 y `mime` su tipo MIME deducido
/// de la extensión (`image/png`, `image/jpeg`, `image/gif` o `image/webp`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageAttachment {
    /// Tipo MIME de la imagen (p. ej. `image/png`).
    pub mime: String,
    /// Contenido de la imagen codificado en base64 (sin prefijo *data URL*).
    pub data: String,
}

/// Compone el bloque de contexto a partir de una lista de `(ruta, contenido)`.
///
/// Cada fichero se precede de una cabecera `## <ruta>` seguida del contenido
/// recortado por la derecha. Los ficheros se separan con una línea en blanco y
/// se respeta el orden recibido. Sin ficheros devuelve una cadena vacía.
pub fn compose_context(files: &[(String, String)]) -> String {
    files
        .iter()
        .map(|(path, content)| format!("## {path}\n\n{}", content.trim_end()))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Lee cada ruta como texto UTF-8 y devuelve pares `(ruta, contenido)` en orden.
///
/// Un fichero inexistente o no UTF-8 produce [`IABotError::Attachment`] con un
/// mensaje legible, sin *panic*.
pub async fn read_files(paths: &[PathBuf]) -> Result<Vec<(String, String)>, IABotError> {
    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        let content = read_to_string(path).await.map_err(|e| {
            IABotError::Attachment(format!(
                "no se pudo leer el fichero `{}` como texto UTF-8: {e}",
                path.display()
            ))
        })?;
        files.push((path.display().to_string(), content));
    }
    Ok(files)
}

/// Deducir el MIME de una imagen a partir de su extensión.
///
/// Reconoce `png`, `jpg`/`jpeg`, `gif` y `webp` (insensible a mayúsculas).
/// Cualquier otra extensión, o la ausencia de ella, produce
/// [`IABotError::Attachment`].
pub fn mime_from_path(path: &Path) -> Result<&'static str, IABotError> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match extension.as_str() {
        "png" => Ok("image/png"),
        "jpg" | "jpeg" => Ok("image/jpeg"),
        "gif" => Ok("image/gif"),
        "webp" => Ok("image/webp"),
        _ => Err(IABotError::Attachment(format!(
            "extensión de imagen no soportada en `{}`; usa png, jpg, jpeg, gif o webp",
            path.display()
        ))),
    }
}

/// Lee cada imagen, la codifica en base64 y deduce su MIME por extensión.
///
/// Una imagen inexistente, ilegible o con extensión no soportada produce
/// [`IABotError::Attachment`].
pub async fn read_images(paths: &[PathBuf]) -> Result<Vec<ImageAttachment>, IABotError> {
    use base64::Engine;

    let mut images = Vec::with_capacity(paths.len());
    for path in paths {
        let mime = mime_from_path(path)?;
        let bytes = tokio::fs::read(path).await.map_err(|e| {
            IABotError::Attachment(format!(
                "no se pudo leer la imagen `{}`: {e}",
                path.display()
            ))
        })?;
        let data = base64::engine::general_purpose::STANDARD.encode(bytes);
        images.push(ImageAttachment {
            mime: mime.to_string(),
            data,
        });
    }
    Ok(images)
}

/// Cliente de chat para proveedores compatibles con la API de OpenAI.
///
/// La implementación de [`fmt::Debug`] es manual para redactar el token y
/// evitar que aparezca en cualquier traza de log.
#[derive(Serialize, Deserialize)]
pub struct IABot {
    #[serde(default = "get_default_log_level")]
    log_level: String,
    #[serde(default = "get_default_system_role")]
    system_role: String,
    #[serde(default = "get_default_provider")]
    provider: String,
    #[serde(default = "get_default_base_url")]
    base_url: String,
    #[serde(default = "get_default_endpoint")]
    endpoint: String,
    #[serde(default = "get_default_models_endpoint")]
    models_endpoint: String,
    #[serde(default = "get_default_token")]
    token: String,
    #[serde(default = "get_default_model")]
    model: String,
    // What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic.
    #[serde(
        default = "get_default_temperature",
        deserialize_with = "deserialize_temperature"
    )]
    temperature: f64,
    #[serde(default = "get_default_num_ctx")]
    num_ctx: u32,
    #[serde(default = "get_default_use_https")]
    use_https: bool,
}

impl fmt::Debug for IABot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let token = if self.token.is_empty() { "" } else { "***" };
        f.debug_struct("IABot")
            .field("log_level", &self.log_level)
            .field("system_role", &self.system_role)
            .field("provider", &self.provider)
            .field("base_url", &self.base_url)
            .field("endpoint", &self.endpoint)
            .field("models_endpoint", &self.models_endpoint)
            .field("token", &token)
            .field("model", &self.model)
            .field("temperature", &self.temperature)
            .field("num_ctx", &self.num_ctx)
            .field("use_https", &self.use_https)
            .finish()
    }
}

fn get_default_log_level() -> String {
    "info".to_string()
}

fn get_default_system_role() -> String {
    "Eres un asistente util y conciso. Responde en espanol claro.".to_string()
}

fn get_default_provider() -> String {
    "openai".to_string() // "openai" o "ollama"
}

fn get_default_base_url() -> String {
    "api.openai.com".to_string()
}

fn get_default_use_https() -> bool {
    true
}

fn get_default_endpoint() -> String {
    "v1/chat/completions".to_string()
}

fn get_default_models_endpoint() -> String {
    "v1/models".to_string()
}

fn get_default_token() -> String {
    "".to_string()
}

fn get_default_model() -> String {
    "gpt-3.5-turbo".to_string()
}

fn get_default_temperature() -> f64 {
    1.0
}

/// Deserializa `temperature` aceptando tanto un número YAML (`0.2`) como una
/// cadena numérica (`"0.2"`).
///
/// Un valor no numérico produce un error de deserialización legible, que
/// [`IABot::read_content`] convierte en [`IABotError::Config`] al cargar.
fn deserialize_temperature<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_norway::Value::deserialize(deserializer)?;

    match value {
        serde_norway::Value::Number(number) => number.as_f64().ok_or_else(|| {
            serde::de::Error::custom("el valor de `temperature` no es un número válido")
        }),
        serde_norway::Value::String(text) => text.trim().parse::<f64>().map_err(|_| {
            serde::de::Error::custom(format!(
                "el valor de `temperature` no es un número válido: `{text}`"
            ))
        }),
        other => Err(serde::de::Error::custom(format!(
            "el valor de `temperature` debe ser un número o una cadena numérica, no: {other:?}"
        ))),
    }
}

fn get_default_num_ctx() -> u32 {
    32000
}

impl Default for IABot {
    fn default() -> Self {
        Self {
            log_level: get_default_log_level(),
            system_role: get_default_system_role(),
            provider: get_default_provider(),
            base_url: get_default_base_url(),
            endpoint: get_default_endpoint(),
            models_endpoint: get_default_models_endpoint(),
            token: get_default_token(),
            model: get_default_model(),
            temperature: get_default_temperature(),
            num_ctx: get_default_num_ctx(),
            use_https: get_default_use_https(),
        }
    }
}

impl IABot {
    pub fn get_log_level(&self) -> &str {
        &self.log_level
    }

    pub fn get_system_role(&self) -> &str {
        &self.system_role
    }

    /// Devuelve el modelo configurado (usado como valor por defecto cuando el
    /// proveedor no informa el modelo en su respuesta).
    pub fn get_model(&self) -> &str {
        &self.model
    }

    /// Indica si el proveedor configurado es Ollama (comparación insensible a mayúsculas).
    fn is_ollama(&self) -> bool {
        self.provider.eq_ignore_ascii_case("ollama")
    }

    /// Valida la configuración antes de usarla.
    ///
    /// Falla cuando el proveedor no es Ollama y el token está vacío.
    pub fn validate(&self) -> Result<(), IABotError> {
        if !self.is_ollama() && self.token.is_empty() {
            return Err(IABotError::Config(
                "el token está vacío para un proveedor que lo requiere".to_string(),
            ));
        }
        Ok(())
    }

    /// Aplica los overrides de una sola invocación sobre la configuración
    /// efectiva (`model`, `temperature` y `provider`).
    ///
    /// Sólo muta los campos cuyo override está presente; el resto conserva el
    /// valor de `dime.yml`. **Nunca** escribe ni modifica el fichero de
    /// configuración: los overrides viven únicamente en memoria.
    ///
    /// Un `provider` distinto de `openai` u `ollama` (comparación insensible a
    /// mayúsculas) devuelve [`IABotError::Config`] sin alterar la configuración.
    ///
    /// Este método se aplica tras `read_content` y **antes** de
    /// [`IABot::validate`], de modo que un override a `ollama` exime de token.
    pub fn apply_overrides(
        &mut self,
        model: Option<String>,
        temperature: Option<f64>,
        provider: Option<String>,
    ) -> Result<(), IABotError> {
        if let Some(provider) = &provider {
            if !provider.eq_ignore_ascii_case("openai") && !provider.eq_ignore_ascii_case("ollama")
            {
                return Err(IABotError::Provider(provider.to_string()));
            }
        }

        if let Some(model) = model {
            self.model = model;
        }
        if let Some(temperature) = temperature {
            self.temperature = temperature;
        }
        if let Some(provider) = provider {
            self.provider = provider.to_ascii_lowercase();
        }

        Ok(())
    }

    /// Devuelve la cabecera `Authorization: Bearer <token>` cuando procede.
    ///
    /// Sólo se envía para el proveedor `openai` y con un token no vacío.
    fn auth_header(&self) -> Option<String> {
        if self.provider.eq_ignore_ascii_case("openai") && !self.token.is_empty() {
            Some(format!("Bearer {}", self.token))
        } else {
            None
        }
    }

    /// Construye la URL de la petición de chat: `{protocolo}://{base_url}/{endpoint}`.
    fn build_url(&self) -> String {
        self.build_url_for(&self.endpoint)
    }

    /// Construye la URL de listado de modelos: `{protocolo}://{base_url}/{models_endpoint}`.
    fn build_models_url(&self) -> String {
        self.build_url_for(&self.models_endpoint)
    }

    /// Compone `{protocolo}://{base_url}/{path}` respetando `use_https`.
    fn build_url_for(&self, path: &str) -> String {
        let protocol = if self.use_https { "https" } else { "http" };
        format!("{}://{}/{}", protocol, self.base_url, path)
    }

    /// Construye el cuerpo JSON de la petición sin tocar la red.
    ///
    /// - `openai`: `temperature` en el nivel superior. El `content` del mensaje
    ///   `user` es una cadena cuando no hay imágenes y un array de content parts
    ///   (texto + `image_url` como *data URL* base64) cuando las hay.
    /// - `ollama`: `temperature` y `num_ctx` dentro de `options`; las imágenes
    ///   van en el campo `images` del mensaje `user` y `content` sigue siendo
    ///   una cadena de texto.
    fn build_body(
        &self,
        system: &str,
        question: &str,
        images: &[ImageAttachment],
    ) -> Result<String, IABotError> {
        let temperature = self.temperature;

        let user_message = if self.is_ollama() {
            let mut message = json!({"role": "user", "content": question});
            if !images.is_empty() {
                if let Some(object) = message.as_object_mut() {
                    let data: Vec<&str> = images.iter().map(|image| image.data.as_str()).collect();
                    object.insert("images".to_string(), json!(data));
                }
            }
            message
        } else {
            json!({"role": "user", "content": self.openai_content(question, images)})
        };

        let messages = json!([
            {"role": "system", "content": system},
            user_message
        ]);

        let payload = if self.is_ollama() {
            json!({
                "model": self.model,
                "messages": messages,
                "options": {
                    "num_ctx": self.num_ctx,
                    "temperature": temperature,
                }
            })
        } else {
            json!({
                "model": self.model,
                "messages": messages,
                "temperature": temperature,
            })
        };

        serde_json::to_string(&payload).map_err(|e| IABotError::Http(e.to_string()))
    }

    /// Compone el `content` del mensaje `user` para `openai`.
    ///
    /// Sin imágenes devuelve la cadena de texto (compatibilidad con peticiones y
    /// tests existentes). Con imágenes devuelve un array de content parts con la
    /// primera parte de texto y una parte `image_url` por cada imagen.
    fn openai_content(&self, question: &str, images: &[ImageAttachment]) -> serde_json::Value {
        if images.is_empty() {
            return json!(question);
        }

        let mut parts = vec![json!({"type": "text", "text": question})];
        parts.extend(images.iter().map(|image| {
            json!({
                "type": "image_url",
                "image_url": {
                    "url": format!("data:{};base64,{}", image.mime, image.data)
                }
            })
        }));
        json!(parts)
    }

    /// Envía la pregunta al proveedor y devuelve su respuesta.
    pub async fn ask(
        &self,
        system: &str,
        question: &str,
        images: &[ImageAttachment],
    ) -> Result<ChatResponse, IABotError> {
        let url = self.build_url();
        debug!("Url: {}", url);
        debug!("Provider: {}", self.provider);
        debug!("Question: {}", question);

        let body = self.build_body(system, question, images)?;

        let client = Client::new();
        let mut request_builder = client.post(&url).header("Content-Type", "application/json");

        // Agregar token solo si es OpenAI y el token no está vacío
        if let Some(header) = self.auth_header() {
            request_builder = request_builder.header("Authorization", header);
        }

        let response = request_builder.body(body).send().await.map_err(|e| {
            error!("{}", e);
            IABotError::Http(e.to_string())
        })?;

        let response_body = response
            .text()
            .await
            .map_err(|e| IABotError::Http(e.to_string()))?;
        debug!("Response length: {}", response_body.len());

        parse_response(&response_body)
    }

    /// Consulta el listado de modelos disponibles del proveedor.
    ///
    /// Llama a `{protocolo}://{base_url}/{models_endpoint}` incluyendo la
    /// cabecera `Authorization` cuando corresponde y devuelve los
    /// identificadores de los modelos. Ante un error de red o de API devuelve
    /// un [`IABotError`] tipado, nunca un *panic*.
    pub async fn list_models(&self) -> Result<Vec<String>, IABotError> {
        let url = self.build_models_url();
        debug!("Models url: {}", url);

        let client = Client::new();
        let mut request_builder = client.get(&url);

        if let Some(header) = self.auth_header() {
            request_builder = request_builder.header("Authorization", header);
        }

        let response = request_builder.send().await.map_err(|e| {
            error!("{}", e);
            IABotError::Http(e.to_string())
        })?;

        let response_body = response
            .text()
            .await
            .map_err(|e| IABotError::Http(e.to_string()))?;

        parse_models_response(&response_body)
    }

    /// Escribe en `file` una configuración con los valores por defecto.
    pub async fn write_default(file: &PathBuf) -> Result<(), IABotError> {
        let default = Self::default();
        let content =
            serde_norway::to_string(&default).map_err(|e| IABotError::Config(e.to_string()))?;
        tokio::fs::write(file, content.as_bytes())
            .await
            .map_err(|e| IABotError::Config(e.to_string()))
    }

    /// Lee y deserializa un fichero `dime.yml`.
    ///
    /// Devuelve un [`IABotError::Config`] legible (mencionando `dime.yml`) en
    /// lugar de terminar el proceso.
    pub async fn read_content(file: &PathBuf) -> Result<IABot, IABotError> {
        info!("File to read: {:?}", file);
        let content = read_to_string(file)
            .await
            .map_err(|e| IABotError::Config(e.to_string()))?;
        serde_norway::from_str(&content).map_err(|e| IABotError::Config(e.to_string()))
    }
}

/// Respuesta de chat con metadatos, lista para serializar.
///
/// `content` es la respuesta del modelo; `model` es el modelo efectivamente
/// usado y `usage` es el objeto de uso tal cual lo devuelve el proveedor
/// (opaco). Cuando el proveedor no aporta `model` o `usage`, el campo es
/// `None` y se serializa como `null`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Contenido textual de la respuesta.
    pub content: String,
    /// Modelo usado por el proveedor, si lo informa.
    pub model: Option<String>,
    /// Objeto de uso crudo del proveedor, si lo informa.
    pub usage: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ApiChatResponse {
    choices: Vec<Choice>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    usage: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    error: serde_json::Value,
}

/// Interpreta el cuerpo de una respuesta de chat.
///
/// Primero intenta el error del proveedor y después la respuesta estándar,
/// devolviendo el contenido junto con el modelo y el objeto de uso cuando están
/// presentes. El campo `error` puede ser un objeto con `message` o una cadena.
/// Cualquier otra forma devuelve [`IABotError::UnexpectedResponse`], nunca un *panic*.
fn parse_response(body: &str) -> Result<ChatResponse, IABotError> {
    if let Some(err) = extract_api_error(body) {
        return Err(err);
    }

    if let Ok(response) = serde_json::from_str::<ApiChatResponse>(body) {
        let ApiChatResponse {
            choices,
            model,
            usage,
        } = response;
        if let Some(choice) = choices.into_iter().next() {
            return Ok(ChatResponse {
                content: choice.message.content,
                model,
                usage,
            });
        }
    }

    Err(IABotError::UnexpectedResponse)
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    id: String,
}

#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModelEntry>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelEntry {
    name: String,
}

/// Interpreta el cuerpo de la respuesta de listado de modelos.
///
/// Primero busca el error del proveedor y después acepta dos formas:
/// `data[*].id`, usada por OpenAI y proveedores compatibles (`v1/models`), y
/// `models[*].name`, usada por Ollama nativo (`api/tags`). Cualquier otra forma
/// devuelve [`IABotError::UnexpectedResponse`], nunca un *panic*.
fn parse_models_response(body: &str) -> Result<Vec<String>, IABotError> {
    if let Some(err) = extract_api_error(body) {
        return Err(err);
    }

    if let Ok(response) = serde_json::from_str::<ModelsResponse>(body) {
        return Ok(response.data.into_iter().map(|entry| entry.id).collect());
    }

    if let Ok(response) = serde_json::from_str::<OllamaModelsResponse>(body) {
        return Ok(response
            .models
            .into_iter()
            .map(|entry| entry.name)
            .collect());
    }

    Err(IABotError::UnexpectedResponse)
}

/// Extrae el error devuelto por el proveedor, si el cuerpo lo contiene.
///
/// Devuelve `None` cuando no hay campo `error` o éste es `null`.
fn extract_api_error(body: &str) -> Option<IABotError> {
    let api_error = serde_json::from_str::<ApiErrorBody>(body).ok()?;
    if api_error.error.is_null() {
        return None;
    }
    if let Some(message) = api_error.error.get("message").and_then(|m| m.as_str()) {
        return Some(IABotError::Api(message.to_string()));
    }
    if let Some(message) = api_error.error.as_str() {
        return Some(IABotError::Api(message.to_string()));
    }
    Some(IABotError::UnexpectedResponse)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Genera una ruta temporal única para los tests de carga de configuración.
    fn temp_yaml_path() -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut path = std::env::temp_dir();
        path.push(format!("dime-test-{}-{}.yml", std::process::id(), n));
        path
    }

    /// Carga una configuración desde una cadena YAML usando [`IABot::read_content`],
    /// exactamente igual que en producción.
    async fn load_config(yaml: &str) -> Result<IABot, IABotError> {
        let path = temp_yaml_path();
        std::fs::write(&path, yaml).expect("write temp yaml");
        let result = IABot::read_content(&path).await;
        let _ = std::fs::remove_file(&path);
        result
    }

    fn bot() -> IABot {
        IABot::default()
    }

    fn openai_bot() -> IABot {
        IABot {
            provider: "openai".to_string(),
            token: "secreto-super-secreto".to_string(),
            ..IABot::default()
        }
    }

    fn ollama_bot() -> IABot {
        IABot {
            provider: "ollama".to_string(),
            ..IABot::default()
        }
    }

    // --- chat spec: Construcción de la URL ---

    #[test]
    fn build_url_uses_https_by_default() {
        assert_eq!(
            bot().build_url(),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn build_url_uses_http_when_use_https_false() {
        let mut b = bot();
        b.use_https = false;
        assert_eq!(b.build_url(), "http://api.openai.com/v1/chat/completions");
    }

    // --- chat spec: Petición para OpenAI ---

    #[test]
    fn build_body_openai_has_messages_and_temperature_at_top_level() {
        let body = openai_bot().build_body("sys", "hola", &[]).expect("body");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["model"], "gpt-3.5-turbo");
        assert_eq!(v["messages"][0]["role"], "system");
        assert_eq!(v["messages"][0]["content"], "sys");
        assert_eq!(v["messages"][1]["role"], "user");
        assert_eq!(v["messages"][1]["content"], "hola");
        assert_eq!(v["temperature"], serde_json::json!(1.0));
        assert!(v.get("options").is_none());
    }

    // --- chat spec: Petición para Ollama ---

    #[test]
    fn build_body_ollama_puts_num_ctx_and_temperature_in_options() {
        let body = ollama_bot().build_body("sys", "hola", &[]).expect("body");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["options"]["num_ctx"], 32000);
        assert_eq!(v["options"]["temperature"], serde_json::json!(1.0));
        assert!(v.get("temperature").is_none());
    }

    // --- chat spec: Temperatura ---

    #[test]
    fn build_body_uses_configured_temperature() {
        let mut b = bot();
        b.temperature = 0.2;
        let body = b.build_body("sys", "q", &[]).expect("body");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["temperature"], serde_json::json!(0.2));
    }

    // --- attachments/chat spec: Imágenes multimodales ---

    fn image(mime: &str, data: &str) -> ImageAttachment {
        ImageAttachment {
            mime: mime.to_string(),
            data: data.to_string(),
        }
    }

    #[test]
    fn build_body_openai_with_image_uses_content_parts() {
        let images = vec![image("image/png", "UE5HREFUQQ==")];
        let body = openai_bot()
            .build_body("sys", "describe", &images)
            .expect("body");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        let content = &v["messages"][1]["content"];
        assert!(content.is_array(), "content debe ser array: {content}");
        assert_eq!(content[0]["type"], "text");
        assert_eq!(content[0]["text"], "describe");
        assert_eq!(content[1]["type"], "image_url");
        let url = content[1]["image_url"]["url"].as_str().expect("url");
        assert!(url.starts_with("data:image/png;base64,"), "url: {url}");
    }

    #[test]
    fn build_body_ollama_with_image_uses_images_field_and_string_content() {
        let images = vec![image("image/png", "UE5HREFUQQ==")];
        let body = ollama_bot()
            .build_body("sys", "describe", &images)
            .expect("body");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        let user = &v["messages"][1];
        assert!(user["content"].is_string(), "content: {}", user["content"]);
        assert_eq!(user["content"], "describe");
        assert_eq!(user["images"][0], "UE5HREFUQQ==");
    }

    #[test]
    fn build_body_openai_without_images_keeps_content_as_string() {
        let body = openai_bot().build_body("sys", "hola", &[]).expect("body");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(v["messages"][1]["content"].is_string());
        assert_eq!(v["messages"][1]["content"], "hola");
    }

    #[test]
    fn build_body_ollama_without_images_omits_images_field() {
        let body = ollama_bot().build_body("sys", "hola", &[]).expect("body");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(v["messages"][1].get("images").is_none());
    }

    // --- chat spec: Tratamiento de respuestas ---

    #[test]
    fn parse_response_ok_returns_content() {
        let body = r#"{"choices":[{"message":{"role":"assistant","content":"Hola"}}]}"#;
        assert_eq!(parse_response(body).unwrap().content, "Hola");
    }

    // --- output/chat spec: respuesta con metadatos ---

    #[test]
    fn parse_response_exposes_content_model_and_usage() {
        let body = r#"{"model":"gpt-4o","usage":{"prompt_tokens":5,"completion_tokens":2,"total_tokens":7},"choices":[{"message":{"role":"assistant","content":"Hola"}}]}"#;
        let response = parse_response(body).expect("respuesta");
        assert_eq!(response.content, "Hola");
        assert_eq!(response.model.as_deref(), Some("gpt-4o"));
        let usage = response.usage.expect("usage");
        assert_eq!(usage["total_tokens"], 7);
    }

    #[test]
    fn parse_response_without_usage_has_none_usage() {
        let body = r#"{"choices":[{"message":{"content":"Hola"}}]}"#;
        let response = parse_response(body).expect("respuesta");
        assert_eq!(response.content, "Hola");
        assert!(response.model.is_none());
        assert!(response.usage.is_none());
    }

    #[test]
    fn parse_response_provider_error_returns_message() {
        let body = r#"{"error":{"message":"invalid_api_key"}}"#;
        let err = parse_response(body).unwrap_err();
        assert!(err.to_string().contains("invalid_api_key"), "err: {err}");
        assert!(matches!(err, IABotError::Api(_)));
    }

    #[test]
    fn parse_response_unexpected_shape_returns_typed_error() {
        let body = r#"{"foo":"bar"}"#;
        let err = parse_response(body).unwrap_err();
        assert!(matches!(err, IABotError::UnexpectedResponse));
    }

    #[test]
    fn parse_response_empty_choices_returns_typed_error() {
        let body = r#"{"choices":[]}"#;
        let err = parse_response(body).unwrap_err();
        assert!(matches!(err, IABotError::UnexpectedResponse));
    }

    // --- config spec: Valores por defecto ---

    #[tokio::test]
    async fn yaml_partial_uses_defaults() {
        let cfg = load_config("model: mi-modelo").await.expect("cfg");
        assert_eq!(cfg.model, "mi-modelo");
        assert_eq!(cfg.provider, "openai");
        assert_eq!(cfg.base_url, "api.openai.com");
        assert_eq!(cfg.endpoint, "v1/chat/completions");
        assert_eq!(cfg.models_endpoint, "v1/models");
        assert_eq!(cfg.temperature, 1.0);
        assert_eq!(cfg.num_ctx, 32000);
        assert!(cfg.use_https);
        assert_eq!(cfg.log_level, "info");
        assert_eq!(cfg.token, "");
    }

    // --- config spec: temperature numérica tolerante ---

    #[tokio::test]
    async fn temperature_number_is_parsed() {
        let cfg = load_config("temperature: 0.2").await.expect("cfg");
        assert_eq!(cfg.temperature, 0.2);
    }

    #[tokio::test]
    async fn temperature_numeric_string_is_parsed() {
        let cfg = load_config("temperature: \"0.2\"").await.expect("cfg");
        assert_eq!(cfg.temperature, 0.2);
    }

    #[tokio::test]
    async fn temperature_default_is_one() {
        let cfg = load_config("model: mi-modelo").await.expect("cfg");
        assert_eq!(cfg.temperature, 1.0);
    }

    #[tokio::test]
    async fn temperature_non_numeric_is_config_error() {
        let err = load_config("temperature: \"alta\"").await.unwrap_err();
        assert!(matches!(err, IABotError::Config(_)), "err: {err}");
        assert!(err.to_string().contains("temperature"), "err: {err}");
    }

    #[tokio::test]
    async fn temperature_bool_is_config_error_mentioning_field() {
        let err = load_config("temperature: true").await.unwrap_err();
        assert!(matches!(err, IABotError::Config(_)), "err: {err}");
        assert!(err.to_string().contains("temperature"), "err: {err}");
    }

    #[tokio::test]
    async fn temperature_list_is_config_error_mentioning_field() {
        let err = load_config("temperature: [1, 2]").await.unwrap_err();
        assert!(matches!(err, IABotError::Config(_)), "err: {err}");
        assert!(err.to_string().contains("temperature"), "err: {err}");
    }

    // --- config spec: Endpoint de modelos configurable ---

    #[tokio::test]
    async fn models_endpoint_default_is_v1_models() {
        let cfg = load_config("model: mi-modelo").await.expect("cfg");
        assert_eq!(cfg.models_endpoint, "v1/models");
    }

    #[tokio::test]
    async fn models_endpoint_custom_value_is_kept() {
        let cfg = load_config("models_endpoint: api/tags").await.expect("cfg");
        assert_eq!(cfg.models_endpoint, "api/tags");
    }

    // --- models spec: URL de listado ---

    #[test]
    fn models_url_uses_default_endpoint() {
        assert_eq!(bot().build_models_url(), "https://api.openai.com/v1/models");
    }

    #[test]
    fn models_url_respects_use_https_false() {
        let mut b = bot();
        b.use_https = false;
        b.base_url = "localhost:11434".to_string();
        b.models_endpoint = "api/tags".to_string();
        assert_eq!(b.build_models_url(), "http://localhost:11434/api/tags");
    }

    // --- models spec: Parseo de la respuesta de listado ---

    #[test]
    fn parse_models_response_returns_ids() {
        let body = r#"{"data":[{"id":"gpt-3.5-turbo"},{"id":"gpt-4o"}]}"#;
        assert_eq!(
            parse_models_response(body).unwrap(),
            vec!["gpt-3.5-turbo", "gpt-4o"]
        );
    }

    #[test]
    fn parse_models_response_ollama_returns_names() {
        let body = r#"{"models":[{"name":"llama2"},{"name":"mistral"}]}"#;
        assert_eq!(
            parse_models_response(body).unwrap(),
            vec!["llama2", "mistral"]
        );
    }

    #[test]
    fn parse_models_response_ollama_api_error_returns_api_error() {
        let body = r#"{"error":{"message":"model not found"}}"#;
        let err = parse_models_response(body).unwrap_err();
        assert!(matches!(err, IABotError::Api(_)), "err: {err}");
    }

    #[test]
    fn parse_models_response_api_error_returns_api_error() {
        let body = r#"{"error":{"message":"invalid_api_key"}}"#;
        let err = parse_models_response(body).unwrap_err();
        assert!(matches!(err, IABotError::Api(_)), "err: {err}");
        assert!(err.to_string().contains("invalid_api_key"), "err: {err}");
    }

    #[test]
    fn parse_models_response_unexpected_shape_returns_typed_error() {
        let err = parse_models_response(r#"{"foo":"bar"}"#).unwrap_err();
        assert!(matches!(err, IABotError::UnexpectedResponse));
    }

    // --- Bloque A: round-trip YAML con serde_norway ---

    #[test]
    fn yaml_round_trip_preserves_values() {
        let b = openai_bot();
        let yaml = serde_norway::to_string(&b).expect("serialize");
        let cfg: IABot = serde_norway::from_str(&yaml).expect("deserialize");
        assert_eq!(cfg.provider, b.provider);
        assert_eq!(cfg.base_url, b.base_url);
        assert_eq!(cfg.endpoint, b.endpoint);
        assert_eq!(cfg.models_endpoint, b.models_endpoint);
        assert_eq!(cfg.model, b.model);
        assert_eq!(cfg.temperature, b.temperature);
        assert_eq!(cfg.num_ctx, b.num_ctx);
        assert_eq!(cfg.use_https, b.use_https);
        assert_eq!(cfg.token, b.token);
        assert_eq!(cfg.log_level, b.log_level);
        assert_eq!(cfg.system_role, b.system_role);
    }

    // --- config spec: Errores legibles ---

    #[tokio::test]
    async fn read_content_invalid_yaml_mentions_dime_yml() {
        let mut path = std::env::temp_dir();
        path.push(format!("dime-invalid-{}.yml", std::process::id()));
        std::fs::write(&path, "provider: [unclosed").unwrap();
        let err = IABot::read_content(&path).await.unwrap_err();
        assert!(err.to_string().contains("dime.yml"), "msg: {err}");
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn read_content_missing_file_mentions_dime_yml() {
        let mut path = std::env::temp_dir();
        path.push(format!("dime-missing-{}.yml", std::process::id()));
        let err = IABot::read_content(&path).await.unwrap_err();
        assert!(err.to_string().contains("dime.yml"), "msg: {err}");
    }

    // --- config spec: El token no se registra ---

    #[test]
    fn debug_redacts_token() {
        let dbg = format!("{:?}", openai_bot());
        assert!(!dbg.contains("secreto-super-secreto"), "leak: {dbg}");
        assert!(dbg.contains("***"), "dbg: {dbg}");
    }

    #[test]
    fn debug_empty_token_is_not_shown_as_redacted() {
        let dbg = format!("{:?}", bot());
        assert!(!dbg.contains("***"), "dbg: {dbg}");
    }

    // --- MAYOR 1: cabecera Authorization ---

    #[test]
    fn auth_header_openai_with_token() {
        assert_eq!(
            openai_bot().auth_header(),
            Some("Bearer secreto-super-secreto".to_string())
        );
    }

    #[test]
    fn auth_header_openai_without_token() {
        let mut b = openai_bot();
        b.token = String::new();
        assert_eq!(b.auth_header(), None);
    }

    #[test]
    fn auth_header_ollama_is_none_even_with_token() {
        let mut b = ollama_bot();
        b.token = "algo".to_string();
        assert_eq!(b.auth_header(), None);
    }

    // --- MAYOR 1: validación de token vacío ---

    #[test]
    fn validate_openai_with_empty_token_fails() {
        let mut b = openai_bot();
        b.token = String::new();
        assert!(b.validate().is_err());
    }

    #[test]
    fn validate_ollama_with_empty_token_ok() {
        assert!(ollama_bot().validate().is_ok());
    }

    // --- MENOR 3: comparación de proveedor case-insensitive ---

    #[test]
    fn is_ollama_is_case_insensitive() {
        let mut b = bot();
        b.provider = "Ollama".to_string();
        assert!(b.is_ollama());
    }

    // --- MENOR 2: error de proveedor como cadena ---

    #[test]
    fn parse_response_provider_error_as_string() {
        let body = r#"{"error":"boom"}"#;
        let err = parse_response(body).unwrap_err();
        assert!(matches!(err, IABotError::Api(_)));
        assert!(err.to_string().contains("boom"));
    }

    // --- MENOR 1: `error: null` debe caer a `choices` ---

    #[test]
    fn parse_response_null_error_falls_back_to_choices() {
        let body = r#"{"error":null,"choices":[{"message":{"content":"Hola"}}]}"#;
        assert_eq!(parse_response(body).unwrap().content, "Hola");
    }

    // --- MENOR 2: no existe dime.yml -> se crea con defaults ---

    #[tokio::test]
    async fn write_default_creates_file_with_defaults() {
        let mut path = std::env::temp_dir();
        path.push(format!("dime-default-{}.yml", std::process::id()));
        let _ = std::fs::remove_file(&path);

        IABot::write_default(&path).await.expect("write_default");
        assert!(path.exists(), "el fichero por defecto debe existir");

        let cfg = IABot::read_content(&path).await.expect("read_content");
        assert_eq!(cfg.provider, "openai");
        assert_eq!(cfg.base_url, "api.openai.com");
        assert_eq!(cfg.model, "gpt-3.5-turbo");
        assert_eq!(cfg.num_ctx, 32000);
        assert!(cfg.use_https);

        let _ = std::fs::remove_file(&path);
    }

    // --- cli-overrides spec: overrides de una sola invocación ---

    #[test]
    fn apply_overrides_sets_model_temperature_and_provider() {
        let mut b = openai_bot();
        b.apply_overrides(
            Some("gpt-4o".to_string()),
            Some(0.2),
            Some("ollama".to_string()),
        )
        .expect("overrides válidos");
        assert_eq!(b.model, "gpt-4o");
        assert_eq!(b.temperature, 0.2);
        assert_eq!(b.provider, "ollama");
    }

    #[test]
    fn apply_overrides_without_values_keeps_configuration() {
        let mut b = openai_bot();
        b.apply_overrides(None, None, None).expect("sin overrides");
        assert_eq!(b.model, "gpt-3.5-turbo");
        assert_eq!(b.temperature, 1.0);
        assert_eq!(b.provider, "openai");
    }

    #[test]
    fn apply_overrides_temperature_only_keeps_model_and_provider() {
        let mut b = openai_bot();
        b.apply_overrides(None, Some(0.2), None)
            .expect("override de temperatura");
        let body = b.build_body("sys", "q", &[]).expect("body");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["temperature"], serde_json::json!(0.2));
        assert_eq!(v["model"], "gpt-3.5-turbo");
    }

    #[test]
    fn apply_overrides_provider_is_case_insensitive() {
        let mut b = openai_bot();
        b.apply_overrides(None, None, Some("OLLAMA".to_string()))
            .expect("proveedor válido");
        assert!(b.is_ollama());
        assert_eq!(b.provider, "ollama");
    }

    #[test]
    fn apply_overrides_unknown_provider_is_provider_error_and_keeps_config() {
        let mut b = openai_bot();
        let err = b
            .apply_overrides(None, None, Some("desconocido".to_string()))
            .expect_err("proveedor inválido");
        assert!(matches!(err, IABotError::Provider(_)), "err: {err}");
        assert!(err.to_string().contains("desconocido"), "err: {err}");
        assert!(
            !err.to_string().contains("dime.yml"),
            "el error no debe atribuirse al fichero: {err}"
        );
        assert_eq!(b.provider, "openai");
    }

    #[test]
    fn apply_overrides_to_ollama_drops_auth_and_does_not_require_token() {
        let mut b = openai_bot();
        b.token = String::new();
        b.apply_overrides(None, Some(0.2), Some("ollama".to_string()))
            .expect("override a ollama");
        // El override del proveedor se aplica ANTES de `validate()`: ollama no
        // exige token.
        assert!(b.validate().is_ok(), "ollama no debe exigir token");
        assert_eq!(b.auth_header(), None, "ollama no envía Authorization");

        let body = b.build_body("sys", "hola", &[]).expect("body");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["options"]["num_ctx"], 32000);
        assert_eq!(v["options"]["temperature"], serde_json::json!(0.2));
        assert!(v.get("temperature").is_none());
    }

    // --- attachments spec: Contexto desde ficheros ---

    fn temp_path(suffix: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut path = std::env::temp_dir();
        path.push(format!(
            "dime-attach-{}-{}-{}",
            std::process::id(),
            n,
            suffix
        ));
        path
    }

    #[test]
    fn compose_context_single_file_has_header_and_content() {
        let files = vec![("README.md".to_string(), "hola mundo".to_string())];
        let context = compose_context(&files);
        assert!(context.contains("## README.md"), "contexto: {context}");
        assert!(context.contains("hola mundo"), "contexto: {context}");
    }

    #[test]
    fn compose_context_preserves_order() {
        let files = vec![
            ("a.txt".to_string(), "contenido A".to_string()),
            ("b.txt".to_string(), "contenido B".to_string()),
        ];
        let context = compose_context(&files);
        let pos_a = context.find("a.txt").expect("a.txt presente");
        let pos_b = context.find("b.txt").expect("b.txt presente");
        assert!(pos_a < pos_b, "a.txt debe ir antes que b.txt: {context}");
    }

    #[test]
    fn compose_context_empty_is_empty() {
        assert_eq!(compose_context(&[]), "");
    }

    #[tokio::test]
    async fn read_files_reads_contents_in_order() {
        let a = temp_path("a.txt");
        let b = temp_path("b.txt");
        std::fs::write(&a, "contenido A").expect("write a");
        std::fs::write(&b, "contenido B").expect("write b");

        let files = read_files(&[a.clone(), b.clone()])
            .await
            .expect("lectura correcta");

        assert_eq!(files.len(), 2);
        assert!(files[0].0.ends_with("a.txt"), "ruta 0: {}", files[0].0);
        assert_eq!(files[0].1, "contenido A");
        assert_eq!(files[1].1, "contenido B");

        let _ = std::fs::remove_file(&a);
        let _ = std::fs::remove_file(&b);
    }

    #[tokio::test]
    async fn read_files_missing_file_is_attachment_error() {
        let missing = temp_path("no-existe.txt");
        let err = read_files(&[missing]).await.unwrap_err();
        assert!(matches!(err, IABotError::Attachment(_)), "err: {err}");
    }

    #[tokio::test]
    async fn read_files_non_utf8_is_attachment_error() {
        let path = temp_path("bin.dat");
        std::fs::write(&path, [0xff, 0xfe, 0x00]).expect("write bin");
        let err = read_files(std::slice::from_ref(&path)).await.unwrap_err();
        assert!(matches!(err, IABotError::Attachment(_)), "err: {err}");
        let _ = std::fs::remove_file(&path);
    }

    // --- attachments spec: Imágenes multimodales ---

    #[tokio::test]
    async fn read_images_encodes_base64_and_mime() {
        let path = temp_path("foto.png");
        std::fs::write(&path, b"PNGDATA").expect("write png");

        let images = read_images(std::slice::from_ref(&path))
            .await
            .expect("lectura imagen");
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].mime, "image/png");
        assert_eq!(images[0].data, "UE5HREFUQQ==");

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn read_images_missing_file_is_attachment_error() {
        let missing = temp_path("no-existe.png");
        let err = read_images(&[missing]).await.unwrap_err();
        assert!(matches!(err, IABotError::Attachment(_)), "err: {err}");
    }

    #[tokio::test]
    async fn read_images_unknown_extension_is_attachment_error() {
        let path = temp_path("documento.pdf");
        std::fs::write(&path, b"%PDF").expect("write pdf");
        let err = read_images(std::slice::from_ref(&path)).await.unwrap_err();
        assert!(matches!(err, IABotError::Attachment(_)), "err: {err}");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn mime_from_path_recognizes_supported_extensions() {
        assert_eq!(
            mime_from_path(&PathBuf::from("a.png")).unwrap(),
            "image/png"
        );
        assert_eq!(
            mime_from_path(&PathBuf::from("a.jpg")).unwrap(),
            "image/jpeg"
        );
        assert_eq!(
            mime_from_path(&PathBuf::from("a.JPEG")).unwrap(),
            "image/jpeg"
        );
        assert_eq!(
            mime_from_path(&PathBuf::from("a.gif")).unwrap(),
            "image/gif"
        );
        assert_eq!(
            mime_from_path(&PathBuf::from("a.webp")).unwrap(),
            "image/webp"
        );
    }

    #[test]
    fn mime_from_path_unknown_extension_is_error() {
        let err = mime_from_path(&PathBuf::from("a.bmp")).unwrap_err();
        assert!(matches!(err, IABotError::Attachment(_)), "err: {err}");
    }

    #[tokio::test]
    async fn apply_overrides_does_not_rewrite_config_file() {
        let path = temp_yaml_path();
        let original = "model: gpt-3.5-turbo\ntemperature: 1.0\nprovider: openai\n";
        std::fs::write(&path, original).expect("write temp yaml");

        let mut bot = IABot::read_content(&path).await.expect("read_content");
        bot.apply_overrides(
            Some("gpt-4o".to_string()),
            Some(0.2),
            Some("ollama".to_string()),
        )
        .expect("overrides válidos");

        let after = std::fs::read_to_string(&path).expect("read back");
        assert_eq!(after, original, "el fichero no debe reescribirse");

        let _ = std::fs::remove_file(&path);
    }
}
