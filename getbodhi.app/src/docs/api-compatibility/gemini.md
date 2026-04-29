---
title: 'Gemini'
description: 'Use the Google Gemini /v1beta/* surface against Bodhi — header rewriting, action dispatch, streaming, and embeddings'
order: 5
---

# Gemini

Bodhi exposes the Google Gemini `/v1beta/*` surface so any tool that speaks Gemini's wire format — Google's `@google/genai` SDK, the Python `google-genai` SDK, hand-rolled curl — works against Bodhi with two changes: point `base_url` at Bodhi, swap the Google API key for a Bodhi token.

If you're new to _why_ this works alongside the OpenAI and Anthropic compat layers, read [Concepts → API Compatibility](/docs/concepts/api-compatibility) first.

## The endpoint shape

Gemini's surface is action-dispatched: a single path with the action encoded as a suffix on the model segment.

```
GET  /v1beta/models                                    — list models
GET  /v1beta/models/{model}                            — model lookup
POST /v1beta/models/{model}:generateContent            — single-turn generation
POST /v1beta/models/{model}:streamGenerateContent      — streaming generation (SSE)
POST /v1beta/models/{model}:embedContent               — single-input embedding
POST /v1beta/models/{model}:batchEmbedContents         — batched embeddings
```

Bodhi splits the last `:` to extract `{model}` and `{action}` and dispatches accordingly. Unsupported actions are rejected with a Gemini-shaped `INVALID_ARGUMENT` error.

## Auth at the gateway

Bodhi accepts any of:

```
Authorization: Bearer <bodhi-api-token>
x-goog-api-key: <bodhi-api-token>
?key=<bodhi-api-token>          (query param — Gemini SDKs use this)
```

If `Authorization` is set, it wins. Otherwise `x-goog-api-key` is rewritten into `Authorization: Bearer ...` before the request reaches the route handler. This is a transport convenience so Gemini SDKs can point at Bodhi without code changes — but the value must always be a **Bodhi-issued token**, never a raw Google API key. The provider key is held server-side in the API-model record.

If you call this endpoint from inside the Bodhi web UI, the session cookie authenticates you and no header is needed.

See [API Tokens](/docs/features/auth/api-tokens) to mint a token.

## Header pass-through

Any header whose name starts with `x-goog-` is forwarded to the upstream provider verbatim. The Google SDKs use these for telemetry (`x-goog-api-client`, `x-goog-request-params`) and for opting in to features. The pass-through means upstream-side analytics and feature flags Just Work without a Bodhi release.

## Query parameters

Query parameters are forwarded verbatim. The most common one is `?alt=sse`, which Gemini SDKs use to request Server-Sent Events from `:streamGenerateContent`. Any other future query parameter Google adds is forwarded the same way.

## Model resolution

The `{model}` path segment is matched against Bodhi's catalog. To use this endpoint, the alias must be configured for the Gemini API format. Local llama.cpp aliases and aliases for other providers (OpenAI, Anthropic) are not exposed through `/v1beta/*`.

If your API-model alias defines a `prefix`, Bodhi strips it from `{model}` before forwarding. So a request to `/v1beta/models/myprefix-gemini-1.5-pro:generateContent` is forwarded as `gemini-1.5-pro`.

`GET /v1beta/models` returns the Gemini-shaped catalog of all configured Gemini aliases, served from cached metadata.

## Streaming

`POST /v1beta/models/{model}:streamGenerateContent` with `?alt=sse` returns Gemini's native SSE format (`Content-Type: text/event-stream`). Bodhi does not transform the stream. The official Gemini SDKs' streaming iterators work without modification.

If you omit `?alt=sse`, Gemini returns a streaming JSON array — that also passes through, but most SDKs expect SSE.

## Embeddings

`:embedContent` and `:batchEmbedContents` are both supported and dispatched to the same handler family. Use `:embedContent` for a single input and `:batchEmbedContents` for a list. The wire format matches Google's spec exactly.

## Errors

Local errors (token rejection, alias not found, malformed model id, unsupported action) are returned in **Google's native error envelope**:

```json
{
  "error": {
    "code": 400,
    "message": "Model 'foo' not found.",
    "status": "NOT_FOUND"
  }
}
```

Upstream errors pass through verbatim. The full mapping of error types is on the [Error Format](/docs/api-compatibility/error-format) page.

## Examples

### curl with `x-goog-api-key`

```bash
curl -X POST "http://localhost:1135/v1beta/models/your-gemini-alias:generateContent" \
  -H "x-goog-api-key: $BODHI_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "contents": [{"parts": [{"text": "Explain MCP in two sentences."}]}]
  }'
```

### curl with Bearer auth and streaming

```bash
curl -X POST "http://localhost:1135/v1beta/models/your-gemini-alias:streamGenerateContent?alt=sse" \
  -H "Authorization: Bearer $BODHI_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "contents": [{"parts": [{"text": "Count to 5."}]}]
  }'
```

### Python (google-genai SDK)

```python
import os
from google import genai

client = genai.Client(
    api_key=os.environ["BODHI_TOKEN"],  # not a raw Google API key
    http_options={"base_url": "http://localhost:1135"},
)

resp = client.models.generate_content(
    model="your-gemini-alias",
    contents="Hello",
)
print(resp.text)
```

The SDK appends `/v1beta/...` to `base_url`. Streaming, tool use, and embeddings work the same way as against Google directly.

## Full schema

See Swagger UI at `http://<your-bodhi-instance>/swagger-ui` for the complete request/response schema and live "try it out". The Gemini-flavoured spec is mounted under `/api-docs/openapi-gemini.json`. The local default is `http://localhost:1135/swagger-ui`.
