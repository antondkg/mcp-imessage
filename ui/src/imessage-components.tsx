import { useState } from "react";

// iMessage blue
const BLUE = "#007AFF";
const GRAY = "#E9E9EB";
const GRAY_DARK = "#3A3A3C";
const BLUE_DARK = "#0A84FF";
const BUBBLE_DARK = "#2C2C2E";

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

// ---- Message Bubble ----
export function MessageBubble({ props }: { props: Record<string, unknown> }) {
  const text = (props.text as string) || "";
  const sender = (props.sender as string) || "";
  const time = (props.time as string) || "";
  const isMe = props.isMe === true || props.isMe === "true";
  const showAvatar = props.showAvatar !== false && props.showAvatar !== "false";
  const dark = useIsDark();

  const bubbleBg = isMe
    ? dark ? BLUE_DARK : BLUE
    : dark ? BUBBLE_DARK : GRAY;
  const textColor = isMe
    ? "#FFFFFF"
    : dark ? "#E5E5EA" : "#000000";
  const timeColor = isMe
    ? "rgba(255,255,255,0.7)"
    : dark ? "rgba(255,255,255,0.4)" : "rgba(0,0,0,0.4)";
  const initials = sender
    .split(" ")
    .map((w) => w[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();

  return (
    <div
      style={{
        display: "flex",
        flexDirection: isMe ? "row-reverse" : "row",
        alignItems: "flex-end",
        gap: 8,
        maxWidth: "100%",
      }}
    >
      {!isMe && showAvatar && (
        <div
          style={{
            width: 28,
            height: 28,
            borderRadius: 14,
            backgroundColor: dark ? "#48484A" : "#C7C7CC",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flexShrink: 0,
            fontSize: 11,
            fontWeight: 600,
            color: dark ? "#E5E5EA" : "#3A3A3C",
          }}
        >
          {initials}
        </div>
      )}
      {!isMe && !showAvatar && <div style={{ width: 28, flexShrink: 0 }} />}
      <div style={{ maxWidth: "75%", minWidth: 60 }}>
        <div
          style={{
            backgroundColor: bubbleBg,
            color: textColor,
            padding: "8px 14px",
            borderRadius: 18,
            borderBottomLeftRadius: isMe ? 18 : 4,
            borderBottomRightRadius: isMe ? 4 : 18,
            fontSize: 15,
            lineHeight: 1.35,
            wordBreak: "break-word",
          }}
        >
          {text}
        </div>
        <div
          style={{
            fontSize: 11,
            color: timeColor,
            marginTop: 2,
            textAlign: isMe ? "right" : "left",
            paddingLeft: isMe ? 0 : 4,
            paddingRight: isMe ? 4 : 0,
          }}
        >
          {time}
        </div>
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

  const initials = name
    .split(" ")
    .map((w) => w[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 12,
        padding: "12px 0",
        borderBottom: `0.5px solid ${dark ? "rgba(255,255,255,0.08)" : "rgba(0,0,0,0.08)"}`,
        cursor: "pointer",
      }}
    >
      <div
        style={{
          width: 44,
          height: 44,
          borderRadius: 22,
          background: `linear-gradient(135deg, ${dark ? "#48484A" : "#A2A2A7"}, ${dark ? "#636366" : "#C7C7CC"})`,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flexShrink: 0,
          fontSize: 15,
          fontWeight: 600,
          color: "#FFFFFF",
        }}
      >
        {initials}
      </div>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", gap: 8 }}>
          <span
            style={{
              fontSize: 15,
              fontWeight: unread ? 700 : 500,
              color: dark ? "#FFFFFF" : "#000000",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
            }}
          >
            {name}
          </span>
          <span
            style={{
              fontSize: 13,
              color: unread ? BLUE : dark ? "rgba(255,255,255,0.4)" : "rgba(0,0,0,0.4)",
              flexShrink: 0,
            }}
          >
            {time}
          </span>
        </div>
        <div
          style={{
            fontSize: 14,
            color: dark ? "rgba(255,255,255,0.5)" : "rgba(0,0,0,0.45)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
            marginTop: 2,
            fontWeight: unread ? 500 : 400,
          }}
        >
          {preview}
        </div>
      </div>
      {unread && (
        <div
          style={{
            width: 10,
            height: 10,
            borderRadius: 5,
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

  const initials = name
    .split(" ")
    .map((w) => w[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();

  const phoneList = phones.split(",").map((p) => p.trim()).filter(Boolean);
  const emailList = emails.split(",").map((e) => e.trim()).filter(Boolean);

  const labelStyle = {
    fontSize: 12,
    fontWeight: 500 as const,
    color: dark ? "rgba(255,255,255,0.4)" : "rgba(0,0,0,0.4)",
    textTransform: "uppercase" as const,
    letterSpacing: 0.5,
    marginTop: 16,
    marginBottom: 4,
  };
  const valueStyle = {
    fontSize: 15,
    color: BLUE,
    marginBottom: 2,
  };

  return (
    <div style={{ textAlign: "center", padding: "24px 0" }}>
      <div
        style={{
          width: 80,
          height: 80,
          borderRadius: 40,
          background: `linear-gradient(135deg, ${dark ? "#48484A" : "#A2A2A7"}, ${dark ? "#636366" : "#C7C7CC"})`,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          margin: "0 auto 12px",
          fontSize: 28,
          fontWeight: 600,
          color: "#FFFFFF",
        }}
      >
        {initials}
      </div>
      <div style={{ fontSize: 22, fontWeight: 600, color: dark ? "#FFFFFF" : "#000000" }}>
        {name}
      </div>
      {phoneList.length > 0 && (
        <div style={{ marginTop: 16, textAlign: "left" }}>
          <div style={labelStyle}>Phone</div>
          {phoneList.map((p, i) => (
            <div key={i} style={valueStyle}>{p}</div>
          ))}
        </div>
      )}
      {emailList.length > 0 && (
        <div style={{ textAlign: "left" }}>
          <div style={labelStyle}>Email</div>
          {emailList.map((e, i) => (
            <div key={i} style={valueStyle}>{e}</div>
          ))}
        </div>
      )}
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

  const initials = sender
    .split(" ")
    .map((w) => w[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();

  // Highlight matching text
  const parts = query
    ? text.split(new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")})`, "gi"))
    : [text];

  return (
    <div
      style={{
        display: "flex",
        alignItems: "flex-start",
        gap: 12,
        padding: "12px 0",
        borderBottom: `0.5px solid ${dark ? "rgba(255,255,255,0.08)" : "rgba(0,0,0,0.08)"}`,
      }}
    >
      <div
        style={{
          width: 36,
          height: 36,
          borderRadius: 18,
          background: `linear-gradient(135deg, ${dark ? "#48484A" : "#A2A2A7"}, ${dark ? "#636366" : "#C7C7CC"})`,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flexShrink: 0,
          fontSize: 13,
          fontWeight: 600,
          color: "#FFFFFF",
        }}
      >
        {initials}
      </div>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline" }}>
          <span style={{ fontSize: 14, fontWeight: 600, color: dark ? "#FFFFFF" : "#000000" }}>
            {sender}
          </span>
          <span style={{ fontSize: 12, color: dark ? "rgba(255,255,255,0.4)" : "rgba(0,0,0,0.4)" }}>
            {time}
          </span>
        </div>
        <div
          style={{
            fontSize: 14,
            color: dark ? "rgba(255,255,255,0.6)" : "rgba(0,0,0,0.55)",
            marginTop: 2,
            lineHeight: 1.4,
          }}
        >
          {parts.map((part, i) =>
            query && part.toLowerCase() === query.toLowerCase() ? (
              <mark
                key={i}
                style={{
                  backgroundColor: dark ? "rgba(10,132,255,0.3)" : "rgba(0,122,255,0.15)",
                  color: "inherit",
                  borderRadius: 2,
                  padding: "0 1px",
                }}
              >
                {part}
              </mark>
            ) : (
              <span key={i}>{part}</span>
            ),
          )}
        </div>
      </div>
    </div>
  );
}
