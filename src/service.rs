use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

const PLIST_PATH: &str = "/Library/LaunchDaemons/com.hocusfocus.daemon.plist";
const BINARY_DEST: &str = "/usr/local/bin/hocus-focus";

const PLIST_CONTENT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.hocusfocus.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/hocus-focus</string>
        <string>daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/var/log/hocus-focus.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/hocus-focus.err.log</string>
</dict>
</plist>
"#;

pub fn install() -> Result<(), Box<dyn std::error::Error>> {
    if !is_root() {
        return Err("This command requires administrator privileges. Please run with 'sudo'.".into());
    }

    println!("[Service] Installing HocusFocus as a system service...");

    let current_exe = std::env::current_exe()?;
    if let Some(parent) = Path::new(BINARY_DEST).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&current_exe, BINARY_DEST)?;
    println!("[Service] Copied binary to {}", BINARY_DEST);

    fs::write(PLIST_PATH, PLIST_CONTENT)?;
    println!("[Service] Created plist at {}", PLIST_PATH);

    let status_chown = Command::new("chown")
        .args(["root:wheel", PLIST_PATH])
        .status()?;
    let status_chmod = Command::new("chmod")
        .args(["644", PLIST_PATH])
        .status()?;

    if !status_chown.success() || !status_chmod.success() {
        return Err("Failed to set correct ownership and permissions for the launchd plist file.".into());
    }

    let _ = Command::new("launchctl")
        .args(["bootout", "system", PLIST_PATH])
        .status();

    let status_load = Command::new("launchctl")
        .args(["bootstrap", "system", PLIST_PATH])
        .status()?;

    if status_load.success() {
        println!("[Service] HocusFocus service successfully loaded and started!");
    } else {
        return Err("Failed to load service with launchctl bootstrap.".into());
    }

    Ok(())
}

pub fn uninstall() -> Result<(), Box<dyn std::error::Error>> {
    if !is_root() {
        return Err("This command requires administrator privileges. Please run with 'sudo'.".into());
    }

    println!("[Service] Uninstalling HocusFocus system service...");

    let status_unload = Command::new("launchctl")
        .args(["bootout", "system", PLIST_PATH])
        .status();

    match status_unload {
        Ok(s) if s.success() => println!("[Service] Stopped and unloaded launchd service."),
        _ => println!("[Service] Service was not running or failed to stop via launchctl."),
    }

    if Path::new(PLIST_PATH).exists() {
        fs::remove_file(PLIST_PATH)?;
        println!("[Service] Removed plist file at {}", PLIST_PATH);
    }

    if Path::new(BINARY_DEST).exists() {
        fs::remove_file(BINARY_DEST)?;
        println!("[Service] Removed binary at {}", BINARY_DEST);
    }

    println!("[Service] Restoring hosts file...");
    let _ = crate::hosts::apply_hosts_block(&std::collections::HashSet::new());

    println!("[Service] Uninstall completed successfully.");
    Ok(())
}

pub fn print_status() {
    let plist_exists = Path::new(PLIST_PATH).exists();
    let binary_exists = Path::new(BINARY_DEST).exists();

    println!("--- HocusFocus Service Status ---");
    println!("Binary installed:  {}", if binary_exists { "Yes (/usr/local/bin/hocus-focus)" } else { "No" });
    println!("Plist installed:   {}", if plist_exists { "Yes (/Library/LaunchDaemons/...)" } else { "No" });

    let output = Command::new("launchctl")
        .args(["print", "system/com.hocusfocus.daemon"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            println!("Service State:     Running (Active)");
            if let Ok(details) = String::from_utf8(out.stdout) {
                if let Some(pid_line) = details.lines().find(|l| l.contains("pid =")) {
                    println!("Service Details:   {}", pid_line.trim());
                }
            }
        }
        _ => {
            println!("Service State:     Stopped (Not loaded)");
        }
    }
}

fn is_root() -> bool {
    if let Ok(output) = Command::new("id").arg("-u").output() {
        if let Ok(uid_str) = String::from_utf8(output.stdout) {
            return uid_str.trim() == "0";
        }
    }
    false
}