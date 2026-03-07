use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use serde_json::{json, Value};
use std::path::PathBuf;

/// Apple epoch offset: seconds from Unix epoch (1970-01-01) to Apple epoch (2001-01-01)
const APPLE_EPOCH_OFFSET: i64 = 978307200;

fn apple_ts_to_unix(ts: i64) -> i64 {
    ts / 1_000_000_000 + APPLE_EPOCH_OFFSET
}

fn db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join("Library/Messages/chat.db")
}

fn open_db() -> Result<Connection> {
    let path = db_path();
    Connection::open(&path).with_context(|| format!("Failed to open chat.db at {:?}", path))
}

/// Fetch messages filtered by participant phone numbers, with optional limit
pub fn fetch(participants: Vec<String>, limit: Option<u32>, after_date: Option<i64>) -> Result<Value> {
    let conn = open_db()?;
    let limit = limit.unwrap_or(50).min(200);

    if participants.is_empty() {
        return Err(anyhow::anyhow!("At least one participant phone number is required"));
    }

    // Build placeholders for IN clause
    let placeholders: String = participants.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

    let sql = format!(
        "SELECT DISTINCT
            m.rowid,
            m.text,
            m.date,
            m.is_from_me,
            COALESCE(h.id, 'me') as sender_handle,
            c.chat_identifier
        FROM message m
        LEFT JOIN handle h ON m.handle_id = h.rowid
        JOIN chat_message_join cmj ON cmj.message_id = m.rowid
        JOIN chat c ON c.rowid = cmj.chat_id
        JOIN chat_handle_join chj ON chj.chat_id = c.rowid
        JOIN handle h2 ON h2.rowid = chj.handle_id
        WHERE h2.id IN ({placeholders})
          AND m.text IS NOT NULL
          AND m.text != ''
          AND (? = 0 OR m.date > ?)
        ORDER BY m.date DESC
        LIMIT ?",
        placeholders = placeholders
    );

    let mut stmt = conn.prepare(&sql)?;

    // Build params: participant list + after_date placeholder x2 + limit
    let after_apple = after_date
        .map(|unix| (unix - APPLE_EPOCH_OFFSET) * 1_000_000_000)
        .unwrap_or(0);
    let has_date_filter: i64 = if after_date.is_some() { 1 } else { 0 };

    let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = participants
        .iter()
        .map(|p| Box::new(p.clone()) as Box<dyn rusqlite::ToSql>)
        .collect();
    param_values.push(Box::new(has_date_filter));
    param_values.push(Box::new(after_apple));
    param_values.push(Box::new(limit as i64));

    let param_refs: Vec<&dyn rusqlite::ToSql> = param_values.iter().map(|v| v.as_ref()).collect();

    let messages: Vec<Value> = stmt
        .query_map(param_refs.as_slice(), |row| {
            let rowid: i64 = row.get(0)?;
            let text: String = row.get(1)?;
            let date_apple: i64 = row.get(2)?;
            let is_from_me: bool = row.get(3)?;
            let sender: String = row.get(4)?;
            let chat_id: String = row.get(5)?;
            Ok((rowid, text, date_apple, is_from_me, sender, chat_id))
        })?
        .filter_map(|r| r.ok())
        .map(|(rowid, text, date_apple, is_from_me, sender, chat_id)| {
            let unix_ts = apple_ts_to_unix(date_apple);
            json!({
                "id": rowid,
                "text": text,
                "timestamp": unix_ts,
                "is_from_me": is_from_me,
                "sender": sender,
                "chat_identifier": chat_id
            })
        })
        .collect();

    Ok(json!({ "messages": messages, "count": messages.len() }))
}

/// Full-text search across all messages
pub fn search(query: String, limit: Option<u32>) -> Result<Value> {
    let conn = open_db()?;
    let limit = limit.unwrap_or(50).min(200);
    let pattern = format!("%{}%", query);

    let sql = "SELECT
        m.rowid,
        m.text,
        m.date,
        m.is_from_me,
        COALESCE(h.id, 'me') as sender_handle,
        c.chat_identifier
    FROM message m
    LEFT JOIN handle h ON m.handle_id = h.rowid
    JOIN chat_message_join cmj ON cmj.message_id = m.rowid
    JOIN chat c ON c.rowid = cmj.chat_id
    WHERE m.text LIKE ?
      AND m.text IS NOT NULL
      AND m.text != ''
    ORDER BY m.date DESC
    LIMIT ?";

    let mut stmt = conn.prepare(sql)?;
    let messages: Vec<Value> = stmt
        .query_map(params![pattern, limit as i64], |row| {
            let rowid: i64 = row.get(0)?;
            let text: String = row.get(1)?;
            let date_apple: i64 = row.get(2)?;
            let is_from_me: bool = row.get(3)?;
            let sender: String = row.get(4)?;
            let chat_id: String = row.get(5)?;
            Ok((rowid, text, date_apple, is_from_me, sender, chat_id))
        })?
        .filter_map(|r| r.ok())
        .map(|(rowid, text, date_apple, is_from_me, sender, chat_id)| {
            let unix_ts = apple_ts_to_unix(date_apple);
            json!({
                "id": rowid,
                "text": text,
                "timestamp": unix_ts,
                "is_from_me": is_from_me,
                "sender": sender,
                "chat_identifier": chat_id
            })
        })
        .collect();

    Ok(json!({ "messages": messages, "count": messages.len(), "query": query }))
}

/// List recent conversation threads
pub fn threads(limit: Option<u32>) -> Result<Value> {
    let conn = open_db()?;
    let limit = limit.unwrap_or(20).min(100);

    let sql = "SELECT
        c.rowid as chat_id,
        c.chat_identifier,
        c.display_name,
        m.text as last_message,
        m.date as last_date,
        m.is_from_me,
        GROUP_CONCAT(DISTINCT h.id) as participants
    FROM chat c
    JOIN chat_message_join cmj ON cmj.chat_id = c.rowid
    JOIN message m ON m.rowid = cmj.message_id
    LEFT JOIN chat_handle_join chj ON chj.chat_id = c.rowid
    LEFT JOIN handle h ON h.rowid = chj.handle_id
    WHERE m.date = (
        SELECT MAX(m2.date)
        FROM chat_message_join cmj2
        JOIN message m2 ON m2.rowid = cmj2.message_id
        WHERE cmj2.chat_id = c.rowid
          AND m2.text IS NOT NULL
          AND m2.text != ''
    )
    AND m.text IS NOT NULL
    AND m.text != ''
    GROUP BY c.rowid
    ORDER BY m.date DESC
    LIMIT ?";

    let mut stmt = conn.prepare(sql)?;
    let threads: Vec<Value> = stmt
        .query_map(params![limit as i64], |row| {
            let chat_id: i64 = row.get(0)?;
            let chat_identifier: String = row.get(1)?;
            let display_name: Option<String> = row.get(2)?;
            let last_message: String = row.get(3)?;
            let last_date_apple: i64 = row.get(4)?;
            let is_from_me: bool = row.get(5)?;
            let participants: Option<String> = row.get(6)?;
            Ok((chat_id, chat_identifier, display_name, last_message, last_date_apple, is_from_me, participants))
        })?
        .filter_map(|r| r.ok())
        .map(|(chat_id, chat_identifier, display_name, last_message, last_date_apple, is_from_me, participants)| {
            let unix_ts = apple_ts_to_unix(last_date_apple);
            let participant_list: Vec<&str> = participants
                .as_deref()
                .unwrap_or("")
                .split(',')
                .filter(|s| !s.is_empty())
                .collect();
            json!({
                "chat_id": chat_id,
                "chat_identifier": chat_identifier,
                "display_name": display_name,
                "last_message": last_message,
                "last_timestamp": unix_ts,
                "last_is_from_me": is_from_me,
                "participants": participant_list
            })
        })
        .collect();

    Ok(json!({ "threads": threads, "count": threads.len() }))
}
