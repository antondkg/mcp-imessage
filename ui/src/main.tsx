import "./globals.css";
import { Component, useState, useEffect, useCallback, type ReactNode } from "react";
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

function parseSpec(
  result: { content?: Array<{ type: string; text?: string }> } | undefined,
): Spec | null {
  const text = result?.content?.find((c) => c.type === "text")?.text;
  if (!text) return null;
  try {
    return forceFullWidth(JSON.parse(text) as Spec);
  } catch {
    return null;
  }
}

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

function McpAppView() {
  const [spec, setSpec] = useState<Spec | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let specReceived = false;

    function handleSpec(s: Spec) {
      if (specReceived) return;
      specReceived = true;
      setSpec(s);
    }

    function onMessage(event: MessageEvent) {
      if (specReceived) return;
      const data = event.data as Record<string, unknown> | undefined;
      if (!data || typeof data !== "object") return;
      const method = data.method as string | undefined;
      const params = data.params as Record<string, unknown> | undefined;

      if (method === "ui/notifications/tool-input" && params?.arguments) {
        const args = params.arguments as Record<string, unknown>;
        const rawSpec = args.spec;
        if (
          rawSpec &&
          typeof rawSpec === "object" &&
          "root" in rawSpec &&
          "elements" in rawSpec
        ) {
          handleSpec(forceFullWidth(rawSpec as Spec));
        }
      }
    }
    window.addEventListener("message", onMessage);

    const app = new McpApp({ name: "imessage-ui", version: "1.0.0" });

    app.ontoolresult = (result) => {
      const parsed = parseSpec(
        result as { content?: Array<{ type: string; text?: string }> },
      );
      if (parsed) handleSpec(parsed);
    };

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

  if (!spec) {
    return (
      <div style={{ padding: 16, color: "#6b7280", fontFamily: "sans-serif", fontSize: 14 }}>
        Loading...
      </div>
    );
  }

  return (
    <JSONUIProvider registry={registry} initialState={spec.state ?? {}}>
      <div className="w-full" style={{ fontFamily: '-apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", sans-serif' }}>
        <Renderer spec={spec} registry={registry} />
      </div>
    </JSONUIProvider>
  );
}

createRoot(document.getElementById("root")!).render(
  <ErrorBoundary>
    <McpAppView />
  </ErrorBoundary>,
);
