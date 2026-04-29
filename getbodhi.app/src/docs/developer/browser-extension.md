---
title: 'Browser Extension'
description: 'How the Bodhi App browser extension exposes authenticated endpoints to web pages, and how SDK clients auto-detect it'
order: 254
---

# Browser Extension

The Bodhi App browser extension is an alternate transport for web apps that want to talk to a user's local Bodhi instance without dealing with cross-origin requests. It bridges the page's JavaScript context to the user's running Bodhi App via the browser, so a hosted webapp at `https://example.com` can call into `http://localhost:1135` without CORS friction or mixed-content warnings.

For most server-to-server callers this isn't relevant — direct HTTP is faster and simpler. The extension is an option, not a requirement.

## What it does

- **Bridges hosted web pages to a local Bodhi.** A page served from any origin can issue requests against the user's Bodhi instance through the extension, with the user's existing browser session attached.
- **Sidesteps CORS and mixed-content.** Because the extension proxies on the page's behalf, the browser doesn't see a cross-origin fetch.
- **Carries the user's session.** No need for the page to handle OAuth — if the user is signed in to Bodhi in another tab, the extension uses that session.

It does **not** add features beyond what direct HTTP can do — same endpoints, same auth model, same rate limits.

## When to use it

| Scenario                                                                                               | Recommended transport      |
| ------------------------------------------------------------------------------------------------------ | -------------------------- |
| Your app runs at the same origin as Bodhi (e.g. localhost:1135 itself)                                 | Direct HTTP                |
| Server-to-server, or a Node.js service                                                                 | Direct HTTP with API token |
| Hosted web app at a different origin (e.g. `https://example.com`) wanting access to user's local Bodhi | Browser extension          |
| Browser-based ChatGPT-style frontends from a hosted domain                                             | Browser extension          |

Direct HTTP and the extension are not mutually exclusive — clients usually try direct first and fall back.

## Installation

Currently supported: **Chrome** (and Chromium-based browsers — Edge, Brave, Arc).

1. From the Bodhi App setup wizard at [Step 5](/docs/install#step-5-browser-extension-optional), or any time later, click **Install Extension**.
2. The wizard opens the Chrome Web Store listing. Add the extension.
3. Return to Bodhi and click **Refresh Status** — the extension reports back as installed.

Firefox and Safari support are not available today.

## Using it from the SDK

The [`@bodhiapp/bodhi-js-react`](/docs/developer/bodhi-js-sdk/getting-started) SDK auto-detects which transport is available:

```typescript
import { useBodhi } from '@bodhiapp/bodhi-js-react';

function StatusBadge() {
  const { isDirect, isExtension, clientState } = useBodhi();
  // clientState.mode is 'direct' | 'extension' | null

  if (isDirect) return <span>Connected (direct)</span>;
  if (isExtension) return <span>Connected (via extension)</span>;
  return <span>Not connected</span>;
}
```

**Detection order:** direct HTTP first (lower latency, no extension dependency), extension as fallback. The selected mode is persisted across page loads, so the user's first connection picks the transport for that origin.

The same hooks (`useChatCompletions`, MCP tool helpers, etc.) work identically in both modes.

## How auth works under the extension

When a page calls Bodhi via the extension:

1. The extension forwards the request to the user's running Bodhi instance.
2. Bodhi sees the request as coming from the extension — the session cookie is already present because the user signed in via the normal OAuth flow.
3. The user's role and scopes apply. There is no "extension role" — the user is whoever they are when signed in to Bodhi.
4. If the user has not yet granted your app access to specific MCP servers, the SDK's `login` flow walks the user through the [app access request](/docs/developer/app-access-requests) approval before any tool calls.

In other words: the extension is a **transport**, not an identity. You still register your OAuth client, you still go through the access request flow for MCPs, you still get an API model token if you want to bypass user sessions for backend calls.

## Limitations

- **Chrome family only** — no Firefox/Safari builds today.
- **Streaming works** — Server-Sent Events flow through the extension correctly, so `/v1/chat/completions` with `stream: true` and `/v1beta/...:streamGenerateContent` behave as expected.
- **Latency** is slightly higher than direct HTTP because of the bridge hop. For most chat UIs this is imperceptible; for tight inner-loop benchmarks, prefer direct.
- **No background access** — the extension only relays requests originating from a page; it does not poll Bodhi when no page is open.

## Troubleshooting

- **"Status: not installed" after install** — refresh the wizard page; the extension reports its presence asynchronously.
- **Page can't reach Bodhi even with extension installed** — the user must have signed in to Bodhi at least once in this browser. Visit `http://localhost:1135` and sign in, then retry.
- **Mixed-content blocked despite extension** — if your page is loaded over HTTPS and Bodhi is `http://localhost:...`, modern Chrome generally permits localhost. If a corporate policy blocks it, fall back to running Bodhi behind a TLS-terminating reverse proxy. See [Reverse Proxy](/docs/deployment/reverse-proxy).

For deeper SDK help, see [Bodhi JS SDK → Getting Started](/docs/developer/bodhi-js-sdk/getting-started).
