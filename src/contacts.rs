use anyhow::{Context, Result};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

struct ContactsCache {
    phone_to_name: HashMap<String, String>,
    email_to_name: HashMap<String, String>,
    loaded_at: Option<Instant>,
    source: CacheSource,
}

#[derive(Clone, Copy, PartialEq)]
enum CacheSource {
    None,
    Sqlite,
    Osascript,
}

static CACHE: OnceLock<Mutex<ContactsCache>> = OnceLock::new();

fn get_cache() -> &'static Mutex<ContactsCache> {
    CACHE.get_or_init(|| {
        Mutex::new(ContactsCache {
            phone_to_name: HashMap::new(),
            email_to_name: HashMap::new(),
            loaded_at: None,
            source: CacheSource::None,
        })
    })
}

/// Normalize a phone number by stripping spaces, dashes, parens
fn normalize_phone(phone: &str) -> String {
    phone
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect()
}

/// Find all AddressBook SQLite databases
fn find_address_book_dbs() -> Vec<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    let sources_dir = PathBuf::from(&home).join("Library/Application Support/AddressBook/Sources");

    let mut dbs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&sources_dir) {
        for entry in entries.flatten() {
            let db_path = entry.path().join("AddressBook-v22.abcddb");
            if db_path.exists() {
                dbs.push(db_path);
            }
        }
    }
    dbs
}

fn build_name(first: Option<&str>, last: Option<&str>) -> Option<String> {
    match (first, last) {
        (Some(f), Some(l)) if !f.is_empty() && !l.is_empty() => Some(format!("{} {}", f, l)),
        (Some(f), _) if !f.is_empty() => Some(f.to_string()),
        (_, Some(l)) if !l.is_empty() => Some(l.to_string()),
        _ => None,
    }
}

/// Try loading contacts from AddressBook SQLite databases (~5ms)
fn load_from_sqlite() -> Option<(HashMap<String, String>, HashMap<String, String>)> {
    let dbs = find_address_book_dbs();
    if dbs.is_empty() {
        return None;
    }

    let mut phone_map = HashMap::new();
    let mut email_map = HashMap::new();
    let mut any_success = false;

    for db_path in &dbs {
        let conn = match Connection::open(db_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if conn.pragma_update(None, "query_only", "ON").is_err() {
            continue;
        }

        // Test access by trying a simple query
        if conn.prepare("SELECT COUNT(*) FROM ZABCDRECORD").is_err() {
            continue;
        }
        any_success = true;

        // Load phone -> name mappings
        if let Ok(mut stmt) = conn.prepare(
            "SELECT p.ZFULLNUMBER, r.ZFIRSTNAME, r.ZLASTNAME
             FROM ZABCDPHONENUMBER p
             JOIN ZABCDRECORD r ON p.ZOWNER = r.Z_PK
             WHERE p.ZFULLNUMBER IS NOT NULL",
        ) {
            if let Ok(rows) = stmt.query_map([], |row| {
                let phone: String = row.get(0)?;
                let first: Option<String> = row.get(1)?;
                let last: Option<String> = row.get(2)?;
                Ok((phone, first, last))
            }) {
                for row in rows.flatten() {
                    let (phone, first, last) = row;
                    if let Some(name) = build_name(first.as_deref(), last.as_deref()) {
                        let normalized = normalize_phone(&phone);
                        if !normalized.is_empty() {
                            phone_map.insert(normalized, name);
                        }
                    }
                }
            }
        }

        // Load email -> name mappings
        if let Ok(mut stmt) = conn.prepare(
            "SELECT e.ZADDRESSNORMALIZED, r.ZFIRSTNAME, r.ZLASTNAME
             FROM ZABCDEMAILADDRESS e
             JOIN ZABCDRECORD r ON e.ZOWNER = r.Z_PK
             WHERE e.ZADDRESSNORMALIZED IS NOT NULL",
        ) {
            if let Ok(rows) = stmt.query_map([], |row| {
                let email: String = row.get(0)?;
                let first: Option<String> = row.get(1)?;
                let last: Option<String> = row.get(2)?;
                Ok((email, first, last))
            }) {
                for row in rows.flatten() {
                    let (email, first, last) = row;
                    if let Some(name) = build_name(first.as_deref(), last.as_deref()) {
                        let email_lower = email.to_lowercase();
                        if !email_lower.is_empty() {
                            email_map.insert(email_lower, name);
                        }
                    }
                }
            }
        };
    }

    if any_success {
        Some((phone_map, email_map))
    } else {
        None
    }
}

/// Fallback: load contacts via osascript (slower but works with Contacts permission)
fn load_from_osascript() -> Option<(HashMap<String, String>, HashMap<String, String>)> {
    let script = r#"tell application "Contacts"
    set output to ""
    repeat with p in every person
        set pName to name of p
        repeat with ph in phones of p
            set output to output & "P" & value of ph & "=" & pName & linefeed
        end repeat
        repeat with em in emails of p
            set output to output & "E" & value of em & "=" & pName & linefeed
        end repeat
    end repeat
    return output
end tell"#;

    let child = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .ok()?;

    let result = wait_with_timeout(child, Duration::from_secs(15))?;
    if !result.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&result.stdout);
    let mut phone_map = HashMap::new();
    let mut email_map = HashMap::new();

    for line in stdout.lines() {
        if line.len() < 2 {
            continue;
        }
        let kind = &line[..1];
        let rest = &line[1..];
        if let Some((key, name)) = rest.split_once('=') {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            match kind {
                "P" => {
                    let normalized = normalize_phone(key.trim());
                    if !normalized.is_empty() {
                        phone_map.insert(normalized, name.to_string());
                    }
                }
                "E" => {
                    let email_lower = key.trim().to_lowercase();
                    if !email_lower.is_empty() {
                        email_map.insert(email_lower, name.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    Some((phone_map, email_map))
}

fn wait_with_timeout(
    mut child: std::process::Child,
    timeout: Duration,
) -> Option<std::process::Output> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = child
                    .stdout
                    .take()
                    .map(|mut s| {
                        let mut buf = Vec::new();
                        std::io::Read::read_to_end(&mut s, &mut buf).ok();
                        buf
                    })
                    .unwrap_or_default();
                let stderr = child
                    .stderr
                    .take()
                    .map(|mut s| {
                        let mut buf = Vec::new();
                        std::io::Read::read_to_end(&mut s, &mut buf).ok();
                        buf
                    })
                    .unwrap_or_default();
                return Some(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
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

/// Load contacts cache: try SQLite first, fall back to osascript
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

    // Try SQLite first (instant)
    if let Some((phone_map, email_map)) = load_from_sqlite() {
        let mut c = cache.lock().unwrap();
        c.phone_to_name = phone_map;
        c.email_to_name = email_map;
        c.loaded_at = Some(Instant::now());
        c.source = CacheSource::Sqlite;
        return;
    }

    // Fall back to osascript (slower, but has Contacts permission via Contacts.app)
    if let Some((phone_map, email_map)) = load_from_osascript() {
        let mut c = cache.lock().unwrap();
        c.phone_to_name = phone_map;
        c.email_to_name = email_map;
        c.loaded_at = Some(Instant::now());
        c.source = CacheSource::Osascript;
        return;
    }

    // Both failed - mark as loaded but empty so we don't retry constantly
    let mut c = cache.lock().unwrap();
    c.loaded_at = Some(Instant::now());
    c.source = CacheSource::None;
}

/// Public wrapper so other modules can trigger cache loading
pub fn ensure_cache_public() {
    ensure_cache();
}

/// Find all phone/email handles whose contact name matches the query (case-insensitive contains)
pub fn find_handles_by_name(query: &str) -> Vec<String> {
    ensure_cache();
    let cache = get_cache();
    let c = cache.lock().unwrap();
    let query_lower = query.to_lowercase();

    let mut handles = Vec::new();
    for (phone, name) in &c.phone_to_name {
        if name.to_lowercase().contains(&query_lower) {
            handles.push(phone.clone());
        }
    }
    for (email, name) in &c.email_to_name {
        if name.to_lowercase().contains(&query_lower) {
            handles.push(email.clone());
        }
    }
    handles
}

/// Look up a contact name by phone number or email. Returns None if not found.
pub fn resolve_name(handle: &str) -> Option<String> {
    ensure_cache();
    let cache = get_cache();
    let c = cache.lock().unwrap();

    // Try phone lookup first
    let normalized = normalize_phone(handle);
    if let Some(name) = c.phone_to_name.get(&normalized) {
        return Some(name.clone());
    }

    // Try email lookup
    let handle_lower = handle.to_lowercase();
    if let Some(name) = c.email_to_name.get(&handle_lower) {
        return Some(name.clone());
    }

    None
}

/// Search contacts - uses osascript targeted search as primary (works with Contacts permission),
/// falls back to cache search
pub fn search(query: &str) -> Result<Value> {
    // Try targeted osascript search first (fast, filtered server-side, works with Contacts permission)
    if let Ok(results) = search_via_osascript(query) {
        if !results.is_empty() {
            return Ok(json!({
                "contacts": results,
                "count": results.len(),
                "query": query
            }));
        }
    }

    // Fall back to cache-based search
    ensure_cache();
    let cache = get_cache();
    let c = cache.lock().unwrap();
    let query_lower = query.to_lowercase();

    let mut contacts_map: HashMap<String, (Vec<String>, Vec<String>)> = HashMap::new();

    for (phone, name) in &c.phone_to_name {
        let entry = contacts_map
            .entry(name.clone())
            .or_insert_with(|| (Vec::new(), Vec::new()));
        entry.0.push(phone.clone());
    }

    for (email, name) in &c.email_to_name {
        let entry = contacts_map
            .entry(name.clone())
            .or_insert_with(|| (Vec::new(), Vec::new()));
        entry.1.push(email.clone());
    }

    let results: Vec<Value> = contacts_map
        .iter()
        .filter(|(name, (phones, emails))| {
            name.to_lowercase().contains(&query_lower)
                || phones.iter().any(|p| p.contains(&query_lower))
                || emails
                    .iter()
                    .any(|e| e.to_lowercase().contains(&query_lower))
        })
        .map(|(name, (phones, emails))| {
            json!({
                "name": name,
                "phones": phones,
                "emails": emails
            })
        })
        .collect();

    Ok(json!({
        "contacts": results,
        "count": results.len(),
        "query": query
    }))
}

/// Targeted osascript search - fast because Contacts.app filters server-side
fn search_via_osascript(query: &str) -> Result<Vec<Value>> {
    let escaped_query = query.replace('\\', "\\\\").replace('"', "\\\"");

    let script = format!(
        r#"tell application "Contacts"
    set results to every person whose name contains "{query}"
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

    let child = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn osascript")?;

    let result = wait_with_timeout(child, Duration::from_secs(10))
        .ok_or_else(|| anyhow::anyhow!("osascript timed out"))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
        return Err(anyhow::anyhow!("Contacts search failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&result.stdout).to_string();
    let contacts: Vec<Value> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty() && line.contains('|'))
        .map(|line| {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            let name = parts.first().copied().unwrap_or("").trim().to_string();
            let phones: Vec<&str> = parts
                .get(1)
                .copied()
                .unwrap_or("")
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            let emails: Vec<&str> = parts
                .get(2)
                .copied()
                .unwrap_or("")
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            json!({ "name": name, "phones": phones, "emails": emails })
        })
        .collect();

    Ok(contacts)
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
    let line = stdout.trim();
    if line.is_empty() || !line.contains('|') {
        return Ok(json!({ "me": {} }));
    }

    let parts: Vec<&str> = line.splitn(3, '|').collect();
    let name = parts.first().copied().unwrap_or("").trim();
    let phones: Vec<&str> = parts
        .get(1)
        .copied()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let emails: Vec<&str> = parts
        .get(2)
        .copied()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(json!({
        "me": {
            "name": name,
            "phones": phones,
            "emails": emails
        }
    }))
}
