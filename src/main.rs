mod ia;

use clap::{CommandFactory, Parser, ValueEnum};
use ia::{compose_user_message, ChatResponse, IABot, IABotError};
use std::io::{self, IsTerminal, Read, Write};
use std::{env, str::FromStr};
use std::{path::PathBuf, process};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Instructions of ChatGPT behaivur (alias: --system)
    #[arg(short, long, alias = "system")]
    instructions: Option<String>,

    /// Sobreescribe `model` de `dime.yml` sólo para esta invocación
    #[arg(long = "model")]
    model: Option<String>,

    /// Sobreescribe `temperature` de `dime.yml` sólo para esta invocación
    #[arg(long = "temperature")]
    temperature: Option<f64>,

    /// Sobreescribe `provider` (`openai`/`ollama`) sólo para esta invocación
    #[arg(long = "provider")]
    provider: Option<String>,

    /// Question for ChatGPT
    #[arg(short = 'q', long = "question", conflicts_with = "question")]
    question_flag: Option<String>,

    /// Question for ChatGPT
    #[arg(value_name = "QUESTION")]
    question: Option<String>,

    /// Lista los modelos disponibles del proveedor y termina
    #[arg(long = "list-models", conflicts_with_all = ["question_flag", "question"])]
    list_models: bool,

    /// Emite la respuesta como un único objeto JSON en stdout (content, model, usage)
    #[arg(long = "json", conflicts_with = "list_models")]
    json: bool,

    /// Garantiza que la salida no contiene secuencias de escape ANSI
    #[arg(long = "no-color")]
    no_color: bool,

    /// Imprime el script de autocompletado del shell indicado y termina
    ///
    /// No exige pregunta, no lee `stdin` y no requiere (ni crea) `dime.yml`.
    #[arg(
        long = "completions",
        value_name = "SHELL",
        value_enum,
        conflicts_with_all = ["question", "question_flag", "list_models"]
    )]
    completions: Option<CompletionShell>,
}

/// Shell soportado por `--completions`.
///
/// Se limita a los tres shells del plan (bash, zsh, fish). El resto de shells de
/// `clap_complete` (elvish, powershell) no se exponen todavía; un valor fuera de
/// este conjunto produce un error de uso de `clap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CompletionShell {
    Bash,
    Zsh,
    Fish,
}

impl From<CompletionShell> for clap_complete::Shell {
    fn from(shell: CompletionShell) -> Self {
        match shell {
            CompletionShell::Bash => clap_complete::Shell::Bash,
            CompletionShell::Zsh => clap_complete::Shell::Zsh,
            CompletionShell::Fish => clap_complete::Shell::Fish,
        }
    }
}

/// Escribe en `writer` el script de autocompletado de `dime` para `shell`.
///
/// Toda la E/S va al `writer` recibido, de modo que la generación pueda testearse
/// con un buffer en memoria sin depender del binario ni de `stdout`.
fn generate_completions(shell: CompletionShell, writer: &mut dyn Write) {
    let mut command = Args::command();
    let shell: clap_complete::Shell = shell.into();
    clap_complete::generate(shell, &mut command, "dime", writer);
}

/// Estado de la entrada estándar en el momento de resolver la pregunta.
///
/// Se modela explícitamente para no leer nunca de un `stdin` interactivo y
/// para poder decidir la fuente de la pregunta con una función pura y testeable.
#[derive(Debug, PartialEq, Eq)]
enum StdinState<'a> {
    /// `stdin` es un TTY (uso interactivo): nunca se lee.
    Tty,
    /// `stdin` no es un TTY y no aporta datos (EOF inmediato).
    Empty,
    /// `stdin` no es un TTY y aporta estos datos.
    Data(&'a str),
}

/// Resultado de resolver la entrada del usuario a partir de los argumentos y de
/// `stdin`.
#[derive(Debug, PartialEq, Eq)]
enum ResolvedInput {
    /// Pregunta simple, sin contexto adicional.
    Question(String),
    /// Pregunta más contexto leído de `stdin`.
    QuestionWithContext { question: String, context: String },
}

/// Decide la fuente de la pregunta y del contexto a partir de la pregunta
/// aportada por argumentos y del estado de `stdin`.
///
/// Es una función pura: no toca `stdin` ni el sistema de archivos.
fn resolve_input(
    question_arg: Option<&str>,
    stdin: StdinState<'_>,
) -> Result<ResolvedInput, IABotError> {
    match stdin {
        // Uso interactivo: nunca se lee de stdin. Sólo vale una pregunta real
        // (el marcador `-` implicaría leer de un terminal interactivo).
        StdinState::Tty => match question_arg {
            Some(question) if question != "-" => Ok(ResolvedInput::Question(question.to_string())),
            _ => Err(IABotError::Input(
                "no se ha proporcionado ninguna pregunta; usa -q/posicional o canaliza la entrada por stdin"
                    .to_string(),
            )),
        },
        // Sin datos y sin pregunta no hay nada que enviar.
        StdinState::Empty => match question_arg {
            Some(question) if question != "-" => Ok(ResolvedInput::Question(question.to_string())),
            _ => Err(IABotError::Input(
                "stdin está vacío y no se ha proporcionado ninguna pregunta".to_string(),
            )),
        },
        // Con datos: la pregunta sale de stdin (sin argumento o con `-`) o bien
        // se usa el argumento como pregunta y stdin como contexto.
        StdinState::Data(data) => match question_arg {
            Some("-") | None => Ok(ResolvedInput::Question(data.trim().to_string())),
            Some(question) => Ok(ResolvedInput::QuestionWithContext {
                question: question.to_string(),
                context: data.trim_end().to_string(),
            }),
        },
    }
}

#[tokio::main]
async fn main() {
    info!("main");

    // Se parsean los argumentos antes de tocar el fichero de configuración:
    // `--help`/`--version` no deben crear ni leer `dime.yml`, y un error de uso
    // (p. ej. `--temperature` no numérica) termina sin realizar ninguna petición.
    let args = Args::parse();

    // `--completions` no exige pregunta, no lee `stdin` ni toca la configuración:
    // se atiende antes de resolver/crear `dime.yml` y antes de leer la entrada.
    if let Some(shell) = args.completions {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        generate_completions(shell, &mut handle);
        let _ = handle.flush();
        return;
    }

    let config = match get_config() {
        Some(path) => path,
        None => {
            let mut path = match env::current_dir() {
                Ok(dir) => dir,
                Err(e) => {
                    eprintln!("Error: no se pudo obtener el directorio actual: {e}");
                    process::exit(1);
                }
            };
            path.push("dime.yml");
            if let Err(e) = IABot::write_default(&path).await {
                eprintln!("Error: {e}");
                process::exit(1);
            }
            path
        }
    };

    let mut iabot = match IABot::read_content(&config).await {
        Ok(bot) => bot,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    };

    // Los overrides se aplican ANTES de `validate()` (con `--provider ollama`
    // no se exige token) y sin reescribir `dime.yml`.
    if let Err(e) = iabot.apply_overrides(args.model, args.temperature, args.provider) {
        eprintln!("Error: {e}");
        process::exit(1);
    }

    if let Err(e) = iabot.validate() {
        eprintln!("Error: {e} (fichero {})", config.display());
        process::exit(1);
    }

    // El logging nunca debe contaminar `stdout`, reservado para la respuesta
    // (`--json` o el contenido). El color del log se decide por su destino real:
    // sólo ANSI cuando `stderr` es un TTY y no se pasa `--no-color`.
    let filter =
        EnvFilter::from_str(iabot.get_log_level()).unwrap_or_else(|_| EnvFilter::new("info"));
    let ansi = should_colorize(args.no_color, std::io::stderr().is_terminal());
    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(ansi),
        )
        .init();

    // `--list-models` ignora por completo `stdin`: se resuelve antes de tocarlo.
    if args.list_models {
        match iabot.list_models().await {
            Ok(models) => {
                for id in models {
                    println!("{}", id);
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
                error!("{}", e);
                process::exit(1);
            }
        }
        return;
    }

    let system_role = args
        .instructions
        .as_deref()
        .unwrap_or_else(|| iabot.get_system_role());

    // Sólo se lee `stdin` cuando NO es un TTY; en modo interactivo nunca se
    // bloquea la ejecución esperando datos.
    let stdin_data = if io::stdin().is_terminal() {
        None
    } else {
        match read_stdin() {
            Ok(data) => Some(data),
            Err(e) => {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
    };
    let stdin_state = match stdin_data.as_deref() {
        None => StdinState::Tty,
        Some(data) if data.trim().is_empty() => StdinState::Empty,
        Some(data) => StdinState::Data(data),
    };

    let question_arg = args.question_flag.as_deref().or(args.question.as_deref());
    let resolved = match resolve_input(question_arg, stdin_state) {
        Ok(resolved) => resolved,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    };

    let question = match resolved {
        ResolvedInput::Question(question) => question,
        ResolvedInput::QuestionWithContext { question, context } => {
            compose_user_message(&question, Some(&context))
        }
    };

    match iabot.ask(system_role, &question).await {
        Ok(response) => {
            println!("{}", render_output(&response, args.json, iabot.get_model()));
        }
        Err(e) => {
            eprintln!("Error: {e}");
            error!("{}", e);
            process::exit(1);
        }
    }
}

/// Lee todo `stdin` como texto UTF-8.
///
/// Capa fina sobre la entrada estándar; las decisiones sobre qué hacer con los
/// datos viven en [`resolve_input`], que es pura y testeable.
fn read_stdin() -> Result<String, IABotError> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|e| IABotError::Input(format!("no se pudo leer de stdin: {e}")))?;
    Ok(buffer)
}

/// Compone la salida que se escribe en `stdout`.
///
/// Con `json = true` devuelve exactamente un objeto JSON compacto con las claves
/// `content`, `model` y `usage`. `model` es siempre una cadena: el modelo
/// informado por el proveedor si está presente y `default_model` en caso
/// contrario; `usage` es `null` si no se aporta. Con `json = false` devuelve el
/// contenido tal cual, sin alterarlo. En ningún caso emite secuencias de escape
/// ANSI.
fn render_output(response: &ChatResponse, json: bool, default_model: &str) -> String {
    if json {
        serde_json::json!({
            "content": response.content,
            "model": response.model.as_deref().unwrap_or(default_model),
            "usage": response.usage,
        })
        .to_string()
    } else {
        response.content.clone()
    }
}

/// Decide si la salida de diagnóstico puede llevar color.
///
/// Sólo cuando el destino (`stderr`) es un TTY y no se pasa `--no-color`. Se usa
/// para configurar `with_ansi` del layer de `tracing`, de modo que nunca se
/// emiten secuencias de escape ANSI cuando la salida está canalizada o
/// redirigida.
fn should_colorize(no_color: bool, stderr_is_terminal: bool) -> bool {
    !no_color && stderr_is_terminal
}

/// Construye, en orden, los posibles emplazamientos de `dime.yml` según la
/// especificación: directorio actual, directorio del ejecutable, `~/.dime.yml`,
/// directorio de configuración y directorio de configuración/`dime`.
fn config_candidates(
    cwd: Option<PathBuf>,
    exe_dir: Option<PathBuf>,
    home: Option<PathBuf>,
    config_dir: Option<PathBuf>,
) -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(dir) = cwd {
        candidates.push(dir.join("dime.yml"));
    }
    if let Some(dir) = exe_dir {
        candidates.push(dir.join("dime.yml"));
    }
    if let Some(home) = home {
        candidates.push(home.join(".dime.yml"));
    }
    if let Some(config_dir) = config_dir {
        candidates.push(config_dir.join("dime.yml"));
        candidates.push(config_dir.join("dime").join("dime.yml"));
    }

    candidates
}

/// Devuelve el primer candidato que existe según el predicado dado.
fn first_existing(candidates: &[PathBuf], exists: impl Fn(&PathBuf) -> bool) -> Option<PathBuf> {
    candidates.iter().find(|path| exists(path)).cloned()
}

/// Localiza `dime.yml` en el orden establecido por la especificación.
fn get_config() -> Option<PathBuf> {
    let candidates = config_candidates(
        env::current_dir().ok(),
        env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(PathBuf::from)),
        dirs::home_dir(),
        dirs::config_dir(),
    );

    first_existing(&candidates, |path| path.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cands() -> Vec<PathBuf> {
        config_candidates(
            Some(PathBuf::from("/cwd")),
            Some(PathBuf::from("/exe")),
            Some(PathBuf::from("/home/user")),
            Some(PathBuf::from("/etc")),
        )
    }

    #[test]
    fn config_candidates_order_is_exact() {
        assert_eq!(
            cands(),
            vec![
                PathBuf::from("/cwd/dime.yml"),
                PathBuf::from("/exe/dime.yml"),
                PathBuf::from("/home/user/.dime.yml"),
                PathBuf::from("/etc/dime.yml"),
                PathBuf::from("/etc/dime/dime.yml"),
            ]
        );
    }

    #[test]
    fn config_candidates_omits_missing_sources() {
        assert!(config_candidates(None, None, None, None).is_empty());
    }

    #[test]
    fn first_existing_picks_current_dir_first() {
        let c = cands();
        assert_eq!(first_existing(&c, |p| p == &c[0]), Some(c[0].clone()));
    }

    #[test]
    fn first_existing_returns_none_when_nothing_exists() {
        let c = cands();
        assert_eq!(first_existing(&c, |_| false), None);
    }

    // --- models spec: Modo listado sin pregunta ---

    #[test]
    fn list_models_parses_without_question() {
        let args = Args::try_parse_from(["dime", "--list-models"]).expect("debe parsear");
        assert!(args.list_models);
        assert!(args.question.is_none());
        assert!(args.question_flag.is_none());
    }

    #[test]
    fn question_is_optional_in_clap() {
        let args = Args::try_parse_from(["dime"]).expect("debe parsear sin pregunta");
        assert!(!args.list_models);
        assert!(args.question.is_none());
        assert!(args.question_flag.is_none());
    }

    #[test]
    fn positional_question_parses_in_normal_mode() {
        let args = Args::try_parse_from(["dime", "hola"]).expect("debe parsear");
        assert!(!args.list_models);
        assert_eq!(args.question.as_deref(), Some("hola"));
    }

    #[test]
    fn question_flag_parses_in_normal_mode() {
        let args = Args::try_parse_from(["dime", "-q", "hola"]).expect("debe parsear");
        assert_eq!(args.question_flag.as_deref(), Some("hola"));
        assert!(!args.list_models);
    }

    #[test]
    fn list_models_conflicts_with_positional_question() {
        let err = Args::try_parse_from(["dime", "--list-models", "hola"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn list_models_conflicts_with_question_flag() {
        let err = Args::try_parse_from(["dime", "--list-models", "-q", "hola"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    // --- stdin-input spec: Entrada desde stdin ---

    #[test]
    fn scenario_question_only_from_stdin() {
        let resolved = resolve_input(None, StdinState::Data("hola\n")).expect("debe resolver");
        assert_eq!(resolved, ResolvedInput::Question("hola".to_string()));
    }

    #[test]
    fn scenario_dash_marker_forces_stdin() {
        let resolved = resolve_input(Some("-"), StdinState::Data("hola\n")).expect("debe resolver");
        assert_eq!(resolved, ResolvedInput::Question("hola".to_string()));
    }

    #[test]
    fn scenario_question_plus_context_from_stdin() {
        let resolved = resolve_input(Some("resume"), StdinState::Data("diff --git a b\n"))
            .expect("debe resolver");
        let ResolvedInput::QuestionWithContext { question, context } = resolved else {
            panic!("esperado QuestionWithContext");
        };
        let message = compose_user_message(&question, Some(&context));
        assert!(message.contains("resume"), "mensaje: {message}");
        assert!(message.contains("diff --git a b"), "mensaje: {message}");
    }

    #[test]
    fn scenario_stdin_without_data_or_question_errors() {
        let err = resolve_input(None, StdinState::Empty).expect_err("debe fallar");
        assert!(matches!(err, IABotError::Input(_)), "err: {err}");
    }

    #[test]
    fn scenario_tty_without_question_is_usage_error() {
        let err = resolve_input(None, StdinState::Tty).expect_err("debe fallar");
        assert!(matches!(err, IABotError::Input(_)), "err: {err}");
    }

    #[test]
    fn tty_with_question_does_not_use_stdin() {
        let resolved = resolve_input(Some("hola"), StdinState::Tty).expect("debe resolver");
        assert_eq!(resolved, ResolvedInput::Question("hola".to_string()));
    }

    #[test]
    fn empty_stdin_with_question_is_plain_question() {
        let resolved = resolve_input(Some("hola"), StdinState::Empty).expect("debe resolver");
        assert_eq!(resolved, ResolvedInput::Question("hola".to_string()));
    }

    #[test]
    fn dash_marker_with_empty_stdin_errors() {
        let err = resolve_input(Some("-"), StdinState::Empty).expect_err("debe fallar");
        assert!(matches!(err, IABotError::Input(_)), "err: {err}");
    }

    #[test]
    fn compose_user_message_without_context_is_question() {
        assert_eq!(compose_user_message("hola", None), "hola");
    }

    #[test]
    fn compose_user_message_ignores_blank_context() {
        assert_eq!(compose_user_message("hola", Some("   \n")), "hola");
    }

    // --- output spec: Salida JSON ---

    fn chat_response() -> ChatResponse {
        ChatResponse {
            content: "Hola".to_string(),
            model: Some("gpt-4o".to_string()),
            usage: Some(serde_json::json!({"prompt_tokens": 5})),
        }
    }

    #[test]
    fn render_json_has_content_model_and_usage() {
        let out = render_output(&chat_response(), true, "gpt-3.5-turbo");
        let value: serde_json::Value = serde_json::from_str(&out).expect("json valido");
        assert_eq!(value["content"], "Hola");
        assert_eq!(value["model"], "gpt-4o");
        assert_eq!(value["usage"]["prompt_tokens"], 5);
    }

    #[test]
    fn render_json_without_usage_is_null_and_model_falls_back() {
        let response = ChatResponse {
            content: "Hola".to_string(),
            model: None,
            usage: None,
        };
        let out = render_output(&response, true, "gpt-3.5-turbo");
        let value: serde_json::Value = serde_json::from_str(&out).expect("json valido");
        assert_eq!(value["model"], "gpt-3.5-turbo");
        assert!(value["usage"].is_null());
    }

    #[test]
    fn render_default_is_plain_content() {
        assert_eq!(
            render_output(&chat_response(), false, "gpt-3.5-turbo"),
            "Hola"
        );
    }

    // --- output spec: Control de color ---

    #[test]
    fn render_never_emits_ansi() {
        for json in [true, false] {
            let out = render_output(&chat_response(), json, "gpt-3.5-turbo");
            assert!(!out.contains('\u{1b}'), "ANSI inesperado: {out:?}");
        }
    }

    #[test]
    fn should_colorize_requires_tty_and_no_no_color() {
        assert!(!should_colorize(true, true));
        assert!(!should_colorize(true, false));
        assert!(!should_colorize(false, false));
        assert!(should_colorize(false, true));
    }

    // --- output spec: flags ---

    #[test]
    fn json_and_no_color_flags_parse() {
        let args = Args::try_parse_from(["dime", "--json", "--no-color", "hola"]).expect("parse");
        assert!(args.json);
        assert!(args.no_color);
        assert_eq!(args.question.as_deref(), Some("hola"));
    }

    #[test]
    fn json_and_no_color_default_to_false() {
        let args = Args::try_parse_from(["dime", "hola"]).expect("parse");
        assert!(!args.json);
        assert!(!args.no_color);
    }

    #[test]
    fn json_conflicts_with_list_models() {
        let err = Args::try_parse_from(["dime", "--json", "--list-models"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    // --- cli-overrides spec: flags de override ---

    #[test]
    fn override_flags_default_to_none() {
        let args = Args::try_parse_from(["dime", "hola"]).expect("parse");
        assert!(args.model.is_none());
        assert!(args.temperature.is_none());
        assert!(args.provider.is_none());
    }

    #[test]
    fn override_flags_parse() {
        let args = Args::try_parse_from([
            "dime",
            "--model",
            "gpt-4o",
            "--temperature",
            "0.2",
            "--provider",
            "ollama",
            "hola",
        ])
        .expect("parse");
        assert_eq!(args.model.as_deref(), Some("gpt-4o"));
        assert_eq!(args.temperature, Some(0.2));
        assert_eq!(args.provider.as_deref(), Some("ollama"));
    }

    #[test]
    fn system_is_alias_of_instructions() {
        let args = Args::try_parse_from(["dime", "--system", "eres un experto", "-q", "hola"])
            .expect("parse");
        assert_eq!(args.instructions.as_deref(), Some("eres un experto"));
    }

    #[test]
    fn temperature_non_numeric_is_usage_error() {
        let err = Args::try_parse_from(["dime", "--temperature", "alta", "hola"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ValueValidation);
    }

    // --- completions spec: Generación de scripts de completado ---

    #[test]
    fn completions_flag_parses_each_supported_shell() {
        for (arg, expected) in [
            ("bash", CompletionShell::Bash),
            ("zsh", CompletionShell::Zsh),
            ("fish", CompletionShell::Fish),
        ] {
            let args = Args::try_parse_from(["dime", "--completions", arg]).expect("debe parsear");
            assert_eq!(args.completions, Some(expected));
        }
    }

    #[test]
    fn completions_defaults_to_none() {
        let args = Args::try_parse_from(["dime", "hola"]).expect("debe parsear");
        assert!(args.completions.is_none());
    }

    #[test]
    fn completions_rejects_unsupported_shell() {
        let err = Args::try_parse_from(["dime", "--completions", "elvish"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::InvalidValue);
    }

    #[test]
    fn completions_conflicts_with_question_flag() {
        let err =
            Args::try_parse_from(["dime", "--completions", "bash", "-q", "hola"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn completions_conflicts_with_positional_question() {
        let err = Args::try_parse_from(["dime", "--completions", "bash", "hola"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn completions_conflicts_with_list_models() {
        let err =
            Args::try_parse_from(["dime", "--completions", "bash", "--list-models"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn generate_completions_writes_script_naming_binary() {
        for shell in [
            CompletionShell::Bash,
            CompletionShell::Zsh,
            CompletionShell::Fish,
        ] {
            let mut buf: Vec<u8> = Vec::new();
            generate_completions(shell, &mut buf);
            let script = String::from_utf8(buf).expect("script UTF-8");
            assert!(!script.trim().is_empty(), "script vacío para {shell:?}");
            assert!(script.contains("dime"), "script sin `dime` para {shell:?}");
        }
    }
}
