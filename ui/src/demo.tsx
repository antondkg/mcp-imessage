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
  { text: "yo are you free tonight? thinking about grabbing dinner", sender: "Jake Vollkommer", time: "", isMe: false, showAvatar: false, showTime: false, isGroupEnd: false },
  { text: "theres a few good spots on brickell", sender: "Jake Vollkommer", time: "2:14 PM", isMe: false, showAvatar: true, showTime: true, isGroupEnd: true },
  { text: "yeah down! where were you thinking?", sender: "You", time: "2:16 PM", isMe: true, showAvatar: false, showTime: true, isGroupEnd: true },
  { text: "that new sushi place on brickell? heard its fire", sender: "Jake Vollkommer", time: "2:17 PM", isMe: false, showAvatar: true, showTime: true, isGroupEnd: true },
  { text: "oh sushi sounds perfect", sender: "You", time: "", isMe: true, showAvatar: false, showTime: false, isGroupEnd: false },
  { text: "7pm work?", sender: "You", time: "2:18 PM", isMe: true, showAvatar: false, showTime: true, isGroupEnd: true },
  { text: "perfect see you there", sender: "Jake Vollkommer", time: "2:19 PM", isMe: false, showAvatar: true, showTime: true, isGroupEnd: true },
  { text: "bet", sender: "You", time: "", isMe: true, showAvatar: false, showTime: false, isGroupEnd: true },
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
          padding: "14px 16px",
          borderBottom: "0.5px solid rgba(0,0,0,0.06)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flexDirection: "column",
          gap: 2,
        }}
      >
        <div style={{ fontSize: 16, fontWeight: 600 }}>Jake Vollkommer</div>
        <div style={{ fontSize: 12, color: "rgba(0,0,0,0.35)" }}>iMessage</div>
      </div>
      <div style={{ padding: "12px 10px 16px", display: "flex", flexDirection: "column", gap: 0 }}>
        <div style={{ textAlign: "center", fontSize: 12, color: "rgba(0,0,0,0.3)", margin: "4px 0 12px", fontWeight: 500 }}>
          Today 2:14 PM
        </div>
        {messages.map((m, i) => (
          <MessageBubble key={i} props={m} />
        ))}
        <div style={{ textAlign: "center", fontSize: 11, color: "rgba(0,0,0,0.28)", marginTop: 6, fontWeight: 400 }}>
          Delivered
        </div>
      </div>
    </div>
  );
}

function ThreadsView() {
  return (
    <div>
      {threads.map((t, i) => (
        <ThreadRow key={i} props={t} />
      ))}
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
      <div style={{ padding: "12px 16px" }}>
        <div
          style={{
            backgroundColor: "rgba(0,0,0,0.04)",
            borderRadius: 10,
            padding: "9px 12px",
            fontSize: 15,
            color: "rgba(0,0,0,0.35)",
            display: "flex",
            alignItems: "center",
            gap: 8,
          }}
        >
          <svg width="15" height="15" viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round">
            <circle cx="8.5" cy="8.5" r="5.5" />
            <line x1="12.5" y1="12.5" x2="17" y2="17" />
          </svg>
          <span style={{ color: "rgba(0,0,0,0.7)" }}>dinner</span>
        </div>
        <div style={{ fontSize: 13, color: "rgba(0,0,0,0.35)", marginTop: 10, fontWeight: 500 }}>
          3 results
        </div>
      </div>
      {searchResults.map((r, i) => (
        <SearchResult key={i} props={r} />
      ))}
    </div>
  );
}

function Demo() {
  const [active, setActive] = useState(0);

  return (
    <div
      style={{
        maxWidth: 390,
        margin: "24px auto",
        fontFamily: '-apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", Helvetica, Arial, sans-serif',
        WebkitFontSmoothing: "antialiased",
        MozOsxFontSmoothing: "grayscale",
      }}
    >
      {/* Tab Bar */}
      <div
        style={{
          display: "flex",
          gap: 0,
          backgroundColor: "rgba(0,0,0,0.03)",
          borderRadius: 10,
          padding: 3,
          marginBottom: 16,
        }}
      >
        {tabs.map((label, i) => (
          <button
            key={label}
            onClick={() => setActive(i)}
            style={{
              flex: 1,
              padding: "7px 0",
              fontSize: 13,
              fontWeight: i === active ? 600 : 400,
              color: i === active ? "#000000" : "rgba(0,0,0,0.4)",
              background: i === active ? "#FFFFFF" : "transparent",
              border: "none",
              borderRadius: 8,
              cursor: "pointer",
              transition: "all 0.2s ease",
              boxShadow: i === active ? "0 1px 3px rgba(0,0,0,0.08)" : "none",
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
          borderRadius: 14,
          overflow: "hidden",
          boxShadow: "0 0 0 0.5px rgba(0,0,0,0.06), 0 2px 8px rgba(0,0,0,0.04), 0 8px 24px rgba(0,0,0,0.03)",
        }}
      >
        {active === 0 && <ConversationView />}
        {active === 1 && <ThreadsView />}
        {active === 2 && <ContactView />}
        {active === 3 && <SearchView />}
      </div>
    </div>
  );
}

createRoot(document.getElementById("root")!).render(<Demo />);
