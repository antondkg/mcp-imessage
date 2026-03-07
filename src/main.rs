mod contacts;
mod messages;
mod send;

use anyhow::Result;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    ServiceExt,
    transport::stdio,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MessagesFetchParams {
    #[schemars(description = "Phone numbers in E.164 format (e.g. +16317457857) to filter by. For group chats, leave empty and use chat_identifier instead.")]
    participants: Option<Vec<String>>,
    #[schemars(description = "Chat identifier for group chats (e.g. chat696614010123836136). Get this from messages_threads. Use this OR participants, not both.")]
    chat_identifier: Option<String>,
    #[schemars(description = "Max number of messages to return (default 50, max 200)")]
    limit: Option<u32>,
    #[schemars(description = "Pagination cursor: only return messages before this unix timestamp (use next_cursor from previous response)")]
    before_timestamp: Option<i64>,
    #[schemars(description = "Only return messages after this unix timestamp")]
    after_timestamp: Option<i64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MessagesSendParams {
    #[schemars(description = "Recipient phone number (E.164) or Apple ID email. For group chats, leave empty and use chat_identifier.")]
    recipient: Option<String>,
    #[schemars(description = "Chat identifier for group chats (from messages_threads). Use this OR recipient, not both.")]
    chat_identifier: Option<String>,
    #[schemars(description = "Message text to send. Optional if sending a file only.")]
    text: Option<String>,
    #[schemars(description = "Absolute path to a file/image to send (e.g. /Users/you/Desktop/photo.jpg). Can be combined with text.")]
    file_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MessagesSearchParams {
    #[schemars(description = "Text to search for in messages")]
    query: String,
    #[schemars(description = "Max number of results (default 50, max 200)")]
    limit: Option<u32>,
    #[schemars(description = "Pagination cursor: only return messages before this unix timestamp")]
    before_timestamp: Option<i64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MessagesThreadsParams {
    #[schemars(description = "Max number of threads to return (default 20, max 100)")]
    limit: Option<u32>,
    #[schemars(description = "Pagination offset (default 0)")]
    offset: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ContactsSearchParams {
    #[schemars(description = "Name, phone, or email to search for")]
    query: String,
}

#[derive(Debug, Clone)]
struct IMessageServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl IMessageServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Fetch iMessages from a conversation. Filter by participant phone numbers (E.164 format). Returns messages ordered newest first with cursor-based pagination.")]
    fn messages_fetch(
        &self,
        Parameters(MessagesFetchParams {
            participants,
            chat_identifier,
            limit,
            before_timestamp,
            after_timestamp,
        }): Parameters<MessagesFetchParams>,
    ) -> String {
        match messages::fetch(participants.unwrap_or_default(), chat_identifier, limit, before_timestamp, after_timestamp) {
            Ok(v) => v.to_string(),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Send an iMessage or SMS. For 1-on-1: provide recipient (phone E.164 or email). For group chats: provide chat_identifier from messages_threads. Can send text, files/images, or both. Messages.app must be running.")]
    fn messages_send(
        &self,
        Parameters(MessagesSendParams { recipient, chat_identifier, text, file_path }): Parameters<MessagesSendParams>,
    ) -> String {
        match send::send_message(recipient.as_deref(), chat_identifier.as_deref(), text.as_deref(), file_path.as_deref()) {
            Ok(v) => v.to_string(),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Full-text search across all iMessages. Returns messages containing the query text, newest first, with cursor-based pagination.")]
    fn messages_search(
        &self,
        Parameters(MessagesSearchParams { query, limit, before_timestamp }): Parameters<MessagesSearchParams>,
    ) -> String {
        match messages::search(query, limit, before_timestamp) {
            Ok(v) => v.to_string(),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "List recent iMessage conversation threads with the last message preview and participant list. Supports offset-based pagination.")]
    fn messages_threads(
        &self,
        Parameters(MessagesThreadsParams { limit, offset }): Parameters<MessagesThreadsParams>,
    ) -> String {
        match messages::threads(limit, offset) {
            Ok(v) => v.to_string(),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Search iCloud contacts by name, phone number, or email address.")]
    fn contacts_search(
        &self,
        Parameters(ContactsSearchParams { query }): Parameters<ContactsSearchParams>,
    ) -> String {
        match contacts::search(&query) {
            Ok(v) => v.to_string(),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Return the user's own contact card (name, phone numbers, email addresses) from Contacts.app.")]
    fn contacts_me(&self) -> String {
        match contacts::me() {
            Ok(v) => v.to_string(),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }
}

#[tool_handler]
impl ServerHandler for IMessageServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "iMessage MCP server -- read/send messages and search contacts via macOS APIs"
                    .to_string(),
            )
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting mcp-imessage server");

    let service = IMessageServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| tracing::error!("Server error: {:?}", e))?;

    service.waiting().await?;
    Ok(())
}
