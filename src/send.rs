use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::process::Command;

/// Send an iMessage to a recipient (phone number in E.164 format or email)
pub fn send_message(recipient: Option<&str>, chat_identifier: Option<&str>, text: &str) -> Result<Value> {
    let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"");

    // Group chat: send to chat by GUID
    if let Some(chat_id) = chat_identifier {
        return send_to_group(chat_id, &escaped_text);
    }

    let recipient = recipient.ok_or_else(|| anyhow::anyhow!("Provide either recipient or chat_identifier"))?;
    let escaped_recipient = recipient.replace('"', "\\\"");

    let script = format!(
        r#"tell application "Messages"
    set targetService to 1st service whose service type = iMessage
    set targetBuddy to buddy "{recipient}" of targetService
    send "{text}" to targetBuddy
    return "sent"
end tell"#,
        recipient = escaped_recipient,
        text = escaped_text
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
            let sms_script = format!(
                r#"tell application "Messages"
    set targetService to 1st service whose service type = SMS
    set targetBuddy to buddy "{recipient}" of targetService
    send "{text}" to targetBuddy
    return "sent"
end tell"#,
                recipient = escaped_recipient,
                text = escaped_text
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

/// Send a message to a group chat by its chat identifier (GUID)
fn send_to_group(chat_id: &str, escaped_text: &str) -> Result<Value> {
    let escaped_chat_id = chat_id.replace('"', "\\\"");

    // Find the chat by looking for its GUID prefix in Messages.app
    // Group chats in Messages.app have a GUID like "iMessage;+;chat123456789"
    let script = format!(
        r#"tell application "Messages"
    set targetChat to a reference to chat id "iMessage;+;{chat_id}"
    send "{text}" to targetChat
    return "sent"
end tell"#,
        chat_id = escaped_chat_id,
        text = escaped_text
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

    // Try without the iMessage prefix (some chats use SMS;+; or other prefixes)
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    // Try finding by iterating chats
    let fallback_script = format!(
        r#"tell application "Messages"
    set allChats to every chat
    repeat with c in allChats
        if id of c contains "{chat_id}" then
            send "{text}" to c
            return "sent"
        end if
    end repeat
    return "chat not found"
end tell"#,
        chat_id = escaped_chat_id,
        text = escaped_text
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
