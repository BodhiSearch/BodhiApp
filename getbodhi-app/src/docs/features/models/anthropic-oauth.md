---
title: 'Anthropic OAuth'
description: 'Use a Claude.ai / Anthropic Console subscription with Bodhi via an OAuth Bearer token instead of an API key'
order: 50
---

# Anthropic OAuth

The **Anthropic OAuth** API format lets Bodhi proxy requests to Anthropic using an OAuth-issued Bearer token instead of a regular API key. If you already pay for a Claude.ai or Anthropic Console subscription and have a tool (such as Anthropic's Claude Code CLI) that holds an OAuth token, you can plug that token into Bodhi and route Bodhi's chat through your existing account — no separate API key, no separate billing line.

This page focuses on the OAuth-specific flow. For everything else about API model configuration (prefixes, fetch-models, Test Connection, Extra Headers / Body), start at [API Models](/docs/features/models/api-models).

## Why OAuth instead of an API key?

Both options point Bodhi at Anthropic. The difference is which account pays for the call:

- **Anthropic** (regular API key) — uses an Anthropic _Console_ developer key. Usage is billed against the Console account.
- **Anthropic OAuth** — uses an OAuth Bearer token issued under a personal Claude.ai / Console _subscription_. Usage flows against that subscription's allowance.

If you already have a Claude.ai Pro or Console subscription with monthly inclusive usage, OAuth is the way to spend that allowance from Bodhi without provisioning a separate API key.

OAuth is **not** the right choice when you want per-tenant billing isolation, when you're integrating into a hosted Bodhi serving multiple users you don't own credentials for, or when you don't have a personal subscription to draw from.

## Where the token comes from

Bodhi does not issue or refresh Anthropic OAuth tokens itself. The token is obtained outside Bodhi — typically by signing into Anthropic's Claude Code CLI on your machine, which performs the OAuth handshake and stores a long-lived token. You copy that token into Bodhi's API model form. It begins with `sk-ant-oat01-`.

When the token eventually expires, you re-run the same flow in your token-issuing tool, then update Bodhi's stored token by editing the API model. There is no in-Bodhi sign-in screen for Anthropic; Bodhi only stores and forwards the token you provide.

## Setting it up

Open `/ui/models/api/new/` and follow the standard [API Models](/docs/features/models/api-models) flow with these specifics:

1. **API Format** → select **Anthropic (Claude Code OAuth)**.
2. **Base URL** → `https://api.anthropic.com/v1` (auto-filled).
3. **API Key field** → paste your OAuth token (the `sk-ant-oat01-...` value).
4. **Extra Headers and Extra Body** → leave the pre-filled defaults alone unless you have a specific reason to change them. Selecting the OAuth format auto-populates the headers and body fields the upstream OAuth flow requires (`anthropic-version`, `anthropic-beta` with the `oauth-2025-04-20` flag, a Claude-Code-style `user-agent`, plus a default `max_tokens` and a placeholder `system` block).
5. **Prefix** (optional) → see [API Models → Prefix routing](/docs/features/models/api-models). A prefix like `oauth/` keeps OAuth-routed requests distinct from a regular Anthropic API entry if you keep both.
6. **Fetch Models** → list the models your subscription can use.
7. **Test Connection** → confirm the token still works.
8. **Save**.

## What Bodhi does at request time

When a chat or `/anthropic/v1/messages` request hits Bodhi for an OAuth-format API model:

1. Bodhi attaches `Authorization: Bearer <your-oauth-token>` to the upstream call.
2. Bodhi merges your Extra Headers (including `anthropic-version`, `anthropic-beta`, `user-agent`) into the outbound request.
3. Bodhi deep-merges your Extra Body into the request body — the default body fields keep the upstream OAuth flow happy.
4. Headers that the _client_ sends starting with `anthropic-` continue to pass through, in addition to your defaults.
5. The upstream's response (streaming or whole) flows back unchanged.

Clients calling Bodhi don't need to know any of this. They authenticate to Bodhi with a Bodhi API token (or a session cookie) and call `/anthropic/v1/messages` exactly as they would for a regular Anthropic API key. Bodhi swaps in the OAuth Bearer at the proxy boundary.

## A working request

```bash
curl http://localhost:1135/anthropic/v1/messages \
  -H "Authorization: Bearer <bodhi-api-token>" \
  -H "anthropic-version: 2023-06-01" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "<your-anthropic-model>",
    "max_tokens": 4096,
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

`<bodhi-api-token>` is a Bodhi API token (see [API Tokens](/docs/features/auth/api-tokens)), not your Anthropic OAuth token. Bodhi attaches the OAuth Bearer to the upstream call on your behalf.

The same OAuth-backed entry also serves the chat UI: pick the API model from the picker in `/ui/chat/` and send messages as usual.

## When the token expires

OAuth tokens issued by tools like Claude Code have a lifetime. When the token expires Bodhi will start seeing 401s from Anthropic on every forwarded request. To recover:

1. Refresh the token in the issuing tool (re-run the Claude Code login flow, for example).
2. Open the API model in Bodhi's edit page.
3. Paste the new token into the API Key field and save.
4. Click Test Connection to confirm everything is back online.

There is no automatic refresh inside Bodhi today. If your tool stores the refresh token elsewhere, treat token rotation as a manual step.

## Troubleshooting

- _"401 from upstream"_ — token expired or revoked. Refresh in the issuing tool, paste the new token into the API model.
- _"Invalid token format"_ — Bodhi expects a token starting with `sk-ant-oat01-`. Make sure you pasted the OAuth token, not a regular API key.
- _"Test Connection times out"_ — outbound network to `api.anthropic.com` is blocked, or the model name in the test isn't allowed for your subscription. Test against a known-good model first.
- _"Streaming stutters or returns malformed chunks"_ — confirm the Extra Headers still include the OAuth-specific `anthropic-beta` flag. If you cleared the field, restore the format defaults by recreating the API model.

## Where to go next

- The full provider-configuration walkthrough: [API Models](/docs/features/models/api-models).
- Other Anthropic surface areas — `/anthropic/v1/messages` semantics will be detailed under API Compatibility once that section lands.
- Using Bodhi from your own client app: [Building Apps](/docs/developer/building-apps).
