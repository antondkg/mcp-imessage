import { defineCatalog } from "@json-render/core";
import { schema } from "@json-render/react/schema";
import { shadcnComponentDefinitions } from "@json-render/shadcn/catalog";
import { z } from "zod";

export const catalog = defineCatalog(schema, {
  components: {
    ...shadcnComponentDefinitions,

    MessageBubble: {
      props: z.object({
        text: z.string(),
        sender: z.string(),
        time: z.string(),
        isMe: z.boolean(),
        showAvatar: z.boolean().nullable(),
      }),
      description:
        "iMessage chat bubble. isMe=true renders blue right-aligned (sent), isMe=false renders gray left-aligned (received). Set showAvatar=true on received messages to show sender initials.",
      example: {
        text: "hey whats up",
        sender: "Jake",
        time: "2:14 PM",
        isMe: false,
        showAvatar: true,
      },
    },

    ThreadRow: {
      props: z.object({
        name: z.string(),
        preview: z.string(),
        time: z.string(),
        unread: z.boolean().nullable(),
      }),
      description:
        "Conversation thread row with avatar, name, last message preview, time, and unread dot. Use in a vertical Stack to build a thread list.",
      example: {
        name: "Jake Vollkommer",
        preview: "see you there",
        time: "2:19 PM",
        unread: true,
      },
    },

    ContactCard: {
      props: z.object({
        name: z.string(),
        phones: z.string().nullable(),
        emails: z.string().nullable(),
      }),
      description:
        "iOS-style contact card with centered avatar, name, phone numbers and emails. Comma-separate multiple phones/emails.",
      example: {
        name: "Jake Vollkommer",
        phones: "+1 (631) 745-7857",
        emails: "jake@vallor.ai",
      },
    },

    SearchResult: {
      props: z.object({
        sender: z.string(),
        text: z.string(),
        time: z.string(),
        query: z.string().nullable(),
      }),
      description:
        "Search result row showing sender, matched message text with highlighted query, and time. Use in a vertical Stack.",
      example: {
        sender: "Jake",
        text: "thinking about grabbing dinner",
        time: "Mar 6",
        query: "dinner",
      },
    },
  },
  actions: {},
});
