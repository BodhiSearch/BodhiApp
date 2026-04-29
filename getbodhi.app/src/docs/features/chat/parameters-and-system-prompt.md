---
title: 'Parameters and System Prompt'
description: 'Tune sampling — temperature, top-p, max tokens, stop words — and override the system prompt per conversation'
order: 2
---

# Parameters and System Prompt

The Settings panel on the right side of the chat UI is where you control how the model generates text. Every parameter has an **enable toggle** so you can opt in only to the ones you want to override; everything else falls back to the model's default.

This page explains each control, where the values come from, and how parameters map to the underlying provider APIs (OpenAI, Anthropic, Gemini, ...).

## How overrides work

Parameters are evaluated in this order, highest priority wins:

1. **Per-conversation override** — what's in the Settings panel right now (if its toggle is on).
2. **Model alias defaults** — values baked into the model alias on the Models page.
3. **Provider / GGUF defaults** — whatever the model itself ships with.

Toggling a control off doesn't reset its value; it just stops Bodhi from sending the field, letting layers below take over. This is the easy way to A/B a parameter without losing your tuned number.

## Where settings live

- **Global defaults** (sidebar starting state) are kept in browser `localStorage`. They apply to any new conversation you start.
- **Per-conversation overrides** are saved into the chat record itself in IndexedDB. Reopening an old chat restores the exact parameters it was running under.
- **Tool selection** is scoped per chat too — see [Tool Calling](/docs/features/chat/tool-calling).
- Changes to the Settings panel apply to **new messages**. Already-completed assistant turns are not regenerated.

Nothing is sent to the server until you send a message; settings never leak across users or browser profiles.

## Sampling controls

### Temperature

Controls randomness. Range: **0.0 – 2.0**.

- `0.0 – 0.3` — focused and deterministic. Good for code, factual answers.
- `0.4 – 0.7` — balanced. Default conversational territory.
- `0.8 – 1.2` — creative, more variety.
- `1.3 – 2.0` — highly random. Most providers degrade beyond ~1.5.

> **Anthropic note:** the Anthropic Messages API caps `temperature` at **1.0**, not 2.0. If you set a higher value while using an Anthropic model, the provider clamps or rejects it.

### Top P (nucleus sampling)

Range: **0.0 – 1.0**. Restricts sampling to the smallest set of tokens whose cumulative probability exceeds Top P.

- `0.1 – 0.5` — very focused vocabulary.
- `0.6 – 0.9` — balanced diversity.
- `0.95 – 1.0` — full vocabulary.

Top P and Temperature both work, and OpenAI's docs recommend tuning **one or the other**, not both. Bodhi happily forwards whichever you toggle on.

### Max Tokens

Maximum number of tokens the model may generate before it must stop.

- **Slider range in the UI:** 0 – 2048.
- Leave the toggle **off** to use the model/provider default — most providers default to a much higher cap, so unchecking this is the easiest way to allow long answers.
- Longer responses use more memory (local models) and cost more (API models).

> **Anthropic note:** Anthropic Messages requires `max_tokens` to be **explicitly set**. When you select an Anthropic model, Bodhi will use the alias default if you haven't enabled the slider — verify the alias has a sane value, or enable the slider here.

### Presence Penalty

Range: **-2.0 to 2.0**. Penalises any token that has already appeared, regardless of frequency. Positive values push the model towards new topics.

### Frequency Penalty

Range: **-2.0 to 2.0**. Penalises tokens by how often they've appeared. Positive values reduce repetition of common words and phrases.

> Negative values for either penalty _encourage_ repetition — useful for stylized output but rarely what you want for a conversation.

### Stop Sequences

Up to **4** custom strings that, when encountered in the output, immediately stop generation.

- Type each sequence and press **Enter** to add it as a chip.
- Click a chip to remove it.
- Common values: `\n\n` (paragraph break), `###` (markdown section), `Human:` (prevent the model from speaking on your behalf).

> Stop sequences fire **mid-stream**. If a stop string appears inside a JSON tool-call payload it will truncate the call — keep stop values out of the tool grammar your model uses.

### Seed

An integer that, on supported providers, makes generation reproducible across runs with the same inputs and settings. Toggle it off when you want fresh randomness on each call.

> Many remote providers treat `seed` as best-effort, not strict. Local llama.cpp models honour it deterministically.

### Max Tool Iterations

Caps the number of agentic tool-call rounds in a single turn. Default **5**, range 1–20.

If a model loops calling tools without ever producing a final answer, lowering this number forces it to wrap up. See [Tool Calling](/docs/features/chat/tool-calling) for the broader loop.

### Stream Response

Toggles streaming on or off. Streaming uses Server-Sent Events; turning it off makes Bodhi wait for the full response and return it in one shot. Useful for debugging when you suspect a streaming-related issue.

### API Token

Optional. If you paste a Bodhi API token here, the chat UI uses it to call `/v1/chat/completions` — handy for testing the same flow your programmatic clients will use. Leave blank to use your normal session cookie.

## System Prompt

A free-form text area that injects a system message at the top of the conversation.

- No length limit beyond the model's context window.
- Stored alongside the conversation, so an old chat keeps the system prompt it was created with.
- Toggling the system-prompt switch off omits the field entirely — the model uses its own default persona.

### Example system prompts

```
You are a helpful coding assistant. Provide concise Python examples
with brief inline comments. Prefer standard library solutions.
```

```
Respond in 2–3 sentences. Use simple language suitable for non-experts.
Avoid jargon unless explicitly asked.
```

```
You are a critical reviewer. For each user submission, flag at least
one risk and one concrete improvement. Do not be sycophantic.
```

### Tips

- Be **specific about behaviour**, not personality (`always show output before explanation` beats `be helpful`).
- Pin output **format** here if you need structured responses (`return YAML`, `wrap code in fenced blocks`).
- Use it to **steer tool use** when MCP tools are enabled (`prefer the search_docs tool before answering from memory`).
- Iterate. Different models react differently to the same prompt.

## How parameters map to provider APIs

Bodhi translates the same set of UI controls into provider-specific request fields. The high-level mapping:

| Bodhi setting     | OpenAI / Responses                        | Anthropic Messages               | Gemini              |
| ----------------- | ----------------------------------------- | -------------------------------- | ------------------- |
| Temperature       | `temperature` (0–2)                       | `temperature` (0–1)              | `temperature`       |
| Top P             | `top_p`                                   | `top_p`                          | `topP`              |
| Max Tokens        | `max_tokens` (or `max_completion_tokens`) | `max_tokens` (**required**)      | `maxOutputTokens`   |
| Presence Penalty  | `presence_penalty`                        | not supported (silently dropped) | not supported       |
| Frequency Penalty | `frequency_penalty`                       | not supported                    | not supported       |
| Stop Sequences    | `stop`                                    | `stop_sequences`                 | `stopSequences`     |
| Seed              | `seed` (best effort)                      | not supported                    | not supported       |
| System Prompt     | `system` message in `messages`            | top-level `system` field         | `systemInstruction` |

If a provider doesn't accept a field, Bodhi simply doesn't forward it; you won't see an error. Use the API Format label under the model picker to check which provider you're hitting before you spend time tuning a parameter the provider ignores.

## Reference configurations

Starting points to copy into the Settings panel:

**Creative writing**

```yaml
Temperature: 0.8
Top P: 0.9
Presence Penalty: 0.6
Frequency Penalty: 0.3
Max Tokens: 2048
```

**Technical / code**

```yaml
Temperature: 0.2
Top P: 1.0
Presence Penalty: 0.1
Frequency Penalty: 0.1
Max Tokens: 1024
```

**Balanced conversation**

```yaml
Temperature: 0.5
Top P: 1.0
Presence Penalty: 0.4
Frequency Penalty: 0.4
Max Tokens: 1500
```

## Related

- [Chat UI](/docs/features/chat/chat-ui) — main walkthrough
- [Tool Calling](/docs/features/chat/tool-calling) — Max Tool Iterations and the agentic loop
- [Models → Overview](/docs/features/models/overview) — where alias-level defaults live
- [Models → Anthropic OAuth](/docs/features/models/anthropic-oauth) — the Anthropic-specific auth flow
