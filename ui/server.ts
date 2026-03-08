import { createMcpApp } from "@json-render/mcp";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { catalog } from "./src/catalog.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

function loadHtml(): string {
  const htmlPath = path.join(__dirname, "dist", "index.html");
  if (!fs.existsSync(htmlPath)) {
    throw new Error(
      `Built HTML not found at ${htmlPath}. Run 'npm run build' first.`,
    );
  }
  return fs.readFileSync(htmlPath, "utf-8");
}

const TOOL_DESCRIPTION = `Render iMessage data as a native iOS-style UI. Use after calling iMessage MCP tools (messages_fetch, messages_threads, messages_search, contacts_search, contacts_me).

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
  - Pass the original search query as the query prop for highlighting`;

async function main() {
  const html = loadHtml();
  const server = await createMcpApp({
    name: "iMessage UI",
    version: "1.0.0",
    catalog,
    html,
    tool: {
      name: "render-imessage-ui",
      title: "Render iMessage UI",
      description: TOOL_DESCRIPTION,
    },
  });
  await server.connect(new StdioServerTransport());
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
