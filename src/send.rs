use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::process::Command;

/// Send an iMessage to a recipient (phone number in E.164 format or email)
pub fn send_message(recipient: &str, text: &str) -> Result<Value> {
    // Escape text for AppleScript: escape backslashes first, then quotes
    let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"");
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
