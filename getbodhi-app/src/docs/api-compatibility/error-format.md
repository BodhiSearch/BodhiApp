---
title: 'Error Format'
description: 'The error envelopes you can see — Bodhi-native, OpenAI-style, and the provider-shaped wrappers the Anthropic and Gemini compat layers return'
order: 8
---

# Error Format

When something goes wrong, the JSON body you get back depends on which endpoint you called. Bodhi keeps each compat layer's errors in the shape that layer's clients expect, so an Anthropic SDK gets Anthropic-shaped errors and an OpenAI SDK gets OpenAI-shaped errors. There are four envelopes in practice. This page tells you which one you'll see and how to read each.

## Quick decision table

| Endpoint prefix                                                  | Local-error envelope                              | Upstream-error envelope (when proxied to a provider)     |
| ---------------------------------------------------------------- | ------------------------------------------------- | -------------------------------------------------------- |
| `/v1/chat/completions`, `/v1/embeddings`, `/v1/responses`        | OpenAI-style                                      | Pass-through (the upstream's own OpenAI-shaped envelope) |
| `/v1/models`, `/api/*` (Ollama)                                  | OpenAI-style                                      | n/a                                                      |
| `/anthropic/v1/messages`, `/v1/messages`, `/anthropic/v1/models` | Anthropic-style                                   | Pass-through (Anthropic's native envelope)               |
| `/v1beta/*` (Gemini)                                             | Gemini-style                                      | Pass-through (Google's native envelope)                  |
| `/bodhi/v1/*`                                                    | Bodhi-native                                      | n/a                                                      |
| `/bodhi/v1/apps/mcps/{id}/mcp`                                   | Bodhi-native (gateway-side) or upstream MCP shape | Pass-through (the upstream MCP server's response)        |

"Local" errors are produced inside Bodhi — token rejection, alias not found, validation failure, instance disabled. "Upstream" errors come from the remote provider Bodhi proxied your call to. Bodhi never rewraps an upstream error envelope, so SDK-side error handling continues to work.

## The Bodhi-native error envelope

Used by every endpoint under `/bodhi/v1/*` (UI APIs, MCP CRUD, app access, settings, tokens) and by the MCP proxy when the proxy itself rejects you (instance not found, access not granted, body too large).

```json
{
  "error": {
    "message": "Validation failed: name is required",
    "type": "invalid_request_error",
    "code": "validation_error",
    "params": { "field": "name" },
    "param": "{\"field\":\"name\"}"
  }
}
```

The HTTP status code carries the high-level category (4xx client error, 5xx server error). Inside the body:

- `message` — human-readable, safe to surface to end users for 4xx; 5xx messages are sanitized so internal details don't leak.
- `type` — error category. The values you can encounter are listed below.
- `code` — a stable, machine-readable code for programmatic branching (e.g. `alias_not_found`, `token_invalid`). Codes follow the pattern `<domain>-<reason>`.
- `params` — structured key/value context for validation errors (which field failed, what the invalid value was).
- `param` — a JSON-encoded form of `params`. This is a superset field so clients that only know the OpenAI shape (where `param` is a single string) can still read it.

This envelope is a strict superset of the OpenAI shape — an OpenAI SDK pointed at a Bodhi-native endpoint will still parse `error.message`, `error.type`, `error.code`, and `error.param`, ignoring `params`.

## The OpenAI-style error envelope

Used by `/v1/chat/completions`, `/v1/embeddings`, `/v1/responses`, `/v1/models`, and `/api/*` (Ollama).

```json
{
  "error": {
    "message": "Model 'foo' not found.",
    "type": "invalid_request_error",
    "param": "model=foo",
    "code": "alias_not_found"
  }
}
```

This is the same wire format the OpenAI API uses. The official OpenAI SDK and any tool that follows OpenAI's spec will parse it without modification. Differences from the Bodhi-native envelope:

- No `params` map — `param` is a flat string of `key=value` pairs.
- No 5xx-message sanitization here either — but the messages are kept generic by design.

## The Anthropic-style envelope

Used by `/anthropic/v1/messages`, `/v1/messages`, and `/anthropic/v1/models` for **local** errors. Upstream errors pass through.

```json
{
  "type": "error",
  "error": {
    "type": "invalid_request_error",
    "message": "Field 'model' is required and must be a string."
  }
}
```

Bodhi maps its internal error categories onto Anthropic's: `forbidden_error` becomes `permission_error`, `internal_server_error` becomes `api_error`, `service_unavailable` becomes `overloaded_error`, etc. The mapping is one-way and lossy by design — Anthropic SDKs only know the Anthropic types.

5xx messages are replaced with `"internal server error"` so internal service details never leak.

## The Gemini-style envelope

Used by `/v1beta/*` for **local** errors. Upstream errors pass through.

```json
{
  "error": {
    "code": 400,
    "message": "Model 'foo' not found.",
    "status": "NOT_FOUND"
  }
}
```

`status` is the gRPC-style string Google uses (`INVALID_ARGUMENT`, `UNAUTHENTICATED`, `PERMISSION_DENIED`, `NOT_FOUND`, `INTERNAL`, `UNAVAILABLE`). `code` mirrors the HTTP status. As with the Anthropic envelope, 5xx messages are sanitized.

## Common error categories

Across all envelopes, these are the categories you'll see most often. The exact `type` string differs by envelope (see the per-page docs for the mapping); the underlying meaning is the same.

| Category                                   | When                                                                   | Typical HTTP status |
| ------------------------------------------ | ---------------------------------------------------------------------- | ------------------- |
| `invalid_request`                          | Body validation, wrong content type, missing field, malformed model id | 400, 422            |
| `authentication_error`                     | No token, expired token, bad token                                     | 401                 |
| `permission_error`                         | Token is valid but lacks the scope or role for this endpoint           | 403                 |
| `not_found_error`                          | Model alias, MCP instance, user, or other entity doesn't exist         | 404                 |
| `rate_limit`                               | Upstream rate-limited (passed through verbatim from the provider)      | 429                 |
| `internal_error`                           | Bug, DB failure, llama.cpp crash, upstream 5xx                         | 500                 |
| `overloaded_error` / `service_unavailable` | Server temporarily can't handle the request                            | 503                 |

Specific stable `code` values exist for the most common reasons (`alias_not_found`, `token_invalid`, `validation_error`, `instance_disabled`, etc.) and are documented in the [error code reference](/docs/reference/error-codes).

## How SDKs surface these

- **OpenAI SDK** — raises `openai.BadRequestError`, `openai.AuthenticationError`, `openai.PermissionDeniedError`, etc. The `code` and `type` are accessible on the exception.
- **Anthropic SDK** — raises `anthropic.BadRequestError`, `anthropic.APIError`, etc. with `error.type` and `error.message`.
- **Google `google-genai` SDK** — raises a `google.genai.errors.APIError` with `code`, `message`, and `status`.
- **Raw `fetch` / `requests`** — read the JSON body and branch on `error.type` (or `error.error.type` for the Anthropic envelope).

## Telling them apart at a glance

If `body.error` is an object with a `message` field — it's OpenAI-style or Bodhi-native.
If `body.error` is an object with `code` (integer), `message`, and `status` (string) — it's Gemini.
If `body.type === "error"` and `body.error` is nested — it's Anthropic.
If `body.error.params` is present — it's specifically the Bodhi-native envelope (the others don't have that field).

## Source of truth

Every endpoint's exact response shape — including the schemas of all four envelopes — is in Swagger UI. The local default is `http://localhost:1135/swagger-ui`.
