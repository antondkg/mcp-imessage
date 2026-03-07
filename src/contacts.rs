use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::process::Command;

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
    set mePerson to me
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
