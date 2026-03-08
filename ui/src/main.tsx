import "./globals.css";
import { Component, useState, useEffect, type ReactNode } from "react";
import { createRoot } from "react-dom/client";
import { App as McpApp } from "@modelcontextprotocol/ext-apps";
import { JSONUIProvider, Renderer, defineRegistry } from "@json-render/react";
import { shadcnComponents } from "@json-render/shadcn";
import type { Spec } from "@json-render/core";
import { catalog } from "./catalog";
import {
  MessageBubble,
  ThreadRow,
  ContactCardComponent,
  SearchResult,
  ThreadListView,
  ThreadListSkeleton,
  ConversationView,
  ContactListView,
  ContactMeView,
  SendResultView,
  SearchResultsView,
} from "./imessage-components";

const { registry } = defineRegistry(catalog, {
  components: {
    ...shadcnComponents,
    MessageBubble: ({ props }: { props: Record<string, unknown> }) => (
      <MessageBubble props={props} />
    ),
    ThreadRow: ({ props }: { props: Record<string, unknown> }) => (
      <ThreadRow props={props} />
    ),
    ContactCard: ({ props }: { props: Record<string, unknown> }) => (
      <ContactCardComponent props={props} />
    ),
    SearchResult: ({ props }: { props: Record<string, unknown> }) => (
      <SearchResult props={props} />
    ),
  },
});

class ErrorBoundary extends Component<
  { children: ReactNode },
  { error: Error | null }
> {
  state = { error: null as Error | null };
  static getDerivedStateFromError(error: Error) {
    return { error };
  }
  render() {
    if (this.state.error) {
      return (
        <div style={{ padding: 16, color: "#dc2626", fontFamily: "monospace", fontSize: 13 }}>
          Error: {this.state.error.message}
        </div>
      );
    }
    return this.props.children;
  }
}

function forceFullWidth(spec: Spec): Spec {
  if (!spec.elements) return spec;
  const elements = { ...spec.elements };
  for (const [key, el] of Object.entries(elements)) {
    if (el.type === "Card" && el.props) {
      elements[key] = {
        ...el,
        props: { ...el.props, maxWidth: "full", centered: false },
      };
    }
  }
  return { ...spec, elements };
}

type ViewData =
  | { type: "spec"; spec: Spec }
  | { type: "threads"; data: any }
  | { type: "messages"; data: any }
  | { type: "contacts"; data: any }
  | { type: "contact_me"; data: any }
  | { type: "sent"; data: any }
  | { type: "search"; data: any };

function applyHostContext(ctx: {
  theme?: string;
  styles?: { variables?: Record<string, string> };
}) {
  if (ctx.theme) {
    document.documentElement.setAttribute("data-theme", ctx.theme);
    document.documentElement.style.colorScheme = ctx.theme;
  }
  if (ctx.styles?.variables) {
    const root = document.documentElement;
    for (const [key, value] of Object.entries(ctx.styles.variables)) {
      root.style.setProperty(key, value);
    }
  }
}

const FONT_FAMILY = '-apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", sans-serif';

// Try to extract a ViewData from any shape of data
function tryParseAny(data: unknown): ViewData | null {
  if (!data || typeof data !== "object") return null;
  const obj = data as Record<string, unknown>;

  // If it has content array (MCP tool result format), extract text and parse
  if (Array.isArray(obj.content)) {
    const textItem = obj.content.find((c: any) => c?.type === "text" && typeof c?.text === "string");
    if (textItem) {
      try {
        return tryParseAny(JSON.parse((textItem as any).text));
      } catch { /* fall through */ }
    }
  }

  // json-render spec
  if (obj.root && obj.elements) {
    return { type: "spec", spec: forceFullWidth(obj as unknown as Spec) };
  }
  // Thread list
  if (Array.isArray(obj.threads)) {
    return { type: "threads", data: obj };
  }
  // Send result with recent messages
  if (obj.success !== undefined && obj.recent_messages) {
    return { type: "sent", data: obj };
  }
  // Search results (has query + messages)
  if (Array.isArray(obj.messages) && obj.query) {
    return { type: "search", data: obj };
  }
  // Messages
  if (Array.isArray(obj.messages)) {
    return { type: "messages", data: obj };
  }
  // Contact list
  if (Array.isArray(obj.contacts)) {
    return { type: "contacts", data: obj };
  }
  // Own contact
  if (obj.me && typeof obj.me === "object" && (obj.me as any).name) {
    return { type: "contact_me", data: obj };
  }

  // Try parsing as a JSON string directly
  if (typeof data === "string") {
    try {
      return tryParseAny(JSON.parse(data));
    } catch { /* fall through */ }
  }

  return null;
}

function McpAppView() {
  const [view, setView] = useState<ViewData | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let viewReceived = false;

    function handleView(v: ViewData) {
      if (viewReceived) return;
      viewReceived = true;
      setView(v);
    }

    function tryHandle(data: unknown) {
      if (viewReceived) return;
      const parsed = tryParseAny(data);
      if (parsed) handleView(parsed);
    }

    function onMessage(event: MessageEvent) {
      if (viewReceived) return;
      const data = event.data;
      if (!data || typeof data !== "object") return;
      const method = (data as any).method as string | undefined;
      const params = (data as any).params as Record<string, unknown> | undefined;

      // Try the entire data object first
      tryHandle(data);

      // Tool input (spec-based rendering)
      if (method === "ui/notifications/tool-input" && params?.arguments) {
        const args = params.arguments as Record<string, unknown>;
        if (args.spec) tryHandle(args.spec);
      }

      // Tool result via postMessage (data tools)
      if (method === "ui/notifications/tool-result" && params) {
        tryHandle(params);
      }

      // Some hosts send the result directly in params.result
      if (params?.result) {
        tryHandle(params.result);
      }

      // Also try the entire params object
      if (params?.content) {
        tryHandle(params);
      }
    }
    window.addEventListener("message", onMessage);

    const app = new McpApp({ name: "imessage-ui", version: "1.0.0" });

    app.ontoolresult = (result) => tryHandle(result);

    app.onhostcontextchanged = (ctx) =>
      applyHostContext(ctx as Parameters<typeof applyHostContext>[0]);

    app.onerror = (err: unknown) => {
      setError(err instanceof Error ? err.message : String(err));
    };

    app
      .connect()
      .then(() => {
        const ctx = app.getHostContext?.();
        if (ctx) applyHostContext(ctx as Parameters<typeof applyHostContext>[0]);

        if (!ctx || !(ctx as Record<string, unknown>).theme) {
          const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
          document.documentElement.setAttribute("data-theme", prefersDark ? "dark" : "light");
          document.documentElement.style.colorScheme = prefersDark ? "dark" : "light";
        }
      })
      .catch((err: unknown) =>
        setError(err instanceof Error ? err.message : String(err)),
      );

    return () => {
      window.removeEventListener("message", onMessage);
      app.close().catch(() => {});
    };
  }, []);

  if (error) {
    return (
      <div style={{ padding: 16, color: "#dc2626", fontFamily: "monospace", fontSize: 13 }}>
        {error}
      </div>
    );
  }

  if (!view) {
    return (
      <div className="w-full" style={{ fontFamily: FONT_FAMILY }}>
        <ThreadListSkeleton />
      </div>
    );
  }

  const wrap = (children: ReactNode) => (
    <div className="w-full" style={{ fontFamily: FONT_FAMILY }}>{children}</div>
  );

  if (view.type === "threads") return wrap(<ThreadListView threads={view.data.threads} />);
  if (view.type === "messages") return wrap(<ConversationView messages={view.data.messages} />);
  if (view.type === "contacts") return wrap(<ContactListView contacts={view.data.contacts} />);
  if (view.type === "contact_me") return wrap(<ContactMeView me={view.data.me} />);
  if (view.type === "sent") return wrap(<SendResultView data={view.data} />);
  if (view.type === "search") return wrap(<SearchResultsView messages={view.data.messages} query={view.data.query} />);

  return (
    <JSONUIProvider registry={registry} initialState={view.spec.state ?? {}}>
      <div className="w-full" style={{ fontFamily: FONT_FAMILY }}>
        <Renderer spec={view.spec} registry={registry} />
      </div>
    </JSONUIProvider>
  );
}

createRoot(document.getElementById("root")!).render(
  <ErrorBoundary>
    <McpAppView />
  </ErrorBoundary>,
);
