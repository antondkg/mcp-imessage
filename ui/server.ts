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

const TOOL_DESCRIPTION = `Render an interactive UI for iMessage data. Use this tool to display conversations, thread lists, contact cards, and search results from the iMessage MCP server as rich visual components.

WHEN TO USE THIS TOOL:
- After fetching messages via messages_fetch, render them as a conversation view
- After listing threads via messages_threads, render them as a thread list
- After searching messages via messages_search, render search results
- After looking up contacts via contacts_search or contacts_me, render contact cards

RENDERING PATTERNS:

1. CONVERSATION VIEW (after messages_fetch):
Use a Card as container, with Stacks of message bubbles. For each message:
- Use a Stack (direction="horizontal", gap=2) with Avatar + Card
- Sent messages (is_from_me=true): align right, use primary-colored Card
- Received messages: align left, use secondary Card
- Show sender_name or "You", timestamp as muted Text

2. THREAD LIST (after messages_threads):
Use a Stack of Cards, one per thread. Each Card contains:
- Heading with display_name
- Text (variant="muted") with last message preview
- Badge with timestamp
- Text (variant="caption") with participant count

3. CONTACT CARD (after contacts_search/contacts_me):
Use a Card with Avatar + Stack of contact details (name, phones, emails)

4. SEARCH RESULTS (after messages_search):
Use a Stack of Cards showing matching messages with highlighted context`;

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
