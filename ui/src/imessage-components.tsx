import { useState } from "react";

// ---- Color system ----
const BLUE = "#007AFF";
const GRAY_BUBBLE = "#E9E9EB";
const BLUE_DARK = "#0A84FF";
const BUBBLE_DARK = "#26252A";

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

function useIsDark() {
  const [dark] = useState(() => {
    if (typeof window === "undefined") return false;
    return (
      document.documentElement.getAttribute("data-theme") === "dark" ||
      window.matchMedia("(prefers-color-scheme: dark)").matches
    );
  });
  return dark;
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
  const text = (props.text as string) || "";
  const sender = (props.sender as string) || "";
  const time = (props.time as string) || "";
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
        maxWidth: "100%",
      }}
    >
      {!isMe && (
        <div style={{ width: 28, flexShrink: 0 }}>
          {showAvatar && <AvatarCircle name={sender} size={28} />}
        </div>
      )}
      <div style={{ maxWidth: "72%", minWidth: 40 }}>
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
export function ThreadRow({ props }: { props: Record<string, unknown> }) {
  const name = (props.name as string) || "";
  const preview = (props.preview as string) || "";
  const time = (props.time as string) || "";
  const unread = props.unread === true || props.unread === "true";
  const dark = useIsDark();
  const [hovered, setHovered] = useState(false);

  return (
    <div
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 12,
        padding: "11px 16px",
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
  const name = (props.name as string) || "";
  const phones = (props.phones as string) || "";
  const emails = (props.emails as string) || "";
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
  const sender = (props.sender as string) || "";
  const text = (props.text as string) || "";
  const time = (props.time as string) || "";
  const query = (props.query as string) || "";
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
