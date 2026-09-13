---
title: 'Models, Aliases, and Files'
description: 'The three things "model" can mean in Bodhi — file vs alias vs API model — and how they relate'
order: 2
---

# Models, Aliases, and Files

When you say "use model X" in Bodhi, that single word can refer to three completely different objects. Mixing them up is the #1 source of confusion for new users. This page lays them out cleanly.

## The three concepts

```
   ┌─────────────────────────────────────────────────────────────┐
   │  1. Model File                                              │
   │     A *.gguf binary on disk in your HuggingFace cache.      │
   │     Lives at ~/.cache/huggingface/hub/...                   │
   │     Owned by HuggingFace tooling; Bodhi just reads it.      │
   └────────────┬────────────────────────────────────────────────┘
                │  referenced by
                ▼
   ┌─────────────────────────────────────────────────────────────┐
   │  2. Model Alias                                             │
   │     A named recipe: file + chat template + inference params │
   │     (temperature, top_p, server flags, etc.)                │
   │     Stored in $BODHI_HOME/aliases/*.yaml                    │
   │     This is what you type in the chat "model" picker        │
   │     and what you put in OpenAI's `model` field.             │
   └─────────────────────────────────────────────────────────────┘

   ┌─────────────────────────────────────────────────────────────┐
   │  3. API Model                                               │
   │     A configured remote provider (OpenAI / Anthropic /      │
   │     Gemini / Groq / OpenAI-compatible) plus the list of     │
   │     models from that provider that you've made available.   │
   │     Stored in the database (encrypted API keys).            │
   │     Each remote model also surfaces in the chat picker.     │
   └─────────────────────────────────────────────────────────────┘
```

Concepts 1 + 2 are about **local inference** (llama.cpp). Concept 3 is about **remote proxying** (forwarding to a third-party API). All three appear together in your model dropdown, and Bodhi picks the right path based on which kind matched.

## 1. Model file

A model file is a `.gguf` file — a quantized weights blob. Bodhi doesn't host or version these; it reads them from your HuggingFace cache (`~/.cache/huggingface/hub/...` on macOS/Linux; the equivalent path on Windows). When you click "Download" in the Models page, Bodhi delegates to HuggingFace's library to fetch the file into that shared cache.

Key facts:

- **Identity:** `{repo_id}:{filename}`, e.g. `microsoft/Phi-3-mini-4k-instruct-gguf:Phi-3-mini-4k-instruct-q4.gguf`.
- **Storage:** outside `$BODHI_HOME` — it's in the shared HF cache, so other tools (`llama.cpp`, `ollama`, `lmstudio`) can use the same file.
- **Cannot be configured.** A file is a file; if you want different behavior, make a different alias.

A file by itself isn't usable in chat — Bodhi needs to know how to format prompts and which inference parameters to pass to llama.cpp. That's what an alias is for.

## 2. Model alias

A **model alias** is a small YAML record that says: _"When someone asks for `my-fast-phi`, load this file with these parameters."_

Example (lives under `$BODHI_HOME/aliases/my-fast-phi.yaml`):

```yaml
alias: my-fast-phi
repo: microsoft/Phi-3-mini-4k-instruct-gguf
filename: Phi-3-mini-4k-instruct-q4.gguf
chat_template: phi3
request_params:
  temperature: 0.7
  top_p: 0.9
  max_tokens: 1024
context_params:
  - '--ctx-size'
  - '4096'
```

Two flavors of model alias exist:

- **User alias** — created by you (or an admin) via the Aliases page. Editable, renameable, deletable.
- **Model alias** (built-in) — the chat-model entries that show up automatically once a file is downloaded. These are read-only convenience entries; if you want different params, copy them into a User alias and edit there.

Aliases are what the chat picker actually shows, and what you put in the `model` field of an OpenAI-compatible request. Aliases are also tenant-scoped: multiple users on the same Docker instance can share aliases (admins control creation).

## 3. API model

An **API model** is a configuration record that points Bodhi at a remote provider:

- **Provider** — OpenAI, OpenAI Responses, Anthropic, Anthropic OAuth, or Gemini. (Other OpenAI-compatible providers like Groq, OpenRouter, Together AI, or HuggingFace Inference are configured via the OpenAI template with a custom `base_url`.)
- **Credentials** — an API key (encrypted at rest), or for Anthropic OAuth, an OAuth-issued token bundle.
- **Models** — the list of remote model IDs you've enabled for chat. Bodhi fetches the catalog from the provider so you can pick which ones to expose.
- **Optional prefix** — prefix the remote model IDs with a tag (e.g. `oai/gpt-4o`) to disambiguate when multiple providers share names.

When the chat or API request hits Bodhi with an API-model `model` value, Bodhi forwards the request to the provider, transparently translating headers (`Authorization` → `x-api-key` for Anthropic, `x-goog-api-key` for Gemini, etc.) and request shape if necessary.

API models are managed in the **API Models** page. PowerUser+ can create, edit, and test them; an admin "Test connection" round-trip and "Fetch models" catalog discovery are built in.

## How they appear together

In the chat model picker (and in `GET /v1/models`), Bodhi merges all three sources:

- Local model aliases (User + Model aliases) — call llama.cpp under the hood.
- API-model entries — proxy to a remote provider.

You can switch between them mid-conversation. Streaming, tool calling, and the rest of the OpenAI-compatible feature set work for both.

## Common confusions

- **"I deleted the alias but the file is still there."** Correct — aliases and files are independent. Delete the file from your HuggingFace cache directly if you want disk space back.
- **"I added an API key but no models show up."** Click "Fetch Models" after entering the key; until you select models, nothing is exposed for chat.
- **"My alias name conflicts with a remote model name."** Use the API-model **prefix** option to namespace remote models (e.g. `oai/`).
- **"Where is my alias YAML on Docker?"** Inside the container at `$BODHI_HOME/aliases/`. Mount that directory into a volume to persist across recreates.

## Where to go next

Forward references (these pages land in a later phase):

- `/docs/features/models/overview` — the Models UI tour.
- `/docs/features/models/model-files` — file management and downloads.
- `/docs/features/models/model-alias` — creating and editing aliases.
- `/docs/features/models/api-models` — full API-model configuration walkthrough.
- `/docs/features/models/anthropic-oauth` — using your Claude.ai or Console account directly, no API key.

Or jump up the stack to **[API Compatibility](/docs/concepts/api-compatibility)** to see how these models are exposed via OpenAI/Anthropic/Gemini-shaped endpoints.
