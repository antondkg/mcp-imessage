import "./globals.css";
import type { ReactNode } from "react";
import { createRoot } from "react-dom/client";
import {
  ContactMeView,
  ConversationView,
  SearchResultsView,
  ThreadListView,
} from "./imessage-components";

const threads = [
  {
    chat_id: 1,
    chat_identifier: "chat-alpha",
    display_name: "Alex Johnson",
    last_message: "Perfect. I will send the notes after lunch.",
    last_timestamp: 1_741_958_800,
    last_is_from_me: false,
    participants: [{ handle: "+14155550123", name: "Alex Johnson" }],
    recent_messages: [
      {
        id: 101,
        text: "Perfect. I will send the notes after lunch.",
        timestamp: 1_741_958_800,
        is_from_me: false,
        sender: "+14155550123",
        sender_name: "Alex Johnson",
        chat_identifier: "chat-alpha",
      },
      {
        id: 100,
        text: "Works for me. Want to regroup at 1:30?",
        timestamp: 1_741_958_320,
        is_from_me: true,
        sender: "me",
        chat_identifier: "chat-alpha",
      },
    ],
  },
  {
    chat_id: 2,
    chat_identifier: "chat-bravo",
    display_name: "Design Sync",
    last_message: "Uploading the refreshed mockups now.",
    last_timestamp: 1_741_955_920,
    last_is_from_me: true,
    participants: [
      { handle: "+14155550124", name: "Morgan Lee" },
      { handle: "+14155550125", name: "Sam Rivera" },
    ],
  },
  {
    chat_id: 3,
    chat_identifier: "chat-charlie",
    display_name: "Jamie Carter",
    last_message: "Thanks again. This UI looks great.",
    last_timestamp: 1_741_953_220,
    last_is_from_me: false,
    participants: [{ handle: "+14155550126", name: "Jamie Carter" }],
  },
];

const conversation = [
  {
    id: 1,
    text: "Did you get a chance to review the message flow?",
    timestamp: 1_741_958_020,
    is_from_me: false,
    sender: "+14155550123",
    sender_name: "Alex Johnson",
    chat_identifier: "chat-alpha",
  },
  {
    id: 2,
    text: "Yep. The thread list and search states feel really natural now.",
    timestamp: 1_741_958_220,
    is_from_me: true,
    sender: "me",
    chat_identifier: "chat-alpha",
  },
  {
    id: 3,
    text: "Nice. I tightened the spacing and the avatars so it reads closer to Messages.app.",
    timestamp: 1_741_958_560,
    is_from_me: false,
    sender: "+14155550123",
    sender_name: "Alex Johnson",
    chat_identifier: "chat-alpha",
  },
  {
    id: 4,
    text: "Perfect. I will send the notes after lunch.",
    timestamp: 1_741_958_800,
    is_from_me: false,
    sender: "+14155550123",
    sender_name: "Alex Johnson",
    chat_identifier: "chat-alpha",
  },
];

const searchConversations = [
  {
    chat_id: 10,
    chat_identifier: "chat-delta",
    display_name: "Ops Review",
    last_message: "The launch checklist is ready for the final pass.",
    last_timestamp: 1_741_956_600,
    last_is_from_me: false,
    participants: [
      { handle: "+14155550127", name: "Taylor Brooks" },
      { handle: "+14155550128", name: "Jordan Kim" },
    ],
    recent_messages: [
      {
        id: 201,
        text: "The launch checklist is ready for the final pass.",
        timestamp: 1_741_956_600,
        is_from_me: false,
        sender: "+14155550127",
        sender_name: "Taylor Brooks",
        chat_identifier: "chat-delta",
      },
    ],
  },
];

const searchMessages = [
  {
    id: 301,
    text: "Can you send the launch checklist before the review call?",
    timestamp: 1_741_956_000,
    is_from_me: false,
    sender: "+14155550127",
    sender_name: "Taylor Brooks",
    chat_identifier: "chat-delta",
  },
  {
    id: 302,
    text: "I updated the checklist and attached the final version.",
    timestamp: 1_741_956_240,
    is_from_me: true,
    sender: "me",
    chat_identifier: "chat-delta",
  },
];

const me = {
  name: "Local User",
  phones: ["+1 (415) 555-0123"],
  emails: ["local.user@example.com"],
};

function PhoneFrame({
  title,
  subtitle,
  contentHeight = 760,
  children,
}: {
  title: string;
  subtitle: string;
  contentHeight?: number;
  children: ReactNode;
}) {
  return (
    <div
      style={{
        width: 420,
        background: "rgba(255,255,255,0.78)",
        border: "1px solid rgba(15,23,42,0.08)",
        borderRadius: 36,
        overflow: "hidden",
        boxShadow:
          "0 30px 80px rgba(15,23,42,0.18), inset 0 1px 0 rgba(255,255,255,0.8)",
        backdropFilter: "blur(18px)",
      }}
    >
      <div
        style={{
          padding: "18px 20px 14px",
          borderBottom: "1px solid rgba(15,23,42,0.06)",
          background:
            "linear-gradient(180deg, rgba(255,255,255,0.92), rgba(248,250,252,0.88))",
        }}
      >
        <div style={{ fontSize: 22, fontWeight: 700, color: "#0f172a" }}>
          {title}
        </div>
        <div style={{ fontSize: 14, color: "rgba(15,23,42,0.55)", marginTop: 4 }}>
          {subtitle}
        </div>
      </div>
      <div
        style={{
          height: contentHeight,
          overflow: "hidden",
          background:
            "linear-gradient(180deg, rgba(248,250,252,1), rgba(255,255,255,1))",
        }}
      >
        {children}
      </div>
    </div>
  );
}

function Layout({ children }: { children: ReactNode }) {
  return (
    <div
      style={{
        minHeight: "100vh",
        padding: 48,
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "center",
        background:
          "radial-gradient(circle at top left, rgba(96,165,250,0.22), transparent 32%), radial-gradient(circle at bottom right, rgba(59,130,246,0.18), transparent 28%), linear-gradient(135deg, #eff6ff 0%, #f8fafc 48%, #eef2ff 100%)",
      }}
    >
      {children}
    </div>
  );
}

function App() {
  document.documentElement.setAttribute("data-theme", "light");
  document.documentElement.style.colorScheme = "light";

  const view = new URLSearchParams(window.location.search).get("view") ?? "threads";

  if (view === "conversation") {
    return (
      <Layout>
        <PhoneFrame
          title="Conversation View"
          subtitle="A native-style message thread with avatars, grouped bubbles, and timestamps"
          contentHeight={620}
        >
          <ConversationView messages={conversation} autoScroll={false} />
        </PhoneFrame>
      </Layout>
    );
  }

  if (view === "search") {
    return (
      <Layout>
        <div style={{ display: "flex", gap: 28, alignItems: "center" }}>
          <PhoneFrame
            title="Search Results"
            subtitle="Conversations and matching messages rendered in one flow"
            contentHeight={560}
          >
            <SearchResultsView
              messages={searchMessages}
              conversations={searchConversations}
              query="checklist"
            />
          </PhoneFrame>
          <PhoneFrame
            title="Contact Card"
            subtitle="Contact details rendered with the same iOS-style language"
            contentHeight={560}
          >
            <ContactMeView me={me} />
          </PhoneFrame>
        </div>
      </Layout>
    );
  }

  return (
    <Layout>
      <PhoneFrame
        title="Thread List"
        subtitle="Recent conversations rendered with the embedded iMessage-style UI"
        contentHeight={420}
      >
        <ThreadListView threads={threads} />
      </PhoneFrame>
    </Layout>
  );
}

createRoot(document.getElementById("root")!).render(<App />);
