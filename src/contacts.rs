use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

struct ContactsCache {
    phone_to_name: HashMap<String, String>,
    loaded_at: Option<Instant>,
}

static CACHE: OnceLock<Mutex<ContactsCache>> = OnceLock::new();

fn get_cache() -> &'static Mutex<ContactsCache> {
    CACHE.get_or_init(|| {
        Mutex::new(ContactsCache {
            phone_to_name: HashMap::new(),
            loaded_at: None,
        })
    })
}

/// Normalize a phone number by stripping spaces, dashes, parens
fn normalize_phone(phone: &str) -> String {
    phone.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect()
}

/// Load all contacts into cache (refreshes every 5 minutes).
/// Uses a timeout to avoid blocking if Contacts.app needs permission.
fn ensure_cache() {
    let cache = get_cache();
    {
        let c = cache.lock().unwrap();
        if let Some(loaded_at) = c.loaded_at {
            if loaded_at.elapsed() < Duration::from_secs(300) {
                return;
            }
        }
    }

    let script = r#"tell application "Contacts"
    set output to ""
    repeat with p in every person
        set pName to name of p
        repeat with ph in phones of p
            set output to output & value of ph & "=" & pName & linefeed
        end repeat
    end repeat
    return output
end tell"#;

    let child = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    let Ok(child) = child else { return };

    // Wait with a 5-second timeout to avoid blocking
    let result = wait_with_timeout(child, Duration::from_secs(5));

    if let Some(output) = result {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut c = cache.lock().unwrap();
            c.phone_to_name.clear();
            for line in stdout.lines() {
                if let Some((phone, name)) = line.split_once('=') {
                    let normalized = normalize_phone(phone.trim());
                    if !normalized.is_empty() && !name.trim().is_empty() {
                        c.phone_to_name.insert(normalized, name.trim().to_string());
                    }
                }
            }
            c.loaded_at = Some(Instant::now());
        }
    }
}

fn wait_with_timeout(mut child: std::process::Child, timeout: Duration) -> Option<std::process::Output> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = child.stdout.take().map(|mut s| {
                    let mut buf = Vec::new();
                    std::io::Read::read_to_end(&mut s, &mut buf).ok();
                    buf
                }).unwrap_or_default();
                let stderr = child.stderr.take().map(|mut s| {
                    let mut buf = Vec::new();
                    std::io::Read::read_to_end(&mut s, &mut buf).ok();
                    buf
                }).unwrap_or_default();
                return Some(std::process::Output { status, stdout, stderr });
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(_) => return None,
        }
    }
}

/// Look up a contact name by phone number. Returns None if not found or cache unavailable.
pub fn resolve_name(phone: &str) -> Option<String> {
    ensure_cache();
    let cache = get_cache();
    let c = cache.lock().unwrap();
    let normalized = normalize_phone(phone);
    c.phone_to_name.get(&normalized).cloned()
}

/// Parse the structured output from our AppleScript contact queries.
/// Format per contact: "NAME|PHONES|EMAILS" where PHONES and EMAILS are comma-separated
fn parse_contacts_output(output: &str) -> Vec<Value> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty() && line.contains('|'))
        .map(|line| {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            let name = parts.first().copied().unwrap_or("").trim().to_string();
            let phones_raw = parts.get(1).copied().unwrap_or("").trim().to_string();
            let emails_raw = parts.get(2).copied().unwrap_or("").trim().to_string();

            let phones: Vec<&str> = phones_raw
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            let emails: Vec<&str> = emails_raw
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            json!({
                "name": name,
                "phones": phones,
                "emails": emails
            })
        })
        .collect()
}

/// Search contacts by name, phone, or email
pub fn search(query: &str) -> Result<Value> {
    let escaped_query = query.replace('"', "\\\"");

    let script = format!(
        r#"tell application "Contacts"
    set results to every person whose name contains "{query}" or (value of phones contains "{query}") or (value of emails contains "{query}")
    set output to ""
    repeat with p in results
        set pName to name of p

        set phoneList to ""
        repeat with ph in phones of p
            if phoneList is "" then
                set phoneList to value of ph
            else
                set phoneList to phoneList & "," & value of ph
            end if
        end repeat

        set emailList to ""
        repeat with em in emails of p
            if emailList is "" then
                set emailList to value of em
            else
                set emailList to emailList & "," & value of em
            end if
        end repeat

        set output to output & pName & "|" & phoneList & "|" & emailList & linefeed
    end repeat
    return output
end tell"#,
        query = escaped_query
    );

    let result = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .context("Failed to run osascript for contacts search")?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
        return Err(anyhow::anyhow!("Contacts search failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&result.stdout).to_string();
    let contacts = parse_contacts_output(&stdout);

    Ok(json!({
        "contacts": contacts,
        "count": contacts.len(),
        "query": query
    }))
}

/// Return the user's own contact card
pub fn me() -> Result<Value> {
    let script = r#"tell application "Contacts"
    set mePerson to my card
    set pName to name of mePerson

    set phoneList to ""
    repeat with ph in phones of mePerson
        if phoneList is "" then
            set phoneList to value of ph
        else
            set phoneList to phoneList & "," & value of ph
        end if
    end repeat

    set emailList to ""
    repeat with em in emails of mePerson
        if emailList is "" then
            set emailList to value of em
        else
            set emailList to emailList & "," & value of em
        end if
    end repeat

    return pName & "|" & phoneList & "|" & emailList
end tell"#;

    let result = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .context("Failed to run osascript for contacts_me")?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
        return Err(anyhow::anyhow!("contacts_me failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&result.stdout).to_string();
    let contacts = parse_contacts_output(&stdout);
    let me = contacts.into_iter().next().unwrap_or(json!({}));

    Ok(json!({ "me": me }))
}
