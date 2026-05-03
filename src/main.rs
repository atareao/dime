mod ia;

use clap::Parser;
use ia::IABot;
use std::{env, str::FromStr};
use std::{path::PathBuf, process};
use tracing::{debug, error, info};
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
    let config = match get_config().await {
        Some(path) => path,
        None => {
            let mut path = env::current_dir().unwrap();
            path.push("dime.yml");
            IABot::write_default(&path).await;
            path
        }
    };
    let iabot = IABot::read_content(&config).await;
    if iabot.get_provider() != "ollama" && iabot.get_token() == "" {
        println!("Error: Token in {} is empty", &config.to_str().unwrap());
        process::exit(1);
    }
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(iabot.get_log_level()).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    let system_role = args
        .instructions
        .as_deref()
        .unwrap_or(iabot.get_system_role());
    let question = args
        .question_flag
        .as_deref()
        .or(args.question.as_deref())
        .unwrap();

    match iabot.ask(system_role, question).await {
        Ok(response) => {
            println!("{}", response)
        }
        Err(e) => {
            error!("{}", e);
        }
    }
}

async fn get_config() -> Option<PathBuf> {
    let mut current_path = std::env::current_dir().unwrap();
    current_path.push("dime.yml");
    debug!("Current path: {}", current_path.display());
    if (tokio::fs::metadata(&current_path)).await.is_ok() {
        return Some(current_path);
    }
    let mut exe_path = std::env::current_exe().unwrap();
    exe_path.push("dime.yml");
    debug!("Exe path: {}", exe_path.display());
    if (tokio::fs::metadata(&exe_path)).await.is_ok() {
        return Some(exe_path);
    }
    let mut home_path = dirs::home_dir().unwrap();
    debug!("Home path: {}", home_path.display());
    home_path.push(".dime.yml");
    if (tokio::fs::metadata(&home_path)).await.is_ok() {
        return Some(home_path);
    }
    let mut config_path = dirs::config_dir().unwrap();
    config_path.push("dime.yml");
    debug!("Config path: {}", config_path.display());
    if (tokio::fs::metadata(&config_path)).await.is_ok() {
        return Some(config_path);
    }
    let mut config_folder = dirs::config_dir().unwrap();
    config_folder.push("dime");
    config_folder.push("dime.yml");
    debug!("Config folder: {}", config_folder.display());
    if (tokio::fs::metadata(&config_folder)).await.is_ok() {
        return Some(config_folder);
    }
    None
}
