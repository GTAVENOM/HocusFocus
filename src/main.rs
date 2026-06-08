pub mod config;
pub mod daemon;
pub mod hosts;
pub mod scheduler;
pub mod service;

use clap::{Parser, Subcommand};
use config::Config;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hocus-focus")]
#[command(about = "A scheduled website blocker daemon for macOS", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,

    Validate,

    Check,

    Daemon {
        #[arg(short, long, default_value_t = 10)]
        interval: u64,
    },

    Install,

    Uninstall,

    Status,
}

fn main() {
    let cli = Cli::parse();

    let config_path = match get_config_path(cli.config) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("[Error] {}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Init => {
            println!("[Init] Initializing configuration...");
            match Config::load_or_create(&config_path) {
                Ok(_) => println!("[Init] Configuration successfully initialized at {:?}", config_path),
                Err(e) => {
                    eprintln!("[Error] Failed to initialize config: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Validate => {
            println!("[Validate] Loading configuration file at {:?}", config_path);
            match Config::load_or_create(&config_path) {
                Ok(config) => {
                    println!("[Validate] Configuration file is valid! Loaded {} rule(s).", config.rules.len());
                    for rule in config.rules {
                        println!("  Rule: \"{}\" (blocks: {})", rule.name, rule.domains.join(", "));
                        for sched in rule.schedules {
                            let days = if sched.days.is_empty() { "Everyday".to_string() } else { sched.days.join(", ") };
                            println!("    Schedule: {} from {} to {}", days, sched.start, sched.end);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[Error] Configuration file is invalid: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Check => {
            match Config::load_or_create(&config_path) {
                Ok(config) => {
                    let now = chrono::Local::now();
                    let blocked = scheduler::get_blocked_domains(&config.rules, &now);
                    println!("Current Time: {}", now.format("%Y-%m-%d %H:%M:%S (%A)"));
                    if blocked.is_empty() {
                        println!("Status: Focus off. No websites are currently blocked.");
                    } else {
                        println!("Status: Focus active! Currently blocking {} websites:", blocked.len());
                        for domain in blocked {
                            println!("  - {}", domain);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[Error] Failed to load config: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Daemon { interval } => {
            if let Err(e) = daemon::run_daemon(&config_path, interval) {
                eprintln!("[Error] Daemon crashed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Install => {
            if let Err(e) = service::install() {
                eprintln!("[Error] Installation failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Uninstall => {
            if let Err(e) = service::uninstall() {
                eprintln!("[Error] Uninstallation failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Status => {
            println!("Configuration Path: {:?}", config_path);
            service::print_status();

            if config_path.exists() {
                if let Ok(config) = Config::load_or_create(&config_path) {
                    let now = chrono::Local::now();
                    let blocked = scheduler::get_blocked_domains(&config.rules, &now);
                    println!("Active blocks now:  {:?}", blocked);
                }
            }
        }
    }
}

fn get_config_path(custom_path: Option<PathBuf>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(path) = custom_path {
        Ok(path)
    } else if let Some(path) = config::get_default_config_path() {
        Ok(path)
    } else {
        Err("Could not resolve default config directory. Please specify a config file using --config <path>.".into())
    }
}