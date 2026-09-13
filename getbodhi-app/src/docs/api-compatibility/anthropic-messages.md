---
title: 'Anthropic Messages'
description: 'Use the Anthropic Messages API against Bodhi — header rewriting, model resolution, streaming, and the API-key vs Anthropic-OAuth split'
order: 4
---

# Anthropic Messages

`POST /anthropic/v1/messages` and `POST /v1/messages` are wired to the **same handler** — pick whichever your SDK or proxy already targets. Anthropic's official SDK and any tool that speaks the Messages wire format (LangChain, LlamaIndex, hand-rolled curl) work against either path with two changes: point `base_url` at Bodhi, swap the Anthropic API key for a Bodhi token.

If you're new to _why_ this works, read [Concepts → API Compatibility](/docs/concepts/api-compatibility) first.

## Two paths to "talk Anthropic" through Bodhi

There are two distinct upstream paths, and they share this endpoint:

1. **API-key path** — Bodhi holds an Anthropic API key in the API-model record. Calls are forwarded to `https://api.anthropic.com/v1/messages` with `x-api-key: <stored-key>` and the default `anthropic-version` if you didn't set one.
2. **Anthropic-OAuth path** — Bodhi holds an OAuth Bearer token (obtained via the Claude Code CLI flow) and forwards the call as `Authorization: Bearer <oauth-token>`. This is the path you use when you want to talk to Anthropic without paying for an API key — the same auth your Claude Pro / Claude.ai account uses.

Which path runs is determined by the API-model alias, not the request. From the client's perspective the wire format is identical. See [Anthropic OAuth](/docs/features/models/anthropic-oauth) for setup.

## Auth at the gateway

Bodhi accepts either of:

```
Authorization: Bearer <bodhi-api-token>
x-api-key: <bodhi-api-token>
```

If both are present, the `Authorization` header wins. The `x-api-key` header is a transport convenience so that Anthropic SDKs can point at Bodhi without code changes — but the value you put in it must always be a **Bodhi-issued token**, never a raw Anthropic key. Bodhi rewrites `x-api-key` into `Authorization: Bearer ...` before the request reaches the route handler. The upstream provider key is held server-side in the API-model record, never sent by the client.

If you call this endpoint from inside the Bodhi web UI, the session cookie authenticates you and no header is needed.

See [API Tokens](/docs/features/auth/api-tokens) to mint a token.

## Header pass-through

Any header whose name starts with `anthropic-` is forwarded to the upstream provider verbatim. This includes:

- `anthropic-version` — pinning to a specific Anthropic API version. If you don't send one, Bodhi inserts a sensible default.
- `anthropic-beta` — opt-in to Anthropic's beta features (e.g., extended thinking, computer use).
- Any other future `anthropic-*` header your SDK adds.

This means the latest Anthropic SDK features that ride on headers Just Work — Bodhi doesn't need a release for each one.

## Model resolution

The `model` field is matched against Bodhi's catalog. To use this endpoint, the alias must be configured for the `anthropic` or `anthropic_oauth` API format. If the alias is configured for OpenAI, Gemini, or another format, the request is rejected with `invalid_request_error`. Local llama.cpp aliases are not exposed through `/anthropic/v1/messages`.

`GET /anthropic/v1/models` returns the Anthropic-shaped catalog of all configured Anthropic and Anthropic-OAuth aliases, served from cached metadata — no upstream call.

If your API-model alias defines a `prefix`, Bodhi strips it from the `model` field before forwarding. So a request for `prefix-claude-sonnet-4` is forwarded as `claude-sonnet-4`.

## Streaming

Set `"stream": true` and the response comes back as Anthropic's native SSE format (`Content-Type: text/event-stream`) — `message_start`, `content_block_delta`, `content_block_stop`, `message_delta`, `message_stop` events. Bodhi does not transform the stream. The official Anthropic SDK's streaming iterators work without modification.

## Tool use

Anthropic's `tools` array, `tool_choice`, and `tool_use` / `tool_result` content blocks pass through verbatim. The exact schema (input shapes, blocks, choice modes) is documented at [docs.anthropic.com](https://docs.anthropic.com/en/api/messages). Bodhi adds nothing to it and removes nothing from it — it's a near-transparent proxy.

## Anthropic-OAuth — what's different

When the alias uses the Anthropic-OAuth format, three things change relative to the API-key path:

- The outbound auth header is `Authorization: Bearer <oauth-token>`, not `x-api-key`.
- Extra headers and extra body fields configured on the alias are merged into every request before it leaves the gateway. This is how the OAuth flow injects the Claude Code system prompt and metadata that Anthropic expects from CLI clients.
- **Token refresh is manual.** When the OAuth Bearer expires, Bodhi returns the upstream `401`. Refresh by re-running the Claude Code CLI flow and pasting the new token into the API-model record. This is by design — Bodhi doesn't store the OAuth refresh token server-side.

The wire format your client sees is unchanged either way. See [Anthropic OAuth](/docs/features/models/anthropic-oauth) for the setup walkthrough.

## Errors

Local errors (token rejection, alias not found, malformed `model`, validation failures) are returned in **Anthropic's native error envelope**:

```json
{
  "type": "error",
  "error": {
    "type": "invalid_request_error",
    "message": "Field 'model' is required and must be a string."
  }
}
```

Upstream errors (rate limit, billing, model unavailable) pass through verbatim — same status, same body — so your existing Anthropic SDK error-handling continues to work. The full mapping of error types is on the [Error Format](/docs/api-compatibility/error-format) page.

## Examples

### curl (API-key path)

```bash
curl -X POST http://localhost:1135/anthropic/v1/messages \
  -H "x-api-key: $BODHI_TOKEN" \
  -H "anthropic-version: 2023-06-01" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "your-anthropic-alias",
    "max_tokens": 256,
    "messages": [
      {"role": "user", "content": "Explain MCP in two sentences."}
    ]
  }'
```

The same call works against `/v1/messages`. Add `"stream": true` for SSE.

### Python (Anthropic SDK)

```python
import os
from anthropic import Anthropic

client = Anthropic(
    base_url="http://localhost:1135/anthropic",
    api_key=os.environ["BODHI_TOKEN"],  # not a raw Anthropic key
)

msg = client.messages.create(
    model="your-anthropic-alias",
    max_tokens=256,
    messages=[{"role": "user", "content": "Hello"}],
)
print(msg.content[0].text)
```

The SDK appends `/v1/messages` to `base_url`, so the effective URL is `http://localhost:1135/anthropic/v1/messages`. Switch the alias to an Anthropic-OAuth one and the same code keeps working — no client change.

## Full schema

See Swagger UI at `http://<your-bodhi-instance>/swagger-ui` for the complete request/response schema and live "try it out". The Anthropic-flavoured spec is mounted under `/api-docs/openapi-anthropic.json`. The local default is `http://localhost:1135/swagger-ui`.
