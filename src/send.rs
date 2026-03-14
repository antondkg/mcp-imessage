use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;

const ENABLE_SEND_ENV: &str = "MCP_IMESSAGE_ENABLE_SEND";

pub fn sending_enabled() -> bool {
    matches!(
        std::env::var(ENABLE_SEND_ENV).ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES")
    )
}

/// Send an iMessage to a recipient (phone number in E.164 format or email)
/// Supports text, file attachments, or both.
pub fn send_message(
    recipient: Option<&str>,
    chat_identifier: Option<&str>,
    text: Option<&str>,
    file_path: Option<&str>,
) -> Result<Value> {
    if !sending_enabled() {
        return Err(anyhow::anyhow!(
            "Sending is disabled by default. Set {}=1 to enable messages_send.",
            ENABLE_SEND_ENV
        ));
    }

    if recipient.is_some() && chat_identifier.is_some() {
        return Err(anyhow::anyhow!(
            "Provide either recipient or chat_identifier, not both"
        ));
    }

    if text.is_none() && file_path.is_none() {
        return Err(anyhow::anyhow!(
            "Provide either text or file_path (or both)"
        ));
    }

    let file_path = normalize_file_path(file_path)?;

    // Group chat: send to chat by GUID
    if let Some(chat_id) = chat_identifier {
        return send_to_group(chat_id, text, file_path.as_deref());
    }

    let recipient =
        recipient.ok_or_else(|| anyhow::anyhow!("Provide either recipient or chat_identifier"))?;
    let script = buddy_send_script("iMessage");

    let output = run_osascript(
        &script,
        &[
            recipient,
            text.unwrap_or_default(),
            file_path.as_deref().unwrap_or(""),
        ],
    )
    .context("Failed to run osascript")?;

    if output.status.success() {
        Ok(json!({
            "success": true,
            "recipient": recipient,
            "message": "Message sent successfully"
        }))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Try SMS fallback if iMessage buddy not found
        if stderr.contains("buddy") || stderr.contains("service") {
            let sms_script = buddy_send_script("SMS");
            let sms_output = run_osascript(
                &sms_script,
                &[
                    recipient,
                    text.unwrap_or_default(),
                    file_path.as_deref().unwrap_or(""),
                ],
            )
            .context("Failed to run osascript for SMS fallback")?;

            if sms_output.status.success() {
                return Ok(json!({
                    "success": true,
                    "recipient": recipient,
                    "message": "Message sent via SMS fallback"
                }));
            }
        }

        Err(anyhow::anyhow!(
            "Failed to send message: {} {}",
            stderr,
            stdout
        ))
    }
}

fn normalize_file_path(file_path: Option<&str>) -> Result<Option<String>> {
    let Some(path) = file_path else {
        return Ok(None);
    };

    let path = PathBuf::from(path);
    if !path.is_absolute() {
        return Err(anyhow::anyhow!(
            "file_path must be an absolute path to a file"
        ));
    }

    let canonical = path
        .canonicalize()
        .with_context(|| format!("File not found: {}", path.display()))?;

    if !canonical.is_file() {
        return Err(anyhow::anyhow!(
            "file_path must point to a regular file: {}",
            canonical.display()
        ));
    }

    Ok(Some(canonical.to_string_lossy().into_owned()))
}

fn run_osascript(script: &str, args: &[&str]) -> Result<std::process::Output> {
    let mut command = Command::new("osascript");
    command.arg("-e").arg(script).arg("--");
    for arg in args {
        command.arg(arg);
    }
    command.output().context("Failed to execute osascript")
}

fn buddy_send_script(service_type: &str) -> String {
    format!(
        r#"on run argv
    set recipient to item 1 of argv
    set messageText to item 2 of argv
    set attachmentPath to item 3 of argv

    tell application "Messages"
        set targetService to 1st service whose service type = {service_type}
        set targetBuddy to buddy recipient of targetService
        if attachmentPath is not "" then
            set theFile to POSIX file attachmentPath as alias
            send theFile to targetBuddy
        end if
        if messageText is not "" then
            send messageText to targetBuddy
        end if
        return "sent"
    end tell
end run"#
    )
}

/// Send a message to a group chat by its chat identifier (GUID)
fn send_to_group(chat_id: &str, text: Option<&str>, file_path: Option<&str>) -> Result<Value> {
    let direct_script = r#"on run argv
    set chatId to item 1 of argv
    set messageText to item 2 of argv
    set attachmentPath to item 3 of argv

    tell application "Messages"
        set targetChat to a reference to chat id ("iMessage;+;" & chatId)
        if attachmentPath is not "" then
            set theFile to POSIX file attachmentPath as alias
            send theFile to targetChat
        end if
        if messageText is not "" then
            send messageText to targetChat
        end if
        return "sent"
    end tell
end run"#;

    let output = run_osascript(
        direct_script,
        &[chat_id, text.unwrap_or_default(), file_path.unwrap_or("")],
    )
    .context("Failed to run osascript for group chat")?;

    if output.status.success() {
        return Ok(json!({
            "success": true,
            "chat_identifier": chat_id,
            "message": "Message sent to group chat"
        }));
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    // Try finding by iterating chats
    let fallback_script = r#"on run argv
    set chatId to item 1 of argv
    set messageText to item 2 of argv
    set attachmentPath to item 3 of argv

    tell application "Messages"
        set allChats to every chat
        repeat with c in allChats
            if id of c contains chatId then
                if attachmentPath is not "" then
                    set theFile to POSIX file attachmentPath as alias
                    send theFile to c
                end if
                if messageText is not "" then
                    send messageText to c
                end if
                return "sent"
            end if
        end repeat
        return "chat not found"
    end tell
end run"#;

    let fallback_output = run_osascript(
        fallback_script,
        &[chat_id, text.unwrap_or_default(), file_path.unwrap_or("")],
    )
    .context("Failed to run osascript fallback for group chat")?;

    if fallback_output.status.success() {
        let stdout = String::from_utf8_lossy(&fallback_output.stdout)
            .trim()
            .to_string();
        if stdout == "sent" {
            return Ok(json!({
                "success": true,
                "chat_identifier": chat_id,
                "message": "Message sent to group chat"
            }));
        }
        return Err(anyhow::anyhow!("Group chat not found: {}", chat_id));
    }

    Err(anyhow::anyhow!(
        "Failed to send to group chat: {} {}",
        stderr,
        String::from_utf8_lossy(&fallback_output.stderr).trim()
    ))
}
