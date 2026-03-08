mod contacts;
mod messages;
mod send;

use std::borrow::Cow;
use std::sync::Arc;

use anyhow::Result;
use rmcp::{
    ErrorData, RoleServer,
    ServerHandler,
    handler::server::router::tool::{ToolRoute, ToolRouter},
    model::{
        Annotated, CallToolResult, Content, ListResourcesResult, Meta, RawResource,
        ReadResourceRequestParams, ReadResourceResult, ResourceContents,
        ServerCapabilities, ServerInfo, Tool,
    },
    tool_handler, tool_router,
    service::RequestContext,
    ServiceExt,
    transport::stdio,
};
use serde_json::json;

/// Embedded UI HTML (built from ui/dist/index.html)
const UI_HTML: &str = include_str!("../ui/dist/index.html");
const UI_RESOURCE_URI: &str = "ui://render-imessage-ui/view.html";

#[derive(Debug, Clone)]
struct IMessageServer {
    tool_router: ToolRouter<Self>,
}

fn make_ui_meta() -> Meta {
    let mut m = Meta::new();
    m.0.insert("ui".to_string(), json!({ "resourceUri": UI_RESOURCE_URI }));
    m
}

fn make_schema(props: serde_json::Value) -> Arc<serde_json::Map<String, serde_json::Value>> {
    let mut schema = json!({ "type": "object" });
    schema["properties"] = props;
    Arc::new(schema.as_object().unwrap().clone())
}

fn make_threads_tool_route() -> ToolRoute<IMessageServer> {
    let schema = make_schema(json!({
        "limit": {
            "type": "integer",
            "description": "Max number of threads to return (default 5, max 100). When <= 5, includes 10 recent messages per thread for inline viewing."
        },
        "offset": {
            "type": "integer",
            "description": "Pagination offset (default 0)"
        }
    }));

    let mut tool_def = Tool::new(
        "messages_threads",
        Cow::Borrowed("List recent iMessage conversation threads. Returns 5 most recent threads by default, each with 10 recent messages for inline viewing. Use default unless user asks for more. When limit > 5, messages are omitted to reduce payload."),
        schema,
    );
    tool_def.meta = Some(make_ui_meta());

    ToolRoute::new_dyn(tool_def, |context| {
        Box::pin(async move {
            let args = context.arguments.unwrap_or_default();
            let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as u32);
            let offset = args.get("offset").and_then(|v| v.as_u64()).map(|v| v as u32);
            let result = match messages::threads(limit, offset) {
                Ok(v) => v.to_string(),
                Err(e) => format!("{{\"error\": \"{}\"}}", e),
            };
            Ok(CallToolResult::success(vec![Content::text(result)]))
        })
    })
}

fn make_fetch_tool_route() -> ToolRoute<IMessageServer> {
    let schema = make_schema(json!({
        "participants": {
            "type": "array",
            "items": { "type": "string" },
            "description": "Phone numbers in E.164 format (e.g. +16317457857) to filter by. For group chats, leave empty and use chat_identifier instead."
        },
        "chat_identifier": {
            "type": "string",
            "description": "Chat identifier for group chats (e.g. chat696614010123836136). Get this from messages_threads. Use this OR participants, not both."
        },
        "limit": {
            "type": "integer",
            "description": "Max number of messages to return (default 50, max 200)"
        },
        "before_timestamp": {
            "type": "integer",
            "description": "Pagination cursor: only return messages before this unix timestamp (use next_cursor from previous response)"
        },
        "after_timestamp": {
            "type": "integer",
            "description": "Only return messages after this unix timestamp"
        }
    }));

    let mut tool_def = Tool::new(
        "messages_fetch",
        Cow::Borrowed("Fetch iMessages from a conversation. Filter by participant phone numbers (E.164 format) or chat_identifier. Returns messages ordered newest first with cursor-based pagination."),
        schema,
    );
    tool_def.meta = Some(make_ui_meta());

    ToolRoute::new_dyn(tool_def, |context| {
        Box::pin(async move {
            let args = context.arguments.unwrap_or_default();
            let participants: Vec<String> = args.get("participants")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let chat_identifier = args.get("chat_identifier").and_then(|v| v.as_str()).map(String::from);
            let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as u32);
            let before_timestamp = args.get("before_timestamp").and_then(|v| v.as_i64());
            let after_timestamp = args.get("after_timestamp").and_then(|v| v.as_i64());
            let result = match messages::fetch(participants, chat_identifier, limit, before_timestamp, after_timestamp) {
                Ok(v) => v.to_string(),
                Err(e) => format!("{{\"error\": \"{}\"}}", e),
            };
            Ok(CallToolResult::success(vec![Content::text(result)]))
        })
    })
}

fn make_send_tool_route() -> ToolRoute<IMessageServer> {
    let schema = make_schema(json!({
        "recipient": {
            "type": "string",
            "description": "Recipient phone number (E.164) or Apple ID email. For group chats, leave empty and use chat_identifier."
        },
        "chat_identifier": {
            "type": "string",
            "description": "Chat identifier for group chats (from messages_threads). Use this OR recipient, not both."
        },
        "text": {
            "type": "string",
            "description": "Message text to send. Optional if sending a file only."
        },
        "file_path": {
            "type": "string",
            "description": "Absolute path to a file/image to send (e.g. /Users/you/Desktop/photo.jpg). Can be combined with text."
        }
    }));

    let mut tool_def = Tool::new(
        "messages_send",
        Cow::Borrowed("Send an iMessage or SMS. For 1-on-1: provide recipient (phone E.164 or email). For group chats: provide chat_identifier from messages_threads. Can send text, files/images, or both. Messages.app must be running."),
        schema,
    );
    tool_def.meta = Some(make_ui_meta());

    ToolRoute::new_dyn(tool_def, |context| {
        Box::pin(async move {
            let args = context.arguments.unwrap_or_default();
            let recipient = args.get("recipient").and_then(|v| v.as_str()).map(String::from);
            let chat_identifier = args.get("chat_identifier").and_then(|v| v.as_str()).map(String::from);
            let text = args.get("text").and_then(|v| v.as_str()).map(String::from);
            let file_path = args.get("file_path").and_then(|v| v.as_str()).map(String::from);

            let send_result = send::send_message(
                recipient.as_deref(),
                chat_identifier.as_deref(),
                text.as_deref(),
                file_path.as_deref(),
            );

            let result = match send_result {
                Ok(mut v) => {
                    // After successful send, fetch recent messages for the conversation
                    let recent = if let Some(ref recip) = recipient {
                        messages::fetch(vec![recip.clone()], None, Some(10), None, None).ok()
                    } else if let Some(ref cid) = chat_identifier {
                        messages::fetch(vec![], Some(cid.clone()), Some(10), None, None).ok()
                    } else {
                        None
                    };
                    if let Some(msgs) = recent {
                        if let Some(obj) = v.as_object_mut() {
                            obj.insert("recent_messages".to_string(), msgs["messages"].clone());
                        }
                    }
                    v.to_string()
                }
                Err(e) => format!("{{\"error\": \"{}\"}}", e),
            };
            Ok(CallToolResult::success(vec![Content::text(result)]))
        })
    })
}

fn make_search_tool_route() -> ToolRoute<IMessageServer> {
    let schema_val = json!({
        "type": "object",
        "required": ["query"],
        "properties": {
            "query": {
                "type": "string",
                "description": "Text to search for in messages"
            },
            "limit": {
                "type": "integer",
                "description": "Max number of results (default 50, max 200)"
            },
            "before_timestamp": {
                "type": "integer",
                "description": "Pagination cursor: only return messages before this unix timestamp"
            }
        }
    });
    let schema = Arc::new(schema_val.as_object().unwrap().clone());

    let mut tool_def = Tool::new(
        "messages_search",
        Cow::Borrowed("Full-text search across all iMessages. Returns messages containing the query text, newest first, with cursor-based pagination."),
        schema,
    );
    tool_def.meta = Some(make_ui_meta());

    ToolRoute::new_dyn(tool_def, |context| {
        Box::pin(async move {
            let args = context.arguments.unwrap_or_default();
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as u32);
            let before_timestamp = args.get("before_timestamp").and_then(|v| v.as_i64());
            let result = match messages::search(query, limit, before_timestamp) {
                Ok(v) => v.to_string(),
                Err(e) => format!("{{\"error\": \"{}\"}}", e),
            };
            Ok(CallToolResult::success(vec![Content::text(result)]))
        })
    })
}

fn make_contacts_search_tool_route() -> ToolRoute<IMessageServer> {
    let schema_val = json!({
        "type": "object",
        "required": ["query"],
        "properties": {
            "query": {
                "type": "string",
                "description": "Name, phone, or email to search for"
            }
        }
    });
    let schema = Arc::new(schema_val.as_object().unwrap().clone());

    let mut tool_def = Tool::new(
        "contacts_search",
        Cow::Borrowed("Search iCloud contacts by name, phone number, or email address. Returns a list of matching contacts with their phone numbers and emails."),
        schema,
    );
    tool_def.meta = Some(make_ui_meta());

    ToolRoute::new_dyn(tool_def, |context| {
        Box::pin(async move {
            let args = context.arguments.unwrap_or_default();
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let result = match contacts::search(&query) {
                Ok(v) => v.to_string(),
                Err(e) => format!("{{\"error\": \"{}\"}}", e),
            };
            Ok(CallToolResult::success(vec![Content::text(result)]))
        })
    })
}

fn make_contacts_me_tool_route() -> ToolRoute<IMessageServer> {
    let schema = Arc::new(json!({ "type": "object" }).as_object().unwrap().clone());

    let mut tool_def = Tool::new(
        "contacts_me",
        Cow::Borrowed("Return the user's own contact card (name, phone numbers, email addresses) from Contacts.app."),
        schema,
    );
    tool_def.meta = Some(make_ui_meta());

    ToolRoute::new_dyn(tool_def, |_context| {
        Box::pin(async move {
            let result = match contacts::me() {
                Ok(v) => v.to_string(),
                Err(e) => format!("{{\"error\": \"{}\"}}", e),
            };
            Ok(CallToolResult::success(vec![Content::text(result)]))
        })
    })
}

#[tool_router]
impl IMessageServer {
    fn new() -> Self {
        let mut router = Self::tool_router();
        router.add_route(make_threads_tool_route());
        router.add_route(make_fetch_tool_route());
        router.add_route(make_send_tool_route());
        router.add_route(make_search_tool_route());
        router.add_route(make_contacts_search_tool_route());
        router.add_route(make_contacts_me_tool_route());

        Self {
            tool_router: router,
        }
    }
}

fn ui_csp_meta() -> Meta {
    let mut meta = Meta::new();
    meta.0.insert(
        "ui".to_string(),
        json!({
            "csp": {
                "resourceDomains": ["https:"],
                "connectDomains": ["https:"]
            }
        }),
    );
    meta
}

#[tool_handler]
impl ServerHandler for IMessageServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_instructions(
            "iMessage MCP server -- read/send messages and search contacts via macOS APIs"
                .to_string(),
        )
    }

    fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, ErrorData>> + Send + '_ {
        let resource = Annotated::new(
            RawResource {
                uri: UI_RESOURCE_URI.to_string(),
                name: "iMessage UI".to_string(),
                title: Some("iMessage UI".to_string()),
                description: Some("Native iOS-style iMessage UI renderer".to_string()),
                mime_type: Some("text/html;profile=mcp-app".to_string()),
                size: None,
                icons: None,
                meta: Some(ui_csp_meta()),
            },
            None,
        );

        std::future::ready(Ok(ListResourcesResult::with_all_items(vec![resource])))
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, ErrorData>> + Send + '_ {
        let result = if request.uri == UI_RESOURCE_URI {
            Ok(ReadResourceResult::new(vec![
                ResourceContents::text(UI_HTML, UI_RESOURCE_URI)
                    .with_mime_type("text/html;profile=mcp-app")
                    .with_meta(ui_csp_meta()),
            ]))
        } else {
            Err(ErrorData::resource_not_found(
                "resource_not_found",
                Some(json!({ "uri": request.uri })),
            ))
        };

        std::future::ready(result)
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
