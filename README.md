<div align="center">
  <h1>mcp-imessage</h1>
  <p>A macOS MCP server for reading, searching, drafting, and sending iMessages with a native-style embedded UI.</p>
  <p>
    <img alt="macOS" src="https://img.shields.io/badge/macOS-Only-111827?style=flat-square&logo=apple&logoColor=white">
    <img alt="Rust" src="https://img.shields.io/badge/Built%20with-Rust-b7410e?style=flat-square&logo=rust&logoColor=white">
    <img alt="MCP" src="https://img.shields.io/badge/Protocol-MCP-2563eb?style=flat-square">
    <img alt="License" src="https://img.shields.io/badge/License-MIT-16a34a?style=flat-square">
  </p>
</div>

`mcp-imessage` can:

- read recent iMessage threads
- fetch full conversations by name, number, or group chat
- search messages and conversations
- look up contacts from Contacts.app
- draft messages in the UI before sending
- send iMessages, SMS messages, and file attachments
- render threads in a native-style embedded UI with a live-updating composer

It is built in Rust for the server layer, with a small React/Vite app bundled into the binary for the UI.

UI stack: [shadcn/ui](https://github.com/shadcn-ui/ui) and [json-render](https://github.com/ImVexed/json-render).

## Quick start

### Install with Homebrew

```bash
brew tap antondkg/homebrew-tap
brew install mcp-imessage
```

Then add this to your MCP client config:

```json
{
  "mcpServers": {
    "imessage": {
      "command": "/opt/homebrew/opt/mcp-imessage/bin/mcp-imessage",
      "env": {
        "MCP_IMESSAGE_ENABLE_SEND": "1"
      }
    }
  }
}
```

Common MCP config files:

- Claude Desktop: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Codex / local agent config: `~/.agents/mcp.json`
- Generic MCP clients: whatever config file your client uses for `mcpServers`

Homebrew binary paths:

- Apple Silicon: `/opt/homebrew/opt/mcp-imessage/bin/mcp-imessage`
- Intel Macs: `/usr/local/opt/mcp-imessage/bin/mcp-imessage`

If you do not want send support yet, remove the `env` block and keep only the `command`.

### Build from source

```bash
git clone https://github.com/antondkg/mcp-imessage.git
cd mcp-imessage
cargo build --release
```

The Rust build installs UI dependencies with `npm ci` and bundles the frontend into `ui/dist/index.html`.

Then point your MCP client at:

```text
/absolute/path/to/mcp-imessage/target/release/mcp-imessage
```

## Screenshots

![Thread list UI](assets/screenshots/threads-2026-03-14.png)
![Conversation view UI](assets/screenshots/conversation-2026-03-14.png)
![Search and contact UI](assets/screenshots/search-2026-03-14.png)

## Requirements

- macOS
- Node.js 20+
- Rust stable
- Messages.app signed in to iMessage
- permission to access Messages data, Contacts, and Automation access for Messages.app when sending

## Full Disk Access

macOS protects `~/Library/Messages/chat.db`, so the `mcp-imessage` server process itself needs Full Disk Access to read message history reliably.

1. Open `System Settings`.
2. Go to `Privacy & Security` -> `Full Disk Access`.
3. Add and enable `mcp-imessage`.
4. Grant access to the actual `mcp-imessage` binary, not just the host app.
5. If your MCP host still inherits the host app's privacy boundary, also grant Full Disk Access to that host app.

![Full Disk Access setup for mcp-imessage](assets/screenshots/full-disk-access-2026-03-14.png)

After enabling Full Disk Access, fully restart the host app before testing again.

## Sending

`messages_send` is off by default. Set `MCP_IMESSAGE_ENABLE_SEND=1` in the MCP config for the `mcp-imessage` server entry to enable it.

If you run `mcp-imessage` directly in a terminal instead of through an MCP client, then you can launch it like this:

```bash
MCP_IMESSAGE_ENABLE_SEND=1 /absolute/path/to/mcp-imessage
```

- Do not put contact names into `messages_send` or `messages_draft`.
- Use `contacts_search` first, then pass the phone number in E.164 format like `+14155550123`.
- Group chats should use `chat_identifier` from `messages_threads`.

## For Agents

If you are helping a user install this in an MCP client:

1. Build or install `mcp-imessage`.
2. Add the `mcp-imessage` binary to the client's MCP config.
3. If the user wants send support, add an `env` block with `MCP_IMESSAGE_ENABLE_SEND=1` on the `mcp-imessage` server entry.
4. Make sure Full Disk Access is granted to the `mcp-imessage` binary itself.
5. Restart the MCP host after changing config or permissions.

Claude Desktop example:

File:
`~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "imessage": {
      "command": "/absolute/path/to/mcp-imessage",
      "env": {
        "MCP_IMESSAGE_ENABLE_SEND": "1"
      }
    }
  }
}
```

Codex / local agent example:

File:
`~/.agents/mcp.json`

```json
{
  "mcpServers": {
    "imessage": {
      "command": "/absolute/path/to/mcp-imessage",
      "env": {
        "MCP_IMESSAGE_ENABLE_SEND": "1"
      }
    }
  }
}
```

## Tools

### `messages_threads`

Lists recent conversation threads. For small result sets it also includes recent messages for inline previews.

### `messages_fetch`

Fetches a conversation by contact name, participant number or email, or group chat identifier. Supports pagination with `before_timestamp` and `after_timestamp`.

### `messages_search`

Searches both matching conversations and matching message text.

### `messages_send`

Sends text, file attachments, or both. Works with direct recipients and group chats.

### `messages_draft`

Builds a draft composer UI without sending. The UI shows recent messages, keeps the draft editable, and lets the user approve the send inside the conversation view.

### `contacts_search`

Searches Contacts by name, phone, or email.

### `contacts_me`

Returns the local user's contact card from Contacts.app.

## Development

```bash
cd ui
npm ci
npm run dev
```

Build everything:

```bash
cd ui
npm ci
npm run build

cd ..
cargo check
```

### Skip the automatic UI build

If you already built `ui/dist/index.html` and want Cargo to skip rebuilding it:

```bash
MCP_IMESSAGE_SKIP_UI_BUILD=1 cargo build
```

## Security and privacy

- This project reads local macOS message data directly from the Messages SQLite database.
- Message and Contacts database access are opened in read-only mode.
- Nothing in this repo sends your message history to a remote service by default.
- Sending messages requires AppleScript automation access to Messages.app.
- AppleScript send and search actions pass user input through argv instead of string interpolation.

## License

[MIT](https://opensource.org/licenses/MIT)
