use crate::contacts;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::path::PathBuf;

const APPLE_EPOCH_OFFSET: i64 = 978307200;

fn apple_ts_to_unix(ts: i64) -> i64 {
    ts / 1_000_000_000 + APPLE_EPOCH_OFFSET
}

fn unix_to_apple(ts: i64) -> i64 {
    (ts - APPLE_EPOCH_OFFSET) * 1_000_000_000
}

fn db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join("Library/Messages/chat.db")
}

fn open_db() -> Result<Connection> {
    let path = db_path();
    let conn =
        Connection::open(&path).with_context(|| format!("Failed to open chat.db at {:?}", path))?;
    conn.pragma_update(None, "query_only", "ON")?;
    Ok(conn)
}

/// Extract plain text from NSAttributedString binary blob (attributedBody column).
/// Format: ... NSString \x01\x94\x84\x01 + <length> <text> ...
/// Length is single byte if < 0x80, otherwise multi-byte (low nibble = number of length bytes, little-endian).
fn extract_text_from_attributed_body(data: &[u8]) -> Option<String> {
    let marker = b"NSString";
    let idx = data.windows(marker.len()).position(|w| w == marker)?;
    let after = &data[idx + marker.len()..];

    // Find the '+' (0x2b) byte that precedes the length
    let plus_idx = after.iter().position(|&b| b == 0x2b)?;
    let after_plus = &after[plus_idx + 1..];
    if after_plus.is_empty() {
        return None;
    }

    let length_byte = after_plus[0];
    let (length, text_start): (usize, usize);

    if length_byte < 0x80 {
        length = length_byte as usize;
        text_start = 1;
    } else {
        // Multi-byte length: low nibble = number of bytes for length, little-endian
        let num_bytes = (length_byte & 0x0f) as usize;
        if after_plus.len() < 1 + num_bytes {
            return None;
        }
        let mut len_val: usize = 0;
        for i in 0..num_bytes {
            len_val |= (after_plus[1 + i] as usize) << (8 * i);
        }
        length = len_val;
        text_start = 1 + num_bytes;
    }

    if after_plus.len() < text_start + length {
        return None;
    }

    let text_bytes = &after_plus[text_start..text_start + length];
    let text = String::from_utf8_lossy(text_bytes).to_string();

    // Trim leading control characters (sometimes has \r, \x03, \x0c prefix)
    let trimmed = text
        .trim_start_matches(|c: char| c.is_control())
        .to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn resolve_text(text: Option<String>, attributed_body: Option<Vec<u8>>) -> Option<String> {
    // Prefer text column if it's non-empty
    if let Some(ref t) = text {
        if !t.is_empty() {
            return Some(t.clone());
        }
    }
    // Fall back to extracting from attributedBody
    if let Some(ref body) = attributed_body {
        return extract_text_from_attributed_body(body);
    }
    None
}

fn row_to_message(
    rowid: i64,
    text: String,
    date_apple: i64,
    is_from_me: bool,
    handle_id: Option<String>,
    chat_id: String,
) -> Value {
    let unix_ts = apple_ts_to_unix(date_apple);
    let (sender, sender_name) = if is_from_me {
        ("me".to_string(), None)
    } else {
        let handle = handle_id.unwrap_or_else(|| "unknown".to_string());
        let name = contacts::resolve_name(&handle);
        (handle, name)
    };
    let mut msg = json!({
        "id": rowid,
        "text": text,
        "timestamp": unix_ts,
        "is_from_me": is_from_me,
        "sender": sender,
        "chat_identifier": chat_id
    });
    if let Some(name) = sender_name {
        msg["sender_name"] = json!(name);
    }
    msg
}

/// Resolve a contact name to phone numbers via contacts search
fn resolve_name_to_phones(name: &str) -> Result<Vec<String>> {
    let results = contacts::search(name)?;
    let mut phones = Vec::new();
    if let Some(contacts_arr) = results["contacts"].as_array() {
        for contact in contacts_arr {
            if let Some(phone_arr) = contact["phones"].as_array() {
                for p in phone_arr {
                    if let Some(phone) = p.as_str() {
                        let normalized = phone
                            .chars()
                            .filter(|c| c.is_ascii_digit() || *c == '+')
                            .collect::<String>();
                        if !normalized.is_empty() {
                            phones.push(normalized);
                        }
                    }
                }
            }
        }
    }
    if phones.is_empty() {
        Err(anyhow::anyhow!("No contact found matching '{}'", name))
    } else {
        Ok(phones)
    }
}

pub fn fetch(
    participants: Vec<String>,
    chat_identifier: Option<String>,
    name: Option<String>,
    limit: Option<u32>,
    before_timestamp: Option<i64>,
    after_timestamp: Option<i64>,
) -> Result<Value> {
    let conn = open_db()?;
    let limit = limit.unwrap_or(50).min(200);

    // Build the chat filter: name lookup, chat_identifier, or participant phone numbers
    let identifiers: Vec<String> = if let Some(ref n) = name {
        resolve_name_to_phones(n)?
    } else if let Some(ref cid) = chat_identifier {
        vec![cid.clone()]
    } else if !participants.is_empty() {
        participants.clone()
    } else {
        return Err(anyhow::anyhow!(
            "Provide name, participants, or chat_identifier"
        ));
    };

    let placeholders: String = identifiers
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        "SELECT
            m.rowid,
            m.text,
            m.attributedBody,
            m.date,
            m.is_from_me,
            h.id as handle_id,
            c.chat_identifier
        FROM message m
        LEFT JOIN handle h ON m.handle_id = h.rowid
        JOIN chat_message_join cmj ON cmj.message_id = m.rowid
        JOIN chat c ON c.rowid = cmj.chat_id
        WHERE c.chat_identifier IN ({placeholders})
          AND (m.text IS NOT NULL OR m.attributedBody IS NOT NULL)
          AND (? = 0 OR m.date < ?)
          AND (? = 0 OR m.date > ?)
        ORDER BY m.date DESC
        LIMIT ?"
    );

    let mut stmt = conn.prepare(&sql)?;

    let before_apple = before_timestamp.map(unix_to_apple).unwrap_or(0);
    let has_before: i64 = if before_timestamp.is_some() { 1 } else { 0 };
    let after_apple = after_timestamp.map(unix_to_apple).unwrap_or(0);
    let has_after: i64 = if after_timestamp.is_some() { 1 } else { 0 };

    let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = identifiers
        .iter()
        .map(|p| Box::new(p.clone()) as Box<dyn rusqlite::ToSql>)
        .collect();
    param_values.push(Box::new(has_before));
    param_values.push(Box::new(before_apple));
    param_values.push(Box::new(has_after));
    param_values.push(Box::new(after_apple));
    param_values.push(Box::new(limit as i64));

    let param_refs: Vec<&dyn rusqlite::ToSql> = param_values.iter().map(|v| v.as_ref()).collect();

    let messages: Vec<Value> = stmt
        .query_map(param_refs.as_slice(), |row| {
            let rowid: i64 = row.get(0)?;
            let text: Option<String> = row.get(1)?;
            let attributed_body: Option<Vec<u8>> = row.get(2)?;
            let date_apple: i64 = row.get(3)?;
            let is_from_me: bool = row.get(4)?;
            let handle_id: Option<String> = row.get(5)?;
            let chat_id: String = row.get(6)?;
            Ok((
                rowid,
                text,
                attributed_body,
                date_apple,
                is_from_me,
                handle_id,
                chat_id,
            ))
        })?
        .filter_map(|r| r.ok())
        .filter_map(
            |(rowid, text, attributed_body, date_apple, is_from_me, handle_id, chat_id)| {
                let resolved = resolve_text(text, attributed_body)?;
                Some(row_to_message(
                    rowid, resolved, date_apple, is_from_me, handle_id, chat_id,
                ))
            },
        )
        .collect();

    let next_cursor = messages
        .last()
        .map(|m| m["timestamp"].as_i64().unwrap_or(0));

    Ok(json!({
        "messages": messages,
        "count": messages.len(),
        "next_cursor": next_cursor
    }))
}

/// Find all conversations involving handles that match the query name.
/// Returns threads with recent messages, handling multiple conversations per person.
fn search_conversations_by_name(conn: &Connection, query: &str) -> Vec<Value> {
    // Find matching handles from contacts cache
    contacts::ensure_cache_public();
    let matching_handles = contacts::find_handles_by_name(query);
    if matching_handles.is_empty() {
        return Vec::new();
    }

    // Find all chats where these handles participate
    let placeholders: String = matching_handles
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT DISTINCT c.rowid, c.chat_identifier, c.display_name
         FROM chat c
         JOIN chat_handle_join chj ON chj.chat_id = c.rowid
         JOIN handle h ON h.rowid = chj.handle_id
         WHERE h.id IN ({placeholders})
         ORDER BY c.rowid DESC"
    );

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let param_refs: Vec<&dyn rusqlite::ToSql> = matching_handles
        .iter()
        .map(|h| h as &dyn rusqlite::ToSql)
        .collect();

    let chats: Vec<(i64, String, Option<String>)> = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .ok()
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    let mut conversations = Vec::new();
    for (chat_id, chat_identifier, display_name) in chats {
        // Get participants for this chat
        let participants = get_chat_participants(conn, chat_id);

        // Resolve display name
        let resolved_name =
            if display_name.as_ref().map_or(true, |n| n.is_empty()) && participants.len() == 1 {
                contacts::resolve_name(participants[0]["handle"].as_str().unwrap_or(""))
                    .or(display_name)
            } else {
                display_name
            };

        // Get recent messages
        let recent = fetch_messages_for_chat(conn, &chat_identifier, 10).unwrap_or_default();

        // Get last message info
        let (last_message, last_timestamp, last_is_from_me) = recent
            .first()
            .map(|m| {
                (
                    m["text"].as_str().unwrap_or("").to_string(),
                    m["timestamp"].as_i64().unwrap_or(0),
                    m["is_from_me"].as_bool().unwrap_or(false),
                )
            })
            .unwrap_or_default();

        if last_timestamp == 0 {
            continue; // Skip empty conversations
        }

        conversations.push(json!({
            "chat_id": chat_id,
            "chat_identifier": chat_identifier,
            "display_name": resolved_name,
            "last_message": last_message,
            "last_timestamp": last_timestamp,
            "last_is_from_me": last_is_from_me,
            "participants": participants,
            "recent_messages": recent
        }));
    }

    // Sort by last_timestamp descending
    conversations.sort_by(|a, b| {
        let ta = a["last_timestamp"].as_i64().unwrap_or(0);
        let tb = b["last_timestamp"].as_i64().unwrap_or(0);
        tb.cmp(&ta)
    });

    conversations
}

fn get_chat_participants(conn: &Connection, chat_id: i64) -> Vec<Value> {
    let sql = "SELECT h.id FROM handle h
               JOIN chat_handle_join chj ON chj.handle_id = h.rowid
               WHERE chj.chat_id = ?1";
    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map(params![chat_id], |row| {
        let handle: String = row.get(0)?;
        Ok(handle)
    })
    .ok()
    .map(|rows| {
        rows.filter_map(|r| r.ok())
            .map(|handle| {
                let name = contacts::resolve_name(&handle);
                json!({ "handle": handle, "name": name })
            })
            .collect()
    })
    .unwrap_or_default()
}

pub fn search(query: String, limit: Option<u32>, before_timestamp: Option<i64>) -> Result<Value> {
    let conn = open_db()?;
    let limit = limit.unwrap_or(50).min(200) as usize;
    let query_lower = query.to_lowercase();
    let pattern = format!("%{}%", query);

    // Section 1: Find conversations matching the query as a contact name
    let conversations = search_conversations_by_name(&conn, &query);

    // Section 2: Text content search
    let before_apple = before_timestamp.map(unix_to_apple).unwrap_or(0);
    let has_before: i64 = if before_timestamp.is_some() { 1 } else { 0 };

    // Pass 1: text column matches
    let sql_text = "SELECT
        m.rowid, m.text, m.attributedBody, m.date, m.is_from_me,
        h.id as handle_id, c.chat_identifier
    FROM message m
    LEFT JOIN handle h ON m.handle_id = h.rowid
    JOIN chat_message_join cmj ON cmj.message_id = m.rowid
    JOIN chat c ON c.rowid = cmj.chat_id
    WHERE m.text LIKE ?1
      AND m.text IS NOT NULL AND m.text != ''
      AND (?2 = 0 OR m.date < ?3)
    ORDER BY m.date DESC
    LIMIT ?4";

    let mut stmt = conn.prepare(sql_text)?;
    let mut messages: Vec<Value> = stmt
        .query_map(
            params![pattern, has_before, before_apple, limit as i64],
            |row| {
                let rowid: i64 = row.get(0)?;
                let text: Option<String> = row.get(1)?;
                let attributed_body: Option<Vec<u8>> = row.get(2)?;
                let date_apple: i64 = row.get(3)?;
                let is_from_me: bool = row.get(4)?;
                let handle_id: Option<String> = row.get(5)?;
                let chat_id: String = row.get(6)?;
                Ok((
                    rowid,
                    text,
                    attributed_body,
                    date_apple,
                    is_from_me,
                    handle_id,
                    chat_id,
                ))
            },
        )?
        .filter_map(|r| r.ok())
        .filter_map(
            |(rowid, text, attributed_body, date_apple, is_from_me, handle_id, chat_id)| {
                let resolved = resolve_text(text, attributed_body)?;
                Some(row_to_message(
                    rowid, resolved, date_apple, is_from_me, handle_id, chat_id,
                ))
            },
        )
        .collect();

    // Pass 2: attributedBody messages (text IS NULL)
    let remaining = limit.saturating_sub(messages.len());
    if remaining > 0 {
        let overfetch = ((remaining * 50) as i64).max(500);
        let sql_body = "SELECT
            m.rowid, m.text, m.attributedBody, m.date, m.is_from_me,
            h.id as handle_id, c.chat_identifier
        FROM message m
        LEFT JOIN handle h ON m.handle_id = h.rowid
        JOIN chat_message_join cmj ON cmj.message_id = m.rowid
        JOIN chat c ON c.rowid = cmj.chat_id
        WHERE m.text IS NULL AND m.attributedBody IS NOT NULL
          AND (?1 = 0 OR m.date < ?2)
        ORDER BY m.date DESC
        LIMIT ?3";

        let mut stmt2 = conn.prepare(sql_body)?;
        let body_matches: Vec<Value> = stmt2
            .query_map(params![has_before, before_apple, overfetch], |row| {
                let rowid: i64 = row.get(0)?;
                let text: Option<String> = row.get(1)?;
                let attributed_body: Option<Vec<u8>> = row.get(2)?;
                let date_apple: i64 = row.get(3)?;
                let is_from_me: bool = row.get(4)?;
                let handle_id: Option<String> = row.get(5)?;
                let chat_id: String = row.get(6)?;
                Ok((
                    rowid,
                    text,
                    attributed_body,
                    date_apple,
                    is_from_me,
                    handle_id,
                    chat_id,
                ))
            })?
            .filter_map(|r| r.ok())
            .filter_map(
                |(rowid, text, attributed_body, date_apple, is_from_me, handle_id, chat_id)| {
                    let resolved = resolve_text(text, attributed_body)?;
                    if !resolved.to_lowercase().contains(&query_lower) {
                        return None;
                    }
                    Some(row_to_message(
                        rowid, resolved, date_apple, is_from_me, handle_id, chat_id,
                    ))
                },
            )
            .take(remaining)
            .collect();

        messages.extend(body_matches);
    }

    // Sort merged results by timestamp descending
    messages.sort_by(|a, b| {
        let ta = a["timestamp"].as_i64().unwrap_or(0);
        let tb = b["timestamp"].as_i64().unwrap_or(0);
        tb.cmp(&ta)
    });
    messages.truncate(limit);

    let next_cursor = messages
        .last()
        .map(|m| m["timestamp"].as_i64().unwrap_or(0));

    Ok(json!({
        "conversations": conversations,
        "conversations_count": conversations.len(),
        "messages": messages,
        "count": messages.len(),
        "query": query,
        "next_cursor": next_cursor
    }))
}

fn fetch_messages_for_chat(
    conn: &Connection,
    chat_identifier: &str,
    limit: u32,
) -> Result<Vec<Value>> {
    let sql = "SELECT m.rowid, m.text, m.attributedBody, m.date, m.is_from_me,
                      h.id as handle_id, c.chat_identifier
               FROM message m
               LEFT JOIN handle h ON m.handle_id = h.rowid
               JOIN chat_message_join cmj ON cmj.message_id = m.rowid
               JOIN chat c ON c.rowid = cmj.chat_id
               WHERE c.chat_identifier = ?1
                 AND (m.text IS NOT NULL OR m.attributedBody IS NOT NULL)
               ORDER BY m.date DESC
               LIMIT ?2";
    let mut stmt = conn.prepare(sql)?;
    let messages = stmt
        .query_map(params![chat_identifier, limit as i64], |row| {
            let rowid: i64 = row.get(0)?;
            let text: Option<String> = row.get(1)?;
            let attributed_body: Option<Vec<u8>> = row.get(2)?;
            let date_apple: i64 = row.get(3)?;
            let is_from_me: bool = row.get(4)?;
            let handle_id: Option<String> = row.get(5)?;
            let chat_id: String = row.get(6)?;
            Ok((
                rowid,
                text,
                attributed_body,
                date_apple,
                is_from_me,
                handle_id,
                chat_id,
            ))
        })?
        .filter_map(|r| r.ok())
        .filter_map(
            |(rowid, text, attributed_body, date_apple, is_from_me, handle_id, chat_id)| {
                let resolved = resolve_text(text, attributed_body)?;
                Some(row_to_message(
                    rowid, resolved, date_apple, is_from_me, handle_id, chat_id,
                ))
            },
        )
        .collect();
    Ok(messages)
}

pub fn threads(limit: Option<u32>, offset: Option<u32>) -> Result<Value> {
    let conn = open_db()?;
    let limit = limit.unwrap_or(5).min(100);
    let offset = offset.unwrap_or(0);
    let include_messages = limit <= 5;

    let sql = "WITH latest_msg AS (
        SELECT
            cmj.chat_id,
            MAX(m.rowid) as msg_id
        FROM chat_message_join cmj
        JOIN message m ON m.rowid = cmj.message_id
        WHERE m.text IS NOT NULL AND m.text != ''
           OR m.attributedBody IS NOT NULL
        GROUP BY cmj.chat_id
    )
    SELECT
        c.rowid as chat_id,
        c.chat_identifier,
        c.display_name,
        m.text,
        m.attributedBody,
        m.date as last_date,
        m.is_from_me,
        GROUP_CONCAT(DISTINCT h.id) as participants
    FROM latest_msg lm
    JOIN chat c ON c.rowid = lm.chat_id
    JOIN message m ON m.rowid = lm.msg_id
    LEFT JOIN chat_handle_join chj ON chj.chat_id = c.rowid
    LEFT JOIN handle h ON h.rowid = chj.handle_id
    GROUP BY c.rowid
    ORDER BY m.date DESC
    LIMIT ?1
    OFFSET ?2";

    let mut stmt = conn.prepare(sql)?;
    let mut threads: Vec<Value> = stmt
        .query_map(params![limit as i64, offset as i64], |row| {
            let chat_id: i64 = row.get(0)?;
            let chat_identifier: String = row.get(1)?;
            let display_name: Option<String> = row.get(2)?;
            let text: Option<String> = row.get(3)?;
            let attributed_body: Option<Vec<u8>> = row.get(4)?;
            let last_date_apple: i64 = row.get(5)?;
            let is_from_me: bool = row.get(6)?;
            let participants: Option<String> = row.get(7)?;
            Ok((
                chat_id,
                chat_identifier,
                display_name,
                text,
                attributed_body,
                last_date_apple,
                is_from_me,
                participants,
            ))
        })?
        .filter_map(|r| r.ok())
        .map(
            |(
                chat_id,
                chat_identifier,
                display_name,
                text,
                attributed_body,
                last_date_apple,
                is_from_me,
                participants,
            )| {
                let unix_ts = apple_ts_to_unix(last_date_apple);
                let last_message = resolve_text(text, attributed_body)
                    .unwrap_or_else(|| "(attachment)".to_string());
                let participant_list: Vec<&str> = participants
                    .as_deref()
                    .unwrap_or("")
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .collect();
                // Resolve contact names for participants
                let participant_names: Vec<Value> = participant_list
                    .iter()
                    .map(|p| {
                        let name = contacts::resolve_name(p);
                        json!({ "handle": p, "name": name })
                    })
                    .collect();
                // For 1-on-1 chats without a display_name, try to resolve the contact name
                let resolved_name = if display_name.as_ref().map_or(true, |n| n.is_empty())
                    && participant_list.len() == 1
                {
                    contacts::resolve_name(participant_list[0])
                } else {
                    display_name.clone()
                };
                json!({
                    "chat_id": chat_id,
                    "chat_identifier": chat_identifier,
                    "display_name": resolved_name,
                    "last_message": last_message,
                    "last_timestamp": unix_ts,
                    "last_is_from_me": is_from_me,
                    "participants": participant_names
                })
            },
        )
        .collect();

    // When showing <= 5 threads, include 10 recent messages per thread
    if include_messages {
        for thread in threads.iter_mut() {
            let chat_id = thread["chat_identifier"].as_str().unwrap_or("").to_string();
            if !chat_id.is_empty() {
                if let Ok(msgs) = fetch_messages_for_chat(&conn, &chat_id, 10) {
                    if let Some(obj) = thread.as_object_mut() {
                        obj.insert("recent_messages".to_string(), json!(msgs));
                    }
                }
            }
        }
    }

    Ok(json!({
        "threads": threads,
        "count": threads.len(),
        "offset": offset,
        "next_offset": offset + limit
    }))
}
