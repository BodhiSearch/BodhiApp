---
title: 'Overview'
description: 'Tour of the Models section: what each sub-page is for, and where the model file vs alias vs API model distinction lives in the UI'
order: 0
---

# Models — Overview

The Models section is where you manage everything that can answer a chat request: locally-run GGUF models and remote API providers. This page is a roadmap. If you are new to Bodhi, read **[Models, Aliases, and Files](/docs/concepts/models-aliases-files)** first — it lays out the three concepts that the rest of these pages assume you know.

## The three concepts (recap)

Bodhi treats three different objects as a "model":

1. **Model file** — a `.gguf` weights file on disk (in your HuggingFace cache). Bodhi reads it; HuggingFace tooling owns it.
2. **Model alias** — a named recipe that bundles a file with chat template and inference parameters. This is the value that goes in the `model` field of an OpenAI-shaped request.
3. **API model** — a configured remote provider (OpenAI, Anthropic, Anthropic OAuth, OpenAI Responses, or Gemini), plus the list of remote models you have exposed for chat.

Files and aliases drive **local inference** through llama.cpp. API models **proxy** requests to a third-party service. The chat picker merges all three sources into one list.

## Where things live

| Concept                    | UI route                                          | API route                     | Doc                                                      |
| -------------------------- | ------------------------------------------------- | ----------------------------- | -------------------------------------------------------- |
| Models page (unified list) | `/ui/models/`                                     | `GET /bodhi/v1/models`        | [Model Aliases](/docs/features/models/model-alias)       |
| Model alias (create/edit)  | `/ui/models/alias/new/`, `/ui/models/alias/edit/` | `/bodhi/v1/models`            | [Model Aliases](/docs/features/models/model-alias)       |
| Model files                | `/ui/models/files/`                               | `/bodhi/v1/models/files`      | [Model Files](/docs/features/models/model-files)         |
| Model downloads            | `/ui/models/files/pull/`                          | `/bodhi/v1/models/files/pull` | [Model Downloads](/docs/features/models/model-downloads) |
| API model (create/edit)    | `/ui/models/api/new/`, `/ui/models/api/edit/`     | `/bodhi/v1/models/api`        | [API Models](/docs/features/models/api-models)           |
| Anthropic OAuth provider   | `/ui/models/api/new/` (format selector)           | `/bodhi/v1/models/api`        | [Anthropic OAuth](/docs/features/models/anthropic-oauth) |

The Models page (`/ui/models/`) is the central hub. It shows aliases and API models in one sortable table with a Source badge that tells you which kind of entry each row is.

<img
  src="/doc-images/models-page.jpg"
  alt="Models page with unified list of aliases and API models"
  class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%] mx-auto block"
/>

## Pick the right page for your task

- _"I want to chat with a local model."_ → Download the file via [Model Downloads](/docs/features/models/model-downloads), then start chatting using the auto-generated model alias, or create your own via [Model Aliases](/docs/features/models/model-alias).
- _"I want to use OpenAI / Gemini / Anthropic from inside Bodhi."_ → [API Models](/docs/features/models/api-models) walks through provider setup, fetching models, and the test-connection check.
- _"I have a Claude.ai or Anthropic Console subscription and want to skip API keys."_ → [Anthropic OAuth](/docs/features/models/anthropic-oauth) covers the OAuth-token flow.
- _"I want to free up disk space."_ → [Model Files](/docs/features/models/model-files) shows which GGUFs are cached locally and how to remove them.
- _"I want to see what's available without configuring anything."_ → Open `/ui/models/` after install — Bodhi auto-creates a model-file alias for every downloaded GGUF, so chat works the moment a download finishes.

## What you can and cannot configure here

- **Configurable**: alias names, llama.cpp context flags, default request parameters (temperature, top_p, stop sequences, etc.), API model base URLs, prefixes, extra headers and body fields, which models to expose from a provider.
- **Not configurable from these pages**: chat history, MCP tools, role assignments, server-wide settings. Those live under [Auth](/docs/features/auth/overview), [MCPs](/docs/features/mcps/overview), and [App Settings](/docs/features/settings/app-settings).

## Choosing local vs remote

Every workflow can mix local and API models in the same chat or API call — there is no setting to flip. Pick local when you need privacy, offline access, or cost predictability. Pick remote when you want the latest frontier capability or when local hardware is the bottleneck. The chat UI presents both side-by-side; the Bodhi server decides whether to spin up llama.cpp or forward to a provider based on which entry the `model` field matches.

## Where to go next

- New to local inference? Start with [Model Downloads](/docs/features/models/model-downloads).
- Already have an API key? Jump to [API Models](/docs/features/models/api-models).
- Connecting external apps? See [Developer Guide → Building Apps](/docs/developer/building-apps).
