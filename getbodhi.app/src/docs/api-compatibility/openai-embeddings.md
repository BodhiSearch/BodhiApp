---
title: 'OpenAI Embeddings'
description: 'Use /v1/embeddings against local or remote embedding models with the standard OpenAI SDK'
order: 3
---

# OpenAI Embeddings

`POST /v1/embeddings` returns vector embeddings for one or more input strings. The wire format matches OpenAI's, so any client that already calls `client.embeddings.create(...)` works against Bodhi by changing `base_url` and the API key.

Use this for RAG, semantic search, retrieval, classification — anything that needs to reduce text to a fixed-length numeric vector.

## Auth

```
Authorization: Bearer <bodhi-api-token>
```

Same Bodhi-issued token that works against `/v1/chat/completions`. See [API Tokens](/docs/features/auth/api-tokens).

## Model resolution

The `model` field resolves against Bodhi's combined catalog, exactly like Chat Completions:

- **Local embedding aliases** — a GGUF embedding model (e.g. nomic-embed, bge, e5) loaded via llama.cpp, configured as a model alias.
- **Remote embedding API models** — provider-hosted embedding endpoints (OpenAI's `text-embedding-3-*`, Gemini's embedding models, etc.), configured as API models.

Hit `GET /v1/models` to see what's available. The combined catalog includes embedding models alongside chat models — pick the one you need by name.

For setting up remote providers (and choosing one that supports embeddings), see [API Models](/docs/features/models/api-models).

## Local vs remote — when to pick which

- **Local** is a fixed-cost workhorse. Once the model is loaded, embedding 10,000 chunks costs no per-token spend; only your CPU/GPU time. Good for offline processing, batch jobs, on-prem RAG.
- **Remote** is pay-per-token but typically higher quality at the top tier. Good when you need a single small model from a provider with strong recall, or when you don't want to manage a local embedding model.

You can mix and match: use a local embedding model for ingestion (cheap, bulk) and a remote one for query-time embedding (fewer calls, better recall) — or vice versa. Both go through the same wire format.

## Examples

### curl

```bash
curl -X POST http://localhost:1135/v1/embeddings \
  -H "Authorization: Bearer $BODHI_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "your-embedding-alias",
    "input": "The quick brown fox"
  }'
```

You can pass a single string or an array of strings as `input`. Batch sizes are bounded by the underlying model — see Swagger UI for limits.

### Python (OpenAI SDK)

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:1135/v1",
    api_key=os.environ["BODHI_TOKEN"],
)

resp = client.embeddings.create(
    model="your-embedding-alias",
    input=["First chunk of text", "Second chunk of text"],
)
vectors = [item.embedding for item in resp.data]  # list[list[float]]
```

The response shape is the standard OpenAI `Embedding` object (`data[].embedding`, `usage`, etc.). Vector dimensionality is whatever the underlying model produces — Swagger UI documents the response schema; the model card on the provider's side tells you the dimension count.

## Common gotchas

- **Chat model used as embedding model.** If you pass a chat model name, the request fails — embedding endpoints reject non-embedding aliases.
- **Mismatched dimensions.** If you swap from a 768-dim local model to a 1536-dim remote model in production, your existing vector store will reject the new vectors. Plan migrations carefully.
- **Batch limits.** Local llama.cpp has its own batch ceiling; remote providers have theirs. The error message tells you which side rejected.

## Full schema

See Swagger UI at `http://<your-bodhi-instance>/swagger-ui` for the request body, response shape, and provider-specific options. Default local URL: `http://localhost:1135/swagger-ui`.
