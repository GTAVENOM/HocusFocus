use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

const HOSTS_PATH: &str = "/etc/hosts";
const BACKUP_PATH: &str = "/etc/hosts.hocus-focus.bak";

const BLOCK_START: &str = "# BEGIN HOCUS-FOCUS BLOCKED WEBSITES";
const BLOCK_END: &str = "# END HOCUS-FOCUS BLOCKED WEBSITES";

pub fn ensure_backup() -> io::Result<()> {
    let original = Path::new(HOSTS_PATH);
    let backup = Path::new(BACKUP_PATH);

    if original.exists() && !backup.exists() {
        fs::copy(original, backup)?;
        println!("[Backup] Created original hosts file backup at {}", BACKUP_PATH);
    }
    Ok(())
}

fn modify_hosts_content(content: &str, domains: &HashSet<String>) -> String {
    let mut lines: Vec<&str> = content.lines().collect();

    let mut start_idx = None;
    let mut end_idx = None;

    for (i, line) in lines.iter().enumerate() {
        if line.trim() == BLOCK_START {
            start_idx = Some(i);
        } else if line.trim() == BLOCK_END {
            end_idx = Some(i);
        }
    }

    if let (Some(start), Some(end)) = (start_idx, end_idx) {
        if start <= end {
            lines.drain(start..=end);
        }
    }

    let cleaned_content = lines.join("\n");
    let mut final_content = cleaned_content.trim_end().to_string();
    final_content.push('\n');

    if !domains.is_empty() {
        final_content.push_str(BLOCK_START);
        final_content.push('\n');

        let mut sorted_domains: Vec<&String> = domains.iter().collect();
        sorted_domains.sort();

        for domain in sorted_domains {
            final_content.push_str(&format!("127.0.0.1 {}\n", domain));
            final_content.push_str(&format!("::1 {}\n", domain));
        }

        final_content.push_str(BLOCK_END);
        final_content.push('\n');
    }

    final_content
}

pub fn apply_hosts_block(domains: &HashSet<String>) -> io::Result<()> {
    ensure_backup()?;

    let content = fs::read_to_string(HOSTS_PATH)?;

    let new_content = modify_hosts_content(&content, domains);

    fs::write(HOSTS_PATH, new_content)?;

    flush_dns_cache();

    Ok(())
}

pub fn flush_dns_cache() {
    let status1 = Command::new("dscacheutil")
        .arg("-flushcache")
        .status();

    let status2 = Command::new("killall")
        .args(["-HUP", "mDNSResponder"])
        .status();

    match (status1, status2) {
        (Ok(s1), Ok(s2)) if s1.success() && s2.success() => {
            println!("[DNS] DNS cache flushed successfully.");
        }
        _ => {
            eprintln!("[Warning] Failed to flush DNS cache completely. You may need to run as root.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modify_hosts_content_add() {
        let initial = "127.0.0.1 localhost\n255.255.255.255 broadcasthost\n";
        let mut domains = HashSet::new();
        domains.insert("reddit.com".to_string());
        domains.insert("youtube.com".to_string());

        let result = modify_hosts_content(initial, &domains);

        let expected_contains_1 = "127.0.0.1 localhost";
        let expected_contains_2 = "# BEGIN HOCUS-FOCUS BLOCKED WEBSITES\n127.0.0.1 reddit.com\n::1 reddit.com\n127.0.0.1 youtube.com\n::1 youtube.com\n# END HOCUS-FOCUS BLOCKED WEBSITES\n";

        assert!(result.contains(expected_contains_1));
        assert!(result.contains(expected_contains_2));
    }

    #[test]
    fn test_modify_hosts_content_remove() {
        let initial = "127.0.0.1 localhost\n# BEGIN HOCUS-FOCUS BLOCKED WEBSITES\n127.0.0.1 reddit.com\n# END HOCUS-FOCUS BLOCKED WEBSITES\n";
        let domains = HashSet::new();

        let result = modify_hosts_content(initial, &domains);

        assert!(result.contains("127.0.0.1 localhost"));
        assert!(!result.contains("# BEGIN HOCUS-FOCUS BLOCKED WEBSITES"));
        assert!(!result.contains("reddit.com"));
    }
}