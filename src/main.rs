mod ia;

use clap::Parser;
use ia::IABot;
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
    #[arg(value_name = "QUESTION", required_unless_present = "question_flag")]
    question: Option<String>,
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
    let system_role = args
        .instructions
        .as_deref()
        .unwrap_or_else(|| iabot.get_system_role());
    let Some(question) = args.question_flag.as_deref().or(args.question.as_deref()) else {
        eprintln!("Error: no se ha proporcionado ninguna pregunta");
        process::exit(1);
    };

    match iabot.ask(system_role, question).await {
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
}
