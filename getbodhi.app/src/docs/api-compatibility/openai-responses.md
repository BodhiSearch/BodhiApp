---
title: 'OpenAI Responses'
description: '/v1/responses async polling for reasoning and long-running tasks — pure pass-through to upstream providers, with strict ApiFormat rules'
order: 2
---

# OpenAI Responses API

`POST /v1/responses` is OpenAI's newer, **stateful** completion endpoint, designed for reasoning models and long-running workloads. Unlike Chat Completions (which returns a final answer in one round-trip or streams it), Responses creates a **server-side response object** that you fetch, poll, cancel, or list input items on. Bodhi exposes this surface for upstream providers that implement it.

> **Critical: Bodhi's `/v1/responses` is pure pass-through.** It forwards your request unchanged to the upstream provider configured on the matching API model. There is **no local llama.cpp implementation** of this endpoint — Responses requires a remote provider that natively supports it.

## When to use Responses vs Chat Completions

- **Reasoning models** (e.g. OpenAI's o-series). The reasoning trace is structured into the response object, and the server can return partial reasoning state via `input_items`.
- **Long-running tasks.** A response can keep working in the background; your client polls `GET /v1/responses/{id}` until it completes, instead of holding a long HTTP connection.
- **Cancellable workloads.** You can abort an in-flight response via `POST /v1/responses/{id}/cancel`.

For ordinary chat, prompt-engineering, or non-reasoning workloads, **stick with [Chat Completions](/docs/api-compatibility/openai-chat-completions)** — it's simpler and works for both local and remote models.

## Strict ApiFormat requirement

The `model` field must resolve to an **API model alias configured with the `openai_responses` ApiFormat**. Bodhi validates this on every request:

- Local model aliases (llama.cpp) → **rejected**. Responses doesn't run locally.
- API models configured with the `openai` ApiFormat (Chat Completions) → **rejected**. Wrong format.
- API models configured with the `openai_responses` ApiFormat → accepted, forwarded to the upstream `/v1/responses` endpoint.

If you get back an error like `"Model '...' is not configured for Responses API format. Configure an alias with 'openai_responses' format."`, the model name resolved to the wrong kind of alias. Reconfigure the API model under [Models → API Models](/docs/features/models/api-models) with the Responses format.

The reverse is also true: a model configured as `openai_responses` cannot be called via `/v1/chat/completions` or `/v1/embeddings` — those endpoints expect their own ApiFormats. Pick the right format for the workload when you create the API model.

## The endpoints

| Method + Path                        | Purpose                                                        |
| ------------------------------------ | -------------------------------------------------------------- |
| `POST /v1/responses`                 | Create a response (returns immediately with an ID; may stream) |
| `GET /v1/responses/{id}`             | Fetch the current state of a response                          |
| `DELETE /v1/responses/{id}`          | Delete a stored response                                       |
| `GET /v1/responses/{id}/input_items` | List the input items associated with a response                |
| `POST /v1/responses/{id}/cancel`     | Cancel an in-flight response                                   |

All of these proxy directly to the upstream provider. Whatever the upstream supports, Bodhi supports — the only thing Bodhi adds is auth and header rewriting.

## The flow: create → poll → result

A typical non-streaming flow:

1. **Create** with `POST /v1/responses`. The response body includes an `id` and a `status`. For fast models the status may already be `completed`; for reasoning models it's typically `in_progress`.
2. **Poll** `GET /v1/responses/{id}` on a backoff until `status` becomes `completed`, `failed`, or `cancelled`.
3. **Read** the result from the final response object's `output` field.

If you want streaming, set `"stream": true` on the create call — Bodhi forwards the SSE stream back to you exactly as the upstream produces it.

## Auth

Same as every other Bodhi endpoint:

```
Authorization: Bearer <bodhi-api-token>
```

The Bodhi token is rewritten to the upstream provider's auth header server-side. You never see or pass the upstream's API key.

## Example — create then poll

```bash
# 1. Create a response. The body is forwarded as-is to the upstream provider.
RESP_ID=$(curl -s -X POST http://localhost:1135/v1/responses \
  -H "Authorization: Bearer $BODHI_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "your-responses-alias",
    "input": "Plan a 3-day Tokyo itinerary."
  }' | jq -r '.id')

echo "Created response: $RESP_ID"

# 2. Poll until done.
while :; do
  STATUS=$(curl -s http://localhost:1135/v1/responses/$RESP_ID \
    -H "Authorization: Bearer $BODHI_TOKEN" | jq -r '.status')
  echo "Status: $STATUS"
  case "$STATUS" in
    completed|failed|cancelled) break ;;
  esac
  sleep 2
done

# 3. Fetch the final result.
curl -s http://localhost:1135/v1/responses/$RESP_ID \
  -H "Authorization: Bearer $BODHI_TOKEN" | jq '.output'
```

To cancel before completion: `curl -X POST http://localhost:1135/v1/responses/$RESP_ID/cancel -H "Authorization: Bearer $BODHI_TOKEN"`.

## Common gotchas

- **Wrong ApiFormat.** The most common error. The model must be set up as an `openai_responses` API model. Don't try to point the same alias at both `/v1/chat/completions` and `/v1/responses` — pick one format per alias.
- **No local model support.** Even if you have a strong local reasoning-style model, `/v1/responses` won't run it. Use `/v1/chat/completions` for local inference.
- **Response storage.** Bodhi does not persist response IDs locally — they live on the upstream provider. If the upstream evicts them, polling returns the upstream's not-found error.

## Full schema

See Swagger UI at `http://<your-bodhi-instance>/swagger-ui` for the complete shape of `CreateResponse`, `Response`, `input_items`, and the cancel/delete responses. Default local URL: `http://localhost:1135/swagger-ui`.
