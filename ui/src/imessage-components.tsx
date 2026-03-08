import { useState, useEffect, useRef, useSyncExternalStore } from "react";

// ---- Safe value coercion (raw JSON data may contain objects/arrays) ----
function str(v: unknown): string {
  if (v == null) return "";
  if (typeof v === "string") return v;
  if (typeof v === "number" || typeof v === "boolean") return String(v);
  return "";
}

function arr(v: unknown): unknown[] {
  return Array.isArray(v) ? v : [];
}

// ---- Color system ----
const BLUE = "#007AFF";
const GRAY_BUBBLE = "#E9E9EB";
const BLUE_DARK = "#0A84FF";
const BUBBLE_DARK = "#3A3A3C";

// Deterministic avatar color from name
const AVATAR_COLORS = [
  ["#FF6B6B", "#EE5A24"], // red
  ["#FF9F43", "#F39C12"], // orange
  ["#54A0FF", "#2E86DE"], // blue
  ["#5F27CD", "#341F97"], // purple
  ["#01A3A4", "#00867D"], // teal
  ["#10AC84", "#0A8B6E"], // green
  ["#FF6348", "#E55039"], // coral
  ["#6C5CE7", "#5B4FCF"], // indigo
  ["#FDA7DF", "#D980BC"], // pink
  ["#2ED573", "#1FAD5B"], // emerald
];

function hashName(name: string): number {
  let h = 0;
  for (let i = 0; i < name.length; i++) {
    h = ((h << 5) - h + name.charCodeAt(i)) | 0;
  }
  return Math.abs(h);
}

function avatarGradient(name: string): [string, string] {
  const idx = hashName(name) % AVATAR_COLORS.length;
  return AVATAR_COLORS[idx] as [string, string];
}

// Reactive dark mode detection that updates when data-theme changes
let _darkListeners = new Set<() => void>();
let _isDark = false;

function _computeIsDark(): boolean {
  if (typeof window === "undefined") return false;
  const theme = document.documentElement.getAttribute("data-theme");
  if (theme === "dark") return true;
  if (theme === "light") return false;
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

if (typeof window !== "undefined") {
  _isDark = _computeIsDark();

  // Watch for data-theme attribute changes on <html>
  const observer = new MutationObserver(() => {
    const next = _computeIsDark();
    if (next !== _isDark) {
      _isDark = next;
      _darkListeners.forEach((fn) => fn());
    }
  });
  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["data-theme", "style"],
  });

  // Also watch prefers-color-scheme
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
    const next = _computeIsDark();
    if (next !== _isDark) {
      _isDark = next;
      _darkListeners.forEach((fn) => fn());
    }
  });
}

function useIsDark(): boolean {
  return useSyncExternalStore(
    (cb) => {
      _darkListeners.add(cb);
      return () => _darkListeners.delete(cb);
    },
    () => _isDark,
    () => false,
  );
}

function AvatarCircle({
  name,
  size = 32,
}: {
  name: string;
  size?: number;
}) {
  const [c1, c2] = avatarGradient(name);
  const initials = name
    .split(" ")
    .map((w) => w[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();

  return (
    <div
      style={{
        width: size,
        height: size,
        borderRadius: size / 2,
        background: `linear-gradient(135deg, ${c1}, ${c2})`,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        flexShrink: 0,
        fontSize: size * 0.38,
        fontWeight: 600,
        color: "#FFFFFF",
        letterSpacing: 0.3,
      }}
    >
      {initials}
    </div>
  );
}

// ---- Message Bubble ----
export function MessageBubble({ props }: { props: Record<string, unknown> }) {
  const text = str(props.text);
  const sender = str(props.sender);
  const time = str(props.time);
  const isMe = props.isMe === true || props.isMe === "true";
  const showAvatar = props.showAvatar === true || props.showAvatar === "true";
  const showTime = props.showTime !== false && props.showTime !== "false";
  const isGroupEnd = props.isGroupEnd === true || props.isGroupEnd === "true";
  const dark = useIsDark();

  const bubbleBg = isMe
    ? dark ? BLUE_DARK : BLUE
    : dark ? BUBBLE_DARK : GRAY_BUBBLE;
  const textColor = isMe ? "#FFFFFF" : dark ? "#E5E5EA" : "#000000";

  return (
    <div
      style={{
        display: "flex",
        flexDirection: isMe ? "row-reverse" : "row",
        alignItems: "flex-end",
        gap: 6,
        marginBottom: isGroupEnd ? 8 : 2,
        width: "100%",
      }}
    >
      {!isMe && (
        <div style={{ width: 28, flexShrink: 0 }}>
          {showAvatar && <AvatarCircle name={sender} size={28} />}
        </div>
      )}
      <div style={{ maxWidth: "85%" }}>
        <div
          style={{
            backgroundColor: bubbleBg,
            color: textColor,
            padding: "7px 12px",
            borderRadius: 18,
            borderBottomLeftRadius: isMe ? 18 : isGroupEnd ? 4 : 18,
            borderBottomRightRadius: isMe ? (isGroupEnd ? 4 : 18) : 18,
            fontSize: 15,
            lineHeight: 1.38,
            wordBreak: "break-word",
            whiteSpace: "pre-wrap",
            overflowWrap: "break-word",
          }}
        >
          {text}
        </div>
        {showTime && time && (
          <div
            style={{
              fontSize: 11,
              color: dark ? "rgba(255,255,255,0.35)" : "rgba(0,0,0,0.35)",
              marginTop: 2,
              textAlign: isMe ? "right" : "left",
              paddingLeft: isMe ? 0 : 2,
              paddingRight: isMe ? 2 : 0,
            }}
          >
            {time}
          </div>
        )}
      </div>
    </div>
  );
}

// ---- Thread Row ----
export function ThreadRow({ props, onClick }: { props: Record<string, unknown>; onClick?: () => void }) {
  const name = str(props.name);
  const preview = str(props.preview);
  const time = str(props.time);
  const unread = props.unread === true || props.unread === "true";
  const dark = useIsDark();
  const [hovered, setHovered] = useState(false);

  return (
    <div
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={onClick}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 12,
        padding: "11px 16px",
        width: "100%",
        boxSizing: "border-box",
        borderBottom: `0.5px solid ${dark ? "rgba(255,255,255,0.06)" : "rgba(0,0,0,0.06)"}`,
        cursor: "pointer",
        backgroundColor: hovered
          ? dark ? "rgba(255,255,255,0.04)" : "rgba(0,0,0,0.03)"
          : "transparent",
        transition: "background-color 0.15s ease",
      }}
    >
      <AvatarCircle name={name} size={48} />
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", gap: 8 }}>
          <span
            style={{
              fontSize: 16,
              fontWeight: unread ? 600 : 400,
              color: dark ? "#FFFFFF" : "#000000",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
            }}
          >
            {name}
          </span>
          <div style={{ display: "flex", alignItems: "center", gap: 6, flexShrink: 0 }}>
            <span
              style={{
                fontSize: 14,
                color: dark ? "rgba(255,255,255,0.35)" : "rgba(0,0,0,0.35)",
              }}
            >
              {time}
            </span>
            <svg width="7" height="12" viewBox="0 0 7 12" fill="none" style={{ opacity: 0.3 }}>
              <path d="M1 1L6 6L1 11" stroke={dark ? "#FFF" : "#000"} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </div>
        </div>
        <div
          style={{
            fontSize: 14,
            color: dark ? "rgba(255,255,255,0.45)" : "rgba(0,0,0,0.4)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
            marginTop: 1,
            fontWeight: unread ? 500 : 400,
          }}
        >
          {preview}
        </div>
      </div>
      {unread && (
        <div
          style={{
            width: 12,
            height: 12,
            borderRadius: 6,
            backgroundColor: BLUE,
            flexShrink: 0,
          }}
        />
      )}
    </div>
  );
}

// ---- Contact Card ----
export function ContactCardComponent({ props }: { props: Record<string, unknown> }) {
  const name = str(props.name);
  const phones = str(props.phones);
  const emails = str(props.emails);
  const dark = useIsDark();

  const phoneList = phones.split(",").map((p) => p.trim()).filter(Boolean);
  const emailList = emails.split(",").map((e) => e.trim()).filter(Boolean);

  const sectionBg = dark ? "rgba(255,255,255,0.05)" : "#FFFFFF";
  const sectionBorder = dark ? "rgba(255,255,255,0.06)" : "rgba(0,0,0,0.06)";

  return (
    <div style={{ padding: "20px 0" }}>
      {/* Avatar + Name */}
      <div style={{ textAlign: "center", marginBottom: 20 }}>
        <div style={{ display: "inline-block", marginBottom: 10 }}>
          <AvatarCircle name={name} size={88} />
        </div>
        <div style={{ fontSize: 24, fontWeight: 600, color: dark ? "#FFFFFF" : "#000000", letterSpacing: -0.3 }}>
          {name}
        </div>
      </div>

      {/* Action buttons row */}
      <div style={{ display: "flex", gap: 8, justifyContent: "center", marginBottom: 20 }}>
        {[
          { icon: "message", label: "message" },
          { icon: "phone", label: "call" },
          { icon: "video", label: "video" },
          { icon: "mail", label: "mail" },
        ].map((action) => (
          <div
            key={action.label}
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              gap: 4,
              padding: "10px 16px",
              borderRadius: 12,
              backgroundColor: dark ? "rgba(255,255,255,0.06)" : "rgba(0,0,0,0.04)",
              minWidth: 64,
              cursor: "pointer",
            }}
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke={BLUE} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              {action.icon === "message" && (
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
              )}
              {action.icon === "phone" && (
                <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6A19.79 19.79 0 0 1 2.12 4.18 2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z" />
              )}
              {action.icon === "video" && (
                <>
                  <polygon points="23 7 16 12 23 17 23 7" />
                  <rect x="1" y="5" width="15" height="14" rx="2" ry="2" />
                </>
              )}
              {action.icon === "mail" && (
                <>
                  <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z" />
                  <polyline points="22,6 12,13 2,6" />
                </>
              )}
            </svg>
            <span style={{ fontSize: 11, color: BLUE, fontWeight: 500 }}>{action.label}</span>
          </div>
        ))}
      </div>

      {/* Info sections */}
      <div style={{ borderRadius: 12, overflow: "hidden", border: `0.5px solid ${sectionBorder}` }}>
        {phoneList.map((p, i) => (
          <div
            key={`phone-${i}`}
            style={{
              padding: "12px 16px",
              backgroundColor: sectionBg,
              borderBottom: `0.5px solid ${sectionBorder}`,
            }}
          >
            <div style={{ fontSize: 12, color: dark ? "rgba(255,255,255,0.4)" : "rgba(0,0,0,0.4)", marginBottom: 3 }}>
              phone
            </div>
            <div style={{ fontSize: 16, color: BLUE }}>{p}</div>
          </div>
        ))}
        {emailList.map((e, i) => (
          <div
            key={`email-${i}`}
            style={{
              padding: "12px 16px",
              backgroundColor: sectionBg,
              borderBottom: i < emailList.length - 1 ? `0.5px solid ${sectionBorder}` : "none",
            }}
          >
            <div style={{ fontSize: 12, color: dark ? "rgba(255,255,255,0.4)" : "rgba(0,0,0,0.4)", marginBottom: 3 }}>
              email
            </div>
            <div style={{ fontSize: 16, color: BLUE }}>{e}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

// ---- Search Result ----
export function SearchResult({ props }: { props: Record<string, unknown> }) {
  const sender = str(props.sender);
  const text = str(props.text);
  const time = str(props.time);
  const query = str(props.query);
  const dark = useIsDark();
  const [hovered, setHovered] = useState(false);

  // Highlight matching text
  const parts = query
    ? text.split(new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")})`, "gi"))
    : [text];

  return (
    <div
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: "flex",
        alignItems: "flex-start",
        gap: 12,
        padding: "12px 16px",
        borderBottom: `0.5px solid ${dark ? "rgba(255,255,255,0.06)" : "rgba(0,0,0,0.06)"}`,
        cursor: "pointer",
        backgroundColor: hovered
          ? dark ? "rgba(255,255,255,0.04)" : "rgba(0,0,0,0.03)"
          : "transparent",
        transition: "background-color 0.15s ease",
      }}
    >
      <AvatarCircle name={sender} size={40} />
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline" }}>
          <span style={{ fontSize: 15, fontWeight: 600, color: dark ? "#FFFFFF" : "#000000" }}>
            {sender}
          </span>
          <span style={{ fontSize: 13, color: dark ? "rgba(255,255,255,0.35)" : "rgba(0,0,0,0.35)", flexShrink: 0 }}>
            {time}
          </span>
        </div>
        <div
          style={{
            fontSize: 14,
            color: dark ? "rgba(255,255,255,0.55)" : "rgba(0,0,0,0.5)",
            marginTop: 3,
            lineHeight: 1.4,
          }}
        >
          {parts.map((part, i) =>
            query && part.toLowerCase() === query.toLowerCase() ? (
              <span
                key={i}
                style={{
                  backgroundColor: dark ? "rgba(10,132,255,0.25)" : "rgba(0,122,255,0.12)",
                  color: dark ? "#5AC8FA" : BLUE,
                  borderRadius: 3,
                  padding: "1px 3px",
                  fontWeight: 500,
                }}
              >
                {part}
              </span>
            ) : (
              <span key={i}>{part}</span>
            ),
          )}
        </div>
      </div>
    </div>
  );
}

// ---- Loading Skeletons ----
function SkeletonPulse({ width, height, radius = 4 }: { width: string | number; height: number; radius?: number }) {
  const dark = useIsDark();
  return (
    <div
      style={{
        width,
        height,
        borderRadius: radius,
        backgroundColor: dark ? "rgba(255,255,255,0.08)" : "rgba(0,0,0,0.06)",
        animation: "skeleton-pulse 1.5s ease-in-out infinite",
      }}
    />
  );
}

export function ThreadListSkeleton() {
  const dark = useIsDark();
  return (
    <div style={{ width: "100%" }}>
      <style>{`@keyframes skeleton-pulse { 0%,100% { opacity: 1; } 50% { opacity: 0.4; } }`}</style>
      {[0, 1, 2, 3, 4].map((i) => (
        <div
          key={i}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 12,
            padding: "11px 16px",
            borderBottom: `0.5px solid ${dark ? "rgba(255,255,255,0.06)" : "rgba(0,0,0,0.06)"}`,
          }}
        >
          <SkeletonPulse width={48} height={48} radius={24} />
          <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: 6 }}>
            <SkeletonPulse width={120 + i * 20} height={14} />
            <SkeletonPulse width="80%" height={12} />
          </div>
          <SkeletonPulse width={40} height={12} />
        </div>
      ))}
    </div>
  );
}

// ---- Time Formatting ----
export function formatTimestamp(unix: number): string {
  const date = new Date(unix * 1000);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / 86400000);

  if (days === 0) {
    return date.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
  }
  if (days === 1) return "Yesterday";
  if (days < 7) return date.toLocaleDateString([], { weekday: "long" });
  if (date.getFullYear() === now.getFullYear()) {
    return date.toLocaleDateString([], { month: "short", day: "numeric" });
  }
  return date.toLocaleDateString([], { month: "short", day: "numeric", year: "numeric" });
}

// ---- Auto-render: Thread List View ----
interface ThreadData {
  chat_id: number;
  chat_identifier: string;
  display_name: string | null;
  last_message: string;
  last_timestamp: number;
  last_is_from_me: boolean;
  participants: Array<{ handle: string; name: string | null }>;
  recent_messages?: MessageData[];
}

interface MessageData {
  id: number;
  text: string;
  timestamp: number;
  is_from_me: boolean;
  sender: string;
  sender_name?: string;
  chat_identifier: string;
}

export function ThreadListView({ threads }: { threads: ThreadData[] }) {
  const [selectedIdx, setSelectedIdx] = useState<number | null>(null);
  const dark = useIsDark();

  if (selectedIdx !== null) {
    const thread = threads[selectedIdx];
    const messages = arr(thread.recent_messages) as MessageData[];
    const name = str(thread.display_name) || str(thread.chat_identifier);
    return (
      <div style={{ width: "100%" }}>
        <div
          onClick={() => setSelectedIdx(null)}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            padding: "12px 16px",
            cursor: "pointer",
            borderBottom: `0.5px solid ${dark ? "rgba(255,255,255,0.06)" : "rgba(0,0,0,0.06)"}`,
            color: BLUE,
            fontSize: 16,
            fontWeight: 500,
          }}
        >
          <svg width="8" height="14" viewBox="0 0 8 14" fill="none">
            <path d="M7 1L1.5 7L7 13" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
          <span>{name}</span>
        </div>
        <ConversationView messages={messages} />
      </div>
    );
  }

  return (
    <div style={{ width: "100%" }}>
      {threads.map((t: any, i: number) => (
        <ThreadRow
          key={t.chat_id || i}
          onClick={arr(t.recent_messages).length ? () => setSelectedIdx(i) : undefined}
          props={{
            name: str(t.display_name) || str(t.chat_identifier),
            preview: (t.last_is_from_me ? "You: " : "") + str(t.last_message),
            time: formatTimestamp(t.last_timestamp),
            unread: false,
          }}
        />
      ))}
    </div>
  );
}

// ---- Auto-render: Conversation View ----
export function ConversationView({ messages }: { messages: MessageData[] }) {
  const bottomRef = useRef<HTMLDivElement>(null);
  // Messages come newest-first, reverse for chronological display
  const chronological = [...messages].reverse();

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "instant" });
  }, [messages]);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 2, padding: "8px 12px", width: "100%", boxSizing: "border-box" }}>
      {chronological.map((m, i) => {
        const prev = chronological[i - 1];
        const next = chronological[i + 1];
        const showAvatar = !m.is_from_me && (!prev || prev.is_from_me || prev.sender !== m.sender);
        const isGroupEnd = !next || next.is_from_me !== m.is_from_me || next.sender !== m.sender;
        return (
          <MessageBubble
            key={m.id || i}
            props={{
              text: str(m.text),
              sender: str(m.sender_name) || str(m.sender),
              time: formatTimestamp(m.timestamp),
              isMe: m.is_from_me,
              showAvatar,
              showTime: isGroupEnd,
              isGroupEnd,
            }}
          />
        );
      })}
      <div ref={bottomRef} />
    </div>
  );
}

// ---- Auto-render: Contact List View ----
interface ContactData {
  name: string;
  phones: string[];
  emails: string[];
}

function ContactRow({ contact, onClick }: { contact: ContactData; onClick: () => void }) {
  const dark = useIsDark();
  const [hovered, setHovered] = useState(false);
  const phones = arr(contact.phones).map(str);
  const emails = arr(contact.emails).map(str);
  const subtitle = phones[0] || emails[0] || "";

  return (
    <div
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={onClick}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 12,
        padding: "11px 16px",
        width: "100%",
        boxSizing: "border-box",
        borderBottom: `0.5px solid ${dark ? "rgba(255,255,255,0.06)" : "rgba(0,0,0,0.06)"}`,
        cursor: "pointer",
        backgroundColor: hovered
          ? dark ? "rgba(255,255,255,0.04)" : "rgba(0,0,0,0.03)"
          : "transparent",
        transition: "background-color 0.15s ease",
      }}
    >
      <AvatarCircle name={str(contact.name)} size={40} />
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ fontSize: 16, fontWeight: 500, color: dark ? "#FFFFFF" : "#000000" }}>
          {str(contact.name)}
        </div>
        {subtitle && (
          <div style={{ fontSize: 14, color: dark ? "rgba(255,255,255,0.45)" : "rgba(0,0,0,0.4)", marginTop: 1 }}>
            {subtitle}
          </div>
        )}
      </div>
      <svg width="7" height="12" viewBox="0 0 7 12" fill="none" style={{ opacity: 0.3, flexShrink: 0 }}>
        <path d="M1 1L6 6L1 11" stroke={dark ? "#FFF" : "#000"} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </div>
  );
}

export function ContactListView({ contacts }: { contacts: ContactData[] }) {
  const [selectedIdx, setSelectedIdx] = useState<number | null>(null);
  const dark = useIsDark();

  if (selectedIdx !== null) {
    const c = contacts[selectedIdx];
    return (
      <div style={{ width: "100%" }}>
        <div
          onClick={() => setSelectedIdx(null)}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            padding: "12px 16px",
            cursor: "pointer",
            borderBottom: `0.5px solid ${dark ? "rgba(255,255,255,0.06)" : "rgba(0,0,0,0.06)"}`,
            color: BLUE,
            fontSize: 16,
            fontWeight: 500,
          }}
        >
          <svg width="8" height="14" viewBox="0 0 8 14" fill="none">
            <path d="M7 1L1.5 7L7 13" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
          <span>Contacts</span>
        </div>
        <ContactCardComponent
          props={{
            name: str(c.name),
            phones: arr(c.phones).map(str).join(", "),
            emails: arr(c.emails).map(str).join(", "),
          }}
        />
      </div>
    );
  }

  return (
    <div style={{ width: "100%" }}>
      {contacts.map((c, i) => (
        <ContactRow key={i} contact={c} onClick={() => setSelectedIdx(i)} />
      ))}
    </div>
  );
}

// ---- Auto-render: Contact Me View ----
export function ContactMeView({ me }: { me: ContactData }) {
  return (
    <div style={{ width: "100%" }}>
      <ContactCardComponent
        props={{
          name: str(me.name),
          phones: arr(me.phones).map(str).join(", "),
          emails: arr(me.emails).map(str).join(", "),
        }}
      />
    </div>
  );
}

// ---- Auto-render: Send Result View ----
export function SendResultView({ data }: { data: { success: boolean; message: string; recent_messages?: MessageData[] } }) {
  const dark = useIsDark();

  return (
    <div style={{ width: "100%" }}>
      <div style={{
        display: "flex",
        alignItems: "center",
        gap: 8,
        padding: "10px 16px",
        backgroundColor: dark ? "rgba(48,209,88,0.12)" : "rgba(52,199,89,0.08)",
        borderBottom: `0.5px solid ${dark ? "rgba(255,255,255,0.06)" : "rgba(0,0,0,0.06)"}`,
      }}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="8" fill={dark ? "#30D158" : "#34C759"} />
          <path d="M4.5 8L7 10.5L11.5 5.5" stroke="#FFF" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
        <span style={{ fontSize: 14, fontWeight: 500, color: dark ? "#30D158" : "#34C759" }}>
          {str(data.message) || "Message sent"}
        </span>
      </div>
      {arr(data.recent_messages).length > 0 && (
        <ConversationView messages={arr(data.recent_messages) as MessageData[]} />
      )}
    </div>
  );
}

// ---- Auto-render: Search Results View ----
export function SearchResultsView({ messages, query }: { messages: MessageData[]; query: string }) {
  return (
    <div style={{ width: "100%" }}>
      {messages.map((m, i) => (
        <SearchResult
          key={m.id || i}
          props={{
            sender: str(m.sender_name) || str(m.sender),
            text: str(m.text),
            time: formatTimestamp(m.timestamp),
            query: str(query),
          }}
        />
      ))}
    </div>
  );
}
