use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::process::Command;

/// Send an iMessage to a recipient (phone number in E.164 format or email)
/// Supports text, file attachments, or both.
pub fn send_message(recipient: Option<&str>, chat_identifier: Option<&str>, text: Option<&str>, file_path: Option<&str>) -> Result<Value> {
    if text.is_none() && file_path.is_none() {
        return Err(anyhow::anyhow!("Provide either text or file_path (or both)"));
    }

    // Validate file exists if provided
    if let Some(path) = file_path {
        if !std::path::Path::new(path).exists() {
            return Err(anyhow::anyhow!("File not found: {}", path));
        }
    }

    // Group chat: send to chat by GUID
    if let Some(chat_id) = chat_identifier {
        return send_to_group(chat_id, text, file_path);
    }

    let recipient = recipient.ok_or_else(|| anyhow::anyhow!("Provide either recipient or chat_identifier"))?;
    let escaped_recipient = recipient.replace('"', "\\\"");

    // Build the send commands
    let send_commands = build_send_commands("targetBuddy", text, file_path);

    let script = format!(
        r#"tell application "Messages"
    set targetService to 1st service whose service type = iMessage
    set targetBuddy to buddy "{recipient}" of targetService
    {commands}
    return "sent"
end tell"#,
        recipient = escaped_recipient,
        commands = send_commands
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
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
            let sms_commands = build_send_commands("targetBuddy", text, file_path);
            let sms_script = format!(
                r#"tell application "Messages"
    set targetService to 1st service whose service type = SMS
    set targetBuddy to buddy "{recipient}" of targetService
    {commands}
    return "sent"
end tell"#,
                recipient = escaped_recipient,
                commands = sms_commands
            );

            let sms_output = Command::new("osascript")
                .arg("-e")
                .arg(&sms_script)
                .output()
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

/// Build AppleScript send commands for text and/or file
fn build_send_commands(target_var: &str, text: Option<&str>, file_path: Option<&str>) -> String {
    let mut commands = Vec::new();
    if let Some(path) = file_path {
        let escaped_path = path.replace('"', "\\\"");
        commands.push(format!("send POSIX file \"{}\" to {}", escaped_path, target_var));
    }
    if let Some(t) = text {
        let escaped = t.replace('\\', "\\\\").replace('"', "\\\"");
        commands.push(format!("send \"{}\" to {}", escaped, target_var));
    }
    commands.join("\n    ")
}

/// Send a message to a group chat by its chat identifier (GUID)
fn send_to_group(chat_id: &str, text: Option<&str>, file_path: Option<&str>) -> Result<Value> {
    let escaped_chat_id = chat_id.replace('"', "\\\"");
    let send_commands = build_send_commands("targetChat", text, file_path);

    let script = format!(
        r#"tell application "Messages"
    set targetChat to a reference to chat id "iMessage;+;{chat_id}"
    {commands}
    return "sent"
end tell"#,
        chat_id = escaped_chat_id,
        commands = send_commands
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
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
    let fallback_commands = build_send_commands("c", text, file_path);
    let fallback_script = format!(
        r#"tell application "Messages"
    set allChats to every chat
    repeat with c in allChats
        if id of c contains "{chat_id}" then
            {commands}
            return "sent"
        end if
    end repeat
    return "chat not found"
end tell"#,
        chat_id = escaped_chat_id,
        commands = fallback_commands
    );

    let fallback_output = Command::new("osascript")
        .arg("-e")
        .arg(&fallback_script)
        .output()
        .context("Failed to run osascript fallback for group chat")?;

    if fallback_output.status.success() {
        let stdout = String::from_utf8_lossy(&fallback_output.stdout).trim().to_string();
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
