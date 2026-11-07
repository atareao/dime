//! Tests de integración de la ruta real de red (`ask` y `list_models`).
//!
//! Cada test arranca un servidor `wiremock` en un puerto efímero, escribe un
//! `dime.yml` temporal que apunta a él y ejecuta el binario `dime` con
//! `std::process::Command`. Así se cubre la ruta completa (clap → configuración →
//! `reqwest` → render) sin exponer la API interna de `IABot` y sin tocar la red
//! externa.
//!
//! `wiremock` es una dependencia **de desarrollo**: no forma parte del binario.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::Value;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Ruta de chat usada en la configuración de los tests.
const CHAT_ENDPOINT: &str = "v1/chat/completions";
/// Ruta de listado de modelos usada en la configuración de los tests.
const MODELS_ENDPOINT: &str = "v1/models";
/// Token de pruebas: debe viajar en la cabecera `Authorization`.
const TOKEN: &str = "token-de-pruebas";
/// Rol de sistema configurado, para afirmar el contenido del mensaje `system`.
const SYSTEM_ROLE: &str = "Eres un asistente de pruebas.";
/// Respuesta de chat simulada, con el modelo que debe aparecer en `stdout`.
const CHAT_BODY: &str = r#"{"model":"gpt-4o-mini","choices":[{"message":{"role":"assistant","content":"respuesta simulada"}}]}"#;

/// Directorio temporal que se elimina automáticamente al terminar el test.
struct TempDir(PathBuf);

impl TempDir {
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Directorio temporal único por test dentro del `temp_dir`.
fn temp_dir(name: &str) -> TempDir {
    let dir = std::env::temp_dir().join(format!("dime-http-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("crear directorio temporal");
    TempDir(dir)
}

/// `host:puerto` del servidor mock, sin esquema (`dime` lo añade por `use_https`).
fn host_port(server: &MockServer) -> String {
    server
        .uri()
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string()
}

/// Escribe un `dime.yml` que apunta al servidor mock.
///
/// `provider` decide la forma del cuerpo (y si el token es obligatorio) y
/// `token` el valor de la cabecera `Authorization` (`""` para no enviarla).
fn write_config(dir: &Path, base_url: &str, provider: &str, token: &str) {
    let content = format!(
        "provider: {provider}\n\
         model: gpt-4o-mini\n\
         token: \"{token}\"\n\
         base_url: {base_url}\n\
         use_https: false\n\
         endpoint: {CHAT_ENDPOINT}\n\
         models_endpoint: {MODELS_ENDPOINT}\n\
         temperature: 0.2\n\
         system_role: {SYSTEM_ROLE}\n\
         log_level: error\n"
    );
    std::fs::write(dir.join("dime.yml"), content).expect("escribir config");
}

/// Ejecuta el binario `dime` en `dir`, con `stdin` cerrado (nunca un TTY).
fn run_dime(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dime"))
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .expect("ejecutar dime")
}

/// Monta un mock que responde `body` con el código y el tipo de contenido dados.
async fn mount_response(
    server: &MockServer,
    verb: &str,
    route: &str,
    status: u16,
    content_type: &str,
    body: &str,
) {
    Mock::given(method(verb))
        .and(path(route))
        .respond_with(
            ResponseTemplate::new(status)
                .insert_header("content-type", content_type)
                .set_body_string(body),
        )
        .mount(server)
        .await;
}

/// Devuelve la única petición recibida por el servidor mock.
async fn single_request(server: &MockServer) -> wiremock::Request {
    let requests = server
        .received_requests()
        .await
        .expect("el servidor debe registrar las peticiones");
    assert_eq!(
        requests.len(),
        1,
        "el servidor debe recibir exactamente una petición, recibió: {requests:#?}"
    );
    requests.into_iter().next().expect("una petición")
}

/// Afirma que el proceso escribió un error legible en `stderr` y sin *panic*.
///
/// Sólo se exige el prefijo `Error:` que emite el binario y la ausencia del
/// mensaje de *panic*: el texto concreto de cada variante (`inesperada`, etc.)
/// no se afirma para no acoplar los tests a la redacción de los mensajes.
fn assert_readable_error(stderr: &str) {
    assert!(
        stderr.contains("Error:"),
        "stderr debe contener un error legible, fue: {stderr:?}"
    );
    assert!(
        !stderr.contains("panicked"),
        "no debe haber panic, stderr: {stderr:?}"
    );
}

/// Afirma que la única petición recibida fue un `POST` a la ruta de chat.
///
/// Ancla los tests de error a la petición realmente esperada: si el binario
/// pegara a otra ruta o con otro método, el mock respondería su `404` por
/// defecto y el test de error «pasaría» sin ejercitar el escenario.
async fn assert_single_chat_request(server: &MockServer) {
    let request = single_request(server).await;
    assert_eq!(request.method.as_str(), "POST", "método de la petición");
    assert_eq!(request.url.path(), "/v1/chat/completions", "ruta");
}

/// Valor de la cabecera `Authorization`, si está presente y es texto válido.
fn authorization(request: &wiremock::Request) -> Option<&str> {
    request
        .headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
}

/// Escenario `Petición de chat con autorización`.
///
/// Afirma la petición realmente enviada (método, ruta, cabecera y cuerpo) y que
/// la respuesta simulada llega a `stdout` con código de salida cero.
#[tokio::test]
async fn chat_request_has_method_path_token_and_expected_body() {
    let server = MockServer::start().await;
    mount_response(
        &server,
        "POST",
        "/v1/chat/completions",
        200,
        "application/json",
        CHAT_BODY,
    )
    .await;

    let dir = temp_dir("chat-con-token");
    write_config(dir.path(), &host_port(&server), "openai", TOKEN);

    let output = run_dime(dir.path(), &["-q", "hola"]);

    assert!(
        output.status.success(),
        "debe terminar con código 0: {output:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("respuesta simulada"),
        "stdout debe contener la respuesta del modelo, fue: {stdout:?}"
    );

    let request = single_request(&server).await;
    assert_eq!(request.method.as_str(), "POST", "método de la petición");
    assert_eq!(request.url.path(), "/v1/chat/completions", "ruta");
    assert_eq!(
        authorization(&request).map(str::to_string),
        Some(format!("Bearer {TOKEN}")),
        "cabecera Authorization"
    );

    let body: Value = request.body_json().expect("el cuerpo debe ser JSON");
    assert_eq!(body["model"], "gpt-4o-mini", "modelo: {body}");
    assert_eq!(body["messages"][0]["role"], "system", "cuerpo: {body}");
    assert_eq!(
        body["messages"][0]["content"], SYSTEM_ROLE,
        "cuerpo: {body}"
    );
    assert_eq!(body["messages"][1]["role"], "user", "cuerpo: {body}");
    assert_eq!(body["messages"][1]["content"], "hola", "cuerpo: {body}");
    assert_eq!(body["temperature"], 0.2, "cuerpo: {body}");
}

/// Escenario `Sin token no se envía autorización`.
///
/// El proveedor `ollama` es el único que acepta token vacío y sigue realizando la
/// petición; se afirma que **no** aparece la cabecera `Authorization`.
#[tokio::test]
async fn chat_request_without_token_omits_authorization_header() {
    let server = MockServer::start().await;
    mount_response(
        &server,
        "POST",
        "/v1/chat/completions",
        200,
        "application/json",
        CHAT_BODY,
    )
    .await;

    let dir = temp_dir("chat-sin-token");
    write_config(dir.path(), &host_port(&server), "ollama", "");

    let output = run_dime(dir.path(), &["-q", "hola"]);

    assert!(
        output.status.success(),
        "debe terminar con código 0: {output:?}"
    );

    let request = single_request(&server).await;
    assert_eq!(request.method.as_str(), "POST", "método de la petición");
    assert_eq!(request.url.path(), "/v1/chat/completions", "ruta");
    assert_eq!(
        authorization(&request),
        None,
        "no debe enviarse Authorization, cabeceras: {:#?}",
        request.headers
    );
}

/// Escenario `Listado de modelos`: `--list-models` consulta el `models_endpoint`
/// configurado y muestra los identificadores devueltos.
#[tokio::test]
async fn list_models_hits_models_endpoint_and_prints_ids() {
    let server = MockServer::start().await;
    let models_body = r#"{"data":[{"id":"modelo-alfa"},{"id":"modelo-beta"}]}"#;
    mount_response(
        &server,
        "GET",
        "/v1/models",
        200,
        "application/json",
        models_body,
    )
    .await;

    let dir = temp_dir("list-models");
    write_config(dir.path(), &host_port(&server), "openai", TOKEN);

    let output = run_dime(dir.path(), &["--list-models"]);

    assert!(
        output.status.success(),
        "debe terminar con código 0: {output:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("modelo-alfa") && stdout.contains("modelo-beta"),
        "stdout debe listar los identificadores, fue: {stdout:?}"
    );

    let request = single_request(&server).await;
    assert_eq!(request.method.as_str(), "GET", "método de la petición");
    assert_eq!(request.url.path(), "/v1/models", "ruta del listado");
    assert_eq!(
        authorization(&request).map(str::to_string),
        Some(format!("Bearer {TOKEN}")),
        "el listado debe autenticarse igual que el chat"
    );
}

/// Escenario `Respuesta 401`: error legible por `stderr`, código distinto de
/// cero y nada de la respuesta del modelo en `stdout`.
#[tokio::test]
async fn unauthorized_response_exits_nonzero_with_readable_error() {
    let server = MockServer::start().await;
    let error_body = r#"{"error":{"message":"token inválido"}}"#;
    mount_response(
        &server,
        "POST",
        "/v1/chat/completions",
        401,
        "application/json",
        error_body,
    )
    .await;

    let dir = temp_dir("http-401");
    write_config(dir.path(), &host_port(&server), "openai", TOKEN);

    let output = run_dime(dir.path(), &["-q", "hola"]);

    assert!(
        !output.status.success(),
        "un 401 debe terminar con código distinto de cero: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error") && stderr.contains("token inválido"),
        "stderr debe describir el error del proveedor, fue: {stderr:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.is_empty(),
        "stdout no debe contener la respuesta del modelo, fue: {stdout:?}"
    );
}

/// Escenario `Respuesta 500`: código distinto de cero y sin *panic*.
#[tokio::test]
async fn server_error_response_exits_nonzero_without_panic() {
    let server = MockServer::start().await;
    let error_body = r#"{"error":{"message":"fallo interno del proveedor"}}"#;
    mount_response(
        &server,
        "POST",
        "/v1/chat/completions",
        500,
        "application/json",
        error_body,
    )
    .await;

    let dir = temp_dir("http-500");
    write_config(dir.path(), &host_port(&server), "openai", TOKEN);

    let output = run_dime(dir.path(), &["-q", "hola"]);

    assert!(
        !output.status.success(),
        "un 500 debe terminar con código distinto de cero: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_readable_error(&stderr);
    assert!(
        stderr.contains("fallo interno del proveedor"),
        "stderr debe describir el error del proveedor, fue: {stderr:?}"
    );
    assert_single_chat_request(&server).await;
}

/// Escenario `500 con cuerpo de chat válido`: el estado HTTP manda sobre la
/// forma del cuerpo. Este caso pasaba como éxito antes de comprobar el código.
#[tokio::test]
async fn server_error_with_valid_chat_body_is_not_accepted() {
    let server = MockServer::start().await;
    mount_response(
        &server,
        "POST",
        "/v1/chat/completions",
        500,
        "application/json",
        CHAT_BODY,
    )
    .await;

    let dir = temp_dir("http-500-cuerpo-valido");
    write_config(dir.path(), &host_port(&server), "openai", TOKEN);

    let output = run_dime(dir.path(), &["-q", "hola"]);

    assert!(
        !output.status.success(),
        "un 500 debe terminar con código distinto de cero: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_readable_error(&stderr);
    assert!(
        stderr.contains("500"),
        "stderr debe mencionar el código 500, fue: {stderr:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("respuesta simulada"),
        "stdout no debe contener el contenido del modelo, fue: {stdout:?}"
    );
    assert_single_chat_request(&server).await;
}

/// Escenario `Cuerpo sin el campo esperado`: 200 con un JSON sin `choices`.
#[tokio::test]
async fn body_without_expected_field_exits_nonzero_and_describes_problem() {
    let server = MockServer::start().await;
    mount_response(
        &server,
        "POST",
        "/v1/chat/completions",
        200,
        "application/json",
        r#"{"choices":[]}"#,
    )
    .await;

    let dir = temp_dir("cuerpo-sin-campo");
    write_config(dir.path(), &host_port(&server), "openai", TOKEN);

    let output = run_dime(dir.path(), &["-q", "hola"]);

    assert!(
        !output.status.success(),
        "un cuerpo inesperado debe terminar con código distinto de cero: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_readable_error(&stderr);
    assert!(
        output.stdout.is_empty(),
        "stdout debe quedar vacío, fue: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_single_chat_request(&server).await;
}

/// Escenario `Cuerpo que no es JSON`: 200 con texto plano, error legible y sin
/// *panic*.
#[tokio::test]
async fn non_json_body_exits_nonzero_without_panic() {
    let server = MockServer::start().await;
    mount_response(
        &server,
        "POST",
        "/v1/chat/completions",
        200,
        "text/plain",
        "esto no es JSON",
    )
    .await;

    let dir = temp_dir("cuerpo-no-json");
    write_config(dir.path(), &host_port(&server), "openai", TOKEN);

    let output = run_dime(dir.path(), &["-q", "hola"]);

    assert!(
        !output.status.success(),
        "un cuerpo no JSON debe terminar con código distinto de cero: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_readable_error(&stderr);
    assert_single_chat_request(&server).await;
}

/// Ramas de `validate()`: `provider: openai` con token vacío.
///
/// El proceso debe abortar **antes** de la petición: `stderr` menciona el token
/// y el servidor mock no recibe ninguna petición.
#[tokio::test]
async fn openai_with_empty_token_fails_without_any_request() {
    let server = MockServer::start().await;
    mount_response(
        &server,
        "POST",
        "/v1/chat/completions",
        200,
        "application/json",
        CHAT_BODY,
    )
    .await;

    let dir = temp_dir("openai-sin-token");
    write_config(dir.path(), &host_port(&server), "openai", "");

    let output = run_dime(dir.path(), &["-q", "hola"]);

    assert!(
        !output.status.success(),
        "un token vacío con `openai` debe terminar con código distinto de cero: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_readable_error(&stderr);
    assert!(
        stderr.to_lowercase().contains("token"),
        "stderr debe mencionar el token o las credenciales, fue: {stderr:?}"
    );
    assert!(
        output.stdout.is_empty(),
        "stdout debe quedar vacío, fue: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );

    let requests = server
        .received_requests()
        .await
        .expect("el servidor debe registrar las peticiones");
    assert!(
        requests.is_empty(),
        "no debe llegarse a la red con un token vacío, peticiones: {requests:#?}"
    );
}

/// Cuerpo de `ollama` tal cual viaja en el cable: `options` y `content` cadena.
#[tokio::test]
async fn ollama_chat_body_carries_options_and_string_content() {
    let server = MockServer::start().await;
    mount_response(
        &server,
        "POST",
        "/v1/chat/completions",
        200,
        "application/json",
        CHAT_BODY,
    )
    .await;

    let dir = temp_dir("ollama-cuerpo");
    write_config(dir.path(), &host_port(&server), "ollama", "");

    let output = run_dime(dir.path(), &["-q", "hola"]);

    assert!(
        output.status.success(),
        "debe terminar con código 0: {output:?}"
    );

    let request = single_request(&server).await;
    let body: Value = request.body_json().expect("el cuerpo debe ser JSON");
    assert_eq!(body["options"]["num_ctx"], 32000, "cuerpo: {body}");
    assert_eq!(body["options"]["temperature"], 0.2, "cuerpo: {body}");
    assert!(
        body.get("temperature").is_none(),
        "en ollama la temperatura va en `options`: {body}"
    );
    assert!(
        body["messages"][1]["content"].is_string(),
        "`content` del mensaje user debe ser cadena: {body}"
    );
    assert_eq!(body["messages"][1]["content"], "hola", "cuerpo: {body}");
}

/// `--list-models` con un `401` del proveedor: error legible y sin ids.
#[tokio::test]
async fn list_models_unauthorized_exits_nonzero_without_ids() {
    let server = MockServer::start().await;
    let error_body = r#"{"error":{"message":"credenciales inválidas"}}"#;
    mount_response(
        &server,
        "GET",
        "/v1/models",
        401,
        "application/json",
        error_body,
    )
    .await;

    let dir = temp_dir("list-models-401");
    write_config(dir.path(), &host_port(&server), "openai", TOKEN);

    let output = run_dime(dir.path(), &["--list-models"]);

    assert!(
        !output.status.success(),
        "un 401 en el listado debe terminar con código distinto de cero: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_readable_error(&stderr);
    assert!(
        stderr.contains("credenciales inválidas"),
        "stderr debe describir el error del proveedor, fue: {stderr:?}"
    );
    assert!(
        output.stdout.is_empty(),
        "stdout no debe listar identificadores, fue: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );

    let request = single_request(&server).await;
    assert_eq!(request.method.as_str(), "GET", "método de la petición");
    assert_eq!(request.url.path(), "/v1/models", "ruta del listado");
}

/// Escenario `Solo dependencia de desarrollo`.
///
/// `wiremock` debe figurar en `[dev-dependencies]` y no en `[dependencies]`, de
/// modo que el binario publicado no incorpore código de simulación.
#[test]
fn wiremock_is_a_dev_dependency_only() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("leer Cargo.toml");

    let (runtime_deps, rest) = manifest
        .split_once("[dev-dependencies]")
        .expect("Cargo.toml debe declarar [dev-dependencies]");

    assert!(
        !runtime_deps.contains("wiremock"),
        "wiremock no debe estar en [dependencies]"
    );
    assert!(
        rest.contains("wiremock"),
        "wiremock debe estar en [dev-dependencies]"
    );
}
