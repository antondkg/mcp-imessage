import "./globals.css";
import { useState } from "react";
import { createRoot } from "react-dom/client";
import {
  MessageBubble,
  ThreadRow,
  ContactCardComponent,
  SearchResult,
} from "./imessage-components";

const messages = [
  { text: "yo are you free tonight? thinking about grabbing dinner", sender: "Jake Vollkommer", time: "2:14 PM", isMe: false, showAvatar: true },
  { text: "yeah down! where were you thinking?", sender: "You", time: "2:16 PM", isMe: true, showAvatar: false },
  { text: "that new sushi place on brickell? heard its fire", sender: "Jake Vollkommer", time: "2:17 PM", isMe: false, showAvatar: true },
  { text: "oh sushi sounds perfect. 7pm work?", sender: "You", time: "2:18 PM", isMe: true, showAvatar: false },
  { text: "perfect see you there", sender: "Jake Vollkommer", time: "2:19 PM", isMe: false, showAvatar: true },
  { text: "bet", sender: "You", time: "2:19 PM", isMe: true, showAvatar: false },
];

const threads = [
  { name: "Jake Vollkommer", preview: "perfect see you there", time: "2:19 PM", unread: true },
  { name: "Joao Goncalves", preview: "mano vou chegar atrasado", time: "1:45 PM", unread: false },
  { name: "Liliana Kawase", preview: "Te amo filho, liga quando chegar", time: "12:30 PM", unread: true },
  { name: "Vallor Team", preview: "Jake: shipped the fix, deploying now", time: "11:02 AM", unread: false },
  { name: "Heitor Goncalves", preview: "Bom dia filho", time: "Yesterday", unread: false },
];

const searchResults = [
  { sender: "Jake Vollkommer", text: "yo are you free tonight? thinking about grabbing dinner", time: "Mar 6", query: "dinner" },
  { sender: "Joao Goncalves", text: "bora jantar amanha? dinner at that brazilian place", time: "Mar 4", query: "dinner" },
  { sender: "Liliana Kawase", text: "Filho nao esquece o dinner com a familia domingo", time: "Mar 2", query: "dinner" },
];

const tabs = ["Messages", "Threads", "Contact", "Search"] as const;

function ConversationView() {
  return (
    <div>
      <div
        style={{
          padding: "12px 16px",
          borderBottom: "0.5px solid rgba(0,0,0,0.08)",
          display: "flex",
          alignItems: "center",
          gap: 10,
        }}
      >
        <div
          style={{
            width: 32,
            height: 32,
            borderRadius: 16,
            background: "linear-gradient(135deg, #A2A2A7, #C7C7CC)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: 12,
            fontWeight: 600,
            color: "#FFF",
          }}
        >
          JV
        </div>
        <div>
          <div style={{ fontSize: 15, fontWeight: 600 }}>Jake Vollkommer</div>
          <div style={{ fontSize: 12, color: "rgba(0,0,0,0.4)" }}>iMessage</div>
        </div>
      </div>
      <div style={{ padding: "16px 12px", display: "flex", flexDirection: "column", gap: 6 }}>
        <div style={{ textAlign: "center", fontSize: 12, color: "rgba(0,0,0,0.35)", margin: "4px 0 8px" }}>
          Today, Mar 6
        </div>
        {messages.map((m, i) => (
          <MessageBubble key={i} props={m} />
        ))}
        <div style={{ textAlign: "center", fontSize: 11, color: "rgba(0,0,0,0.3)", marginTop: 8 }}>
          Delivered
        </div>
      </div>
    </div>
  );
}

function ThreadsView() {
  return (
    <div>
      <div style={{ padding: "12px 16px", borderBottom: "0.5px solid rgba(0,0,0,0.08)" }}>
        <div style={{ fontSize: 15, fontWeight: 600 }}>Messages</div>
        <div style={{ fontSize: 12, color: "rgba(0,0,0,0.4)", marginTop: 2 }}>5 conversations</div>
      </div>
      <div style={{ padding: "0 16px" }}>
        {threads.map((t, i) => (
          <ThreadRow key={i} props={t} />
        ))}
      </div>
    </div>
  );
}

function ContactView() {
  return (
    <div style={{ padding: "0 16px" }}>
      <ContactCardComponent
        props={{
          name: "Jake Vollkommer",
          phones: "+1 (631) 745-7857",
          emails: "jake@vallor.ai",
        }}
      />
    </div>
  );
}

function SearchView() {
  return (
    <div>
      <div style={{ padding: "12px 16px", borderBottom: "0.5px solid rgba(0,0,0,0.08)" }}>
        <div
          style={{
            backgroundColor: "rgba(0,0,0,0.05)",
            borderRadius: 10,
            padding: "8px 12px",
            fontSize: 14,
            color: "rgba(0,0,0,0.4)",
            display: "flex",
            alignItems: "center",
            gap: 6,
          }}
        >
          <svg width="14" height="14" viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="8.5" cy="8.5" r="6" />
            <line x1="13" y1="13" x2="18" y2="18" />
          </svg>
          <span>dinner</span>
        </div>
        <div style={{ fontSize: 12, color: "rgba(0,0,0,0.4)", marginTop: 8 }}>
          3 results across all conversations
        </div>
      </div>
      <div style={{ padding: "0 16px" }}>
        {searchResults.map((r, i) => (
          <SearchResult key={i} props={r} />
        ))}
      </div>
    </div>
  );
}

function Demo() {
  const [active, setActive] = useState(0);

  return (
    <div
      style={{
        maxWidth: 420,
        margin: "0 auto",
        fontFamily:
          '-apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", Helvetica, Arial, sans-serif',
        WebkitFontSmoothing: "antialiased",
      }}
    >
      {/* Header */}
      <div style={{ padding: "20px 16px 12px" }}>
        <h1 style={{ fontSize: 28, fontWeight: 700, letterSpacing: -0.5 }}>
          iMessage UI
        </h1>
        <p style={{ fontSize: 13, color: "rgba(0,0,0,0.4)", marginTop: 4 }}>
          json-render MCP preview
        </p>
      </div>

      {/* Tab Bar */}
      <div
        style={{
          display: "flex",
          gap: 0,
          padding: "0 16px",
          marginBottom: 16,
        }}
      >
        {tabs.map((label, i) => (
          <button
            key={label}
            onClick={() => setActive(i)}
            style={{
              flex: 1,
              padding: "8px 0",
              fontSize: 13,
              fontWeight: i === active ? 600 : 400,
              color: i === active ? "#007AFF" : "rgba(0,0,0,0.4)",
              background: "none",
              border: "none",
              borderBottom: i === active ? "2px solid #007AFF" : "2px solid transparent",
              cursor: "pointer",
              transition: "all 0.15s ease",
            }}
          >
            {label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div
        style={{
          backgroundColor: "#FFFFFF",
          borderRadius: 16,
          border: "0.5px solid rgba(0,0,0,0.08)",
          overflow: "hidden",
          boxShadow: "0 1px 3px rgba(0,0,0,0.04), 0 4px 12px rgba(0,0,0,0.03)",
        }}
      >
        {active === 0 && <ConversationView />}
        {active === 1 && <ThreadsView />}
        {active === 2 && <ContactView />}
        {active === 3 && <SearchView />}
      </div>

      <div style={{ textAlign: "center", fontSize: 11, color: "rgba(0,0,0,0.25)", padding: "16px 0" }}>
        Rendered via render-imessage-ui MCP tool
      </div>
    </div>
  );
}

createRoot(document.getElementById("root")!).render(<Demo />);
