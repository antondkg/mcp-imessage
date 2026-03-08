mod contacts;
mod messages;
mod send;

use std::borrow::Cow;
use std::sync::Arc;

use anyhow::Result;
use rmcp::{
    ErrorData, RoleServer,
    ServerHandler,
    handler::server::{
        router::tool::{ToolRoute, ToolRouter},
        wrapper::Parameters,
    },
    model::{
        Annotated, CallToolResult, Content, ListResourcesResult, Meta, RawResource,
        ReadResourceRequestParams, ReadResourceResult, ResourceContents,
        ServerCapabilities, ServerInfo, Tool,
    },
    schemars, tool, tool_handler, tool_router,
    service::RequestContext,
    ServiceExt,
    transport::stdio,
};
use serde::Deserialize;
use serde_json::json;

/// Embedded UI HTML (built from ui/dist/index.html)
const UI_HTML: &str = include_str!("../ui/dist/index.html");
const UI_RESOURCE_URI: &str = "ui://render-imessage-ui/view.html";

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

fn render_tool_description() -> Cow<'static, str> {
    Cow::Borrowed(r#"Render iMessage data as a native iOS-style UI. Use after calling iMessage MCP tools (messages_fetch, messages_threads, messages_search, contacts_search, contacts_me).

CUSTOM iMESSAGE COMPONENTS (prefer these over generic shadcn):

1. MessageBubble - Native iMessage chat bubble
   Props: text (string), sender (string), time (string), isMe (boolean), showAvatar (boolean|null)
   - isMe=true: blue bubble, right-aligned (sent messages)
   - isMe=false: gray bubble, left-aligned (received messages)
   - showAvatar=true: shows sender initials on received messages

2. ThreadRow - Conversation list row
   Props: name (string), preview (string), time (string), unread (boolean|null)
   - Shows avatar initials, name, last message preview, time
   - unread=true: bold name, blue time, blue dot indicator

3. ContactCard - iOS contact card
   Props: name (string), phones (string|null), emails (string|null)
   - Centered avatar, name, phone/email in iOS blue
   - Comma-separate multiple phones or emails

4. SearchResult - Search result row
   Props: sender (string), text (string), time (string), query (string|null)
   - Shows avatar, sender name, message text with query highlighted, time

RENDERING PATTERNS:

CONVERSATION (after messages_fetch):
Root: Stack(direction=vertical, gap=sm)
  Children: one MessageBubble per message
  - Set isMe=true for is_from_me messages, false otherwise
  - Set showAvatar=true on first message or when sender changes
  - Use sender_name for sender, format timestamp for time

THREAD LIST (after messages_threads):
Root: Stack(direction=vertical, gap=none)
  Children: one ThreadRow per thread
  - Use display_name for name, last_message for preview

CONTACT (after contacts_search/contacts_me):
Root: ContactCard with name, phones, emails from the contact data

SEARCH (after messages_search):
Root: Stack(direction=vertical, gap=none)
  Children: one SearchResult per match
  - Pass the original search query as the query prop for highlighting"#)
}

fn render_tool_input_schema() -> Arc<serde_json::Map<String, serde_json::Value>> {
    let schema = json!({
        "type": "object",
        "required": ["spec"],
        "properties": {
            "spec": {
                "type": "object",
                "description": "JSON UI spec with root and elements",
                "required": ["root", "elements"],
                "properties": {
                    "root": { "type": "string" },
                    "elements": {
                        "type": "object",
                        "additionalProperties": {
                            "type": "object",
                            "required": ["type", "props", "children", "visible"],
                            "properties": {
                                "type": {
                                    "type": "string",
                                    "enum": [
                                        "Card", "Stack", "Grid", "Separator", "Tabs",
                                        "Accordion", "Collapsible", "Dialog", "Drawer",
                                        "Carousel", "Table", "Heading", "Text", "Image",
                                        "Avatar", "Badge", "Alert", "Progress", "Skeleton",
                                        "Spinner", "Tooltip", "Popover", "Input", "Textarea",
                                        "Select", "Checkbox", "Radio", "Switch", "Slider",
                                        "Button", "Link", "DropdownMenu", "Toggle",
                                        "ToggleGroup", "ButtonGroup", "Pagination",
                                        "MessageBubble", "ThreadRow", "ContactCard",
                                        "SearchResult"
                                    ]
                                },
                                "props": { "type": "object", "additionalProperties": {} },
                                "children": { "type": "array", "items": { "type": "string" } },
                                "visible": {}
                            }
                        }
                    }
                }
            }
        }
    });
    let obj = schema.as_object().unwrap().clone();
    Arc::new(obj)
}

fn make_render_tool_route() -> ToolRoute<IMessageServer> {
    let mut tool_meta = Meta::new();
    tool_meta.0.insert(
        "ui".to_string(),
        json!({ "resourceUri": UI_RESOURCE_URI }),
    );

    let mut tool_def = Tool::new(
        "render_imessage_ui",
        render_tool_description(),
        render_tool_input_schema(),
    );
    tool_def.meta = Some(tool_meta);

    ToolRoute::new_dyn(tool_def, |context| {
        Box::pin(async move {
            let args = context.arguments.unwrap_or_default();
            let spec = args.get("spec").cloned().unwrap_or(serde_json::Value::Null);
            let text = serde_json::to_string(&spec).unwrap_or_default();
            Ok(CallToolResult::success(vec![Content::text(text)]))
        })
    })
}

#[tool_router]
impl IMessageServer {
    fn new() -> Self {
        let mut router = Self::tool_router();
        router.add_route(make_render_tool_route());

        Self {
            tool_router: router,
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
