mod ia;

use clap::Parser;
use ia::{compose_user_message, IABot, IABotError};
use std::io::{self, IsTerminal, Read};
use std::{env, str::FromStr};
use std::{path::PathBuf, process};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Instructions of ChatGPT behaivur
    #[arg(short, long)]
    instructions: Option<String>,

    /// Question for ChatGPT
    #[arg(short = 'q', long = "question", conflicts_with = "question")]
    question_flag: Option<String>,

    /// Question for ChatGPT
    #[arg(value_name = "QUESTION")]
    question: Option<String>,

    /// Lista los modelos disponibles del proveedor y termina
    #[arg(long = "list-models", conflicts_with_all = ["question_flag", "question"])]
    list_models: bool,
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

    let iabot = match IABot::read_content(&config).await {
        Ok(bot) => bot,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    };

    if let Err(e) = iabot.validate() {
        eprintln!("Error: {e} (fichero {})", config.display());
        process::exit(1);
    }

    let filter =
        EnvFilter::from_str(iabot.get_log_level()).unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

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
            println!("{}", response)
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
}
