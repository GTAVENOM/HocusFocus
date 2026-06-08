use crate::config::Config;
use crate::hosts;
use crate::scheduler::get_blocked_domains;
use chrono::Local;
use std::collections::HashSet;
use std::path::Path;
use std::thread;
use std::time::Duration;

pub fn run_daemon(config_path: &Path, check_interval_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
    println!("[Daemon] Starting HocusFocus monitoring loop...");
    println!("[Daemon] Configuration path: {:?}", config_path);
    println!("[Daemon] Check interval: {} seconds", check_interval_secs);

    hosts::ensure_backup()?;

    ctrlc::set_handler(move || {
        println!("\n[Daemon] Received termination signal. Restoring hosts file...");
        match hosts::apply_hosts_block(&HashSet::new()) {
            Ok(_) => println!("[Daemon] Hosts file restored. Goodbye!"),
            Err(e) => eprintln!("[Error] Failed to restore hosts file on exit: {}", e),
        }
        std::process::exit(0);
    })?;

    let mut active_blocks = HashSet::new();

    let mut first_run = true;

    loop {
        let config = match Config::load_or_create(config_path) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("[Error] Failed to reload configuration: {}. Retrying in next loop.", e);
                thread::sleep(Duration::from_secs(check_interval_secs));
                continue;
            }
        };

        let now = Local::now();
        let target_blocks = get_blocked_domains(&config.rules, &now);

        if target_blocks != active_blocks || first_run {
            if target_blocks.is_empty() {
                println!("[{}] Focus off. Clearing all website blocks.", now.format("%Y-%m-%d %H:%M:%S"));
            } else {
                println!(
                    "[{}] Focus active! Blocking {} website(s): {:?}",
                    now.format("%Y-%m-%d %H:%M:%S"),
                    target_blocks.len(),
                    target_blocks
                );
            }

            if let Err(e) = hosts::apply_hosts_block(&target_blocks) {
                eprintln!("[Error] Failed to apply block rules: {}. Do you have sudo privileges?", e);
            } else {
                active_blocks = target_blocks;
                first_run = false;
            }
        }

        thread::sleep(Duration::from_secs(check_interval_secs));
    }
}