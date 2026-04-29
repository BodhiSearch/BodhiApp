---
title: 'Model Aliases'
description: 'Create, edit, and manage model aliases — the named recipes Bodhi uses to launch llama.cpp with the right file and parameters'
order: 10
---

# Model Aliases

A **model alias** is the named recipe Bodhi uses to launch llama.cpp: which GGUF file to load, which chat template to apply, what default request parameters to use, and which command-line flags to pass to the inference server. Aliases are what you type into a chat client's `model` field — local files alone aren't usable until an alias points at them.

If the difference between a _file_, an _alias_, and an _API model_ is fuzzy, read [Models, Aliases, and Files](/docs/concepts/models-aliases-files) first.

## The Models page

The Models page at `/ui/models/` lists everything Bodhi can answer with — local aliases and remote API models — in one sortable table.

<img
  src="/doc-images/models-page.jpg"
  alt="Models page with the unified list of local aliases and API models"
  class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%] mx-auto block"
/>

**Source badges** tell you which kind of entry each row is:

- **user** — a User Defined Alias you (or another admin) created.
- **model** — an alias auto-generated from a downloaded GGUF file. Read-only.
- **API** — a remote API model. See [API Models](/docs/features/models/api-models).

**Per-row actions**: Chat (jump straight to the chat UI with this model selected), Edit, Preview (capabilities, context window, architecture pulled from GGUF headers), Refresh metadata, New from Model (create a User alias pre-filled with this row's repo and filename), and an external link to the HuggingFace repo or provider URL. Hover over column values to copy them.

## Two flavors of local alias

### User Defined Alias

A YAML record under `$BODHI_HOME/aliases/`. You control the alias name, request parameters, and llama.cpp flags. Editable, renameable, deletable.

### Model File Alias

When you download a GGUF, Bodhi auto-creates a read-only alias named `{repo}:{quantization}` (for example, `QuantFactory/Meta-Llama-3-8B-Instruct-GGUF:Q8_0`). Capabilities, context size, and other metadata come from the embedded GGUF headers. Use this when you just want to chat without configuring anything; copy it to a User Defined Alias if you need to override parameters.

## Sample alias YAML

```yaml
alias: llama3:instruct
repo: QuantFactory/Meta-Llama-3-8B-Instruct-GGUF
filename: Meta-Llama-3-8B-Instruct.Q8_0.gguf
snapshot: 5007652f7a641fe7170e0bad4f63839419bd9213
context_params:
  - '--ctx-size 2048'
  - '--threads 4'
  - '--parallel 1'
  - '--n-predict 4096'
  - '--n-keep 24'
request_params:
  temperature: 0.7
  frequency_penalty: 0.8
  stop:
    - <|start_header_id|>
    - <|end_header_id|>
    - <|eot_id|>
```

**Field reference**:

- `alias` _(required)_ — unique name for this configuration; this is the value clients put in the `model` field.
- `repo` _(required)_ — HuggingFace repository ID.
- `filename` _(required)_ — the specific GGUF file in the repo.
- `snapshot` _(optional)_ — pin to a commit hash. Leave empty for the latest snapshot.
- `context_params` _(optional)_ — array of llama-server flags applied at process startup. One flag per array entry.
- `request_params` _(optional)_ — defaults applied to every request that does not override them: `temperature`, `top_p`, `frequency_penalty`, `presence_penalty`, `max_tokens`, `seed`, and up to four `stop` sequences.

For the full set of llama-server CLI flags, see the [llama.cpp server documentation](https://github.com/ggml-org/llama.cpp/tree/master/tools/server). Server-wide defaults that apply across aliases live under [App Settings](/docs/features/settings/app-settings).

## Creating or editing an alias

Open the form at `/ui/models/alias/new/` (or click "Edit" on a row in the Models page). You can also click "New from Model" on a model-file alias to pre-fill the form with that file's repo and filename.

<img
  src="/doc-images/model-alias.jpg"
  alt="Model Alias form: alias name, repo, filename, context parameters, and a collapsible request parameters section"
  class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%] mx-auto block"
/>

**Form fields**:

- **Alias** — unique identifier. Must not collide with any other alias or API-model ID.
- **Repo / Filename / Snapshot** — points at the GGUF file. The file must already be downloaded (see [Model Downloads](/docs/features/models/model-downloads)).
- **Context Parameters** — one llama-server flag per line in `--flag value` form.
- **Request Parameters** — collapsible section for the inference defaults listed above.

Save and the alias appears in the Models page immediately. The next chat or API request that uses the alias name launches llama.cpp with the configured flags (or reuses the running process if it is already up).

## How a request flows through an alias

When a request arrives with a `model` value that matches a **User Defined Alias**:

1. Bodhi launches the llama.cpp server with the alias's `context_params` if it is not already running for this alias.
2. The alias's `request_params` are merged in as defaults — explicit values in the request win.
3. The request is forwarded to llama.cpp.
4. The response (streamed or whole) flows back to the client.

For a **Model File Alias**, step 1 uses default server flags and step 2 is skipped. Either way, idle llama.cpp processes are torn down after the keep-alive window so resources free up between sessions.

## Performance notes

A few rules of thumb that show up repeatedly:

- **Threads vs memory** — more `--threads` can mean faster inference, but each thread costs RAM. Start at half your CPU core count.
- **Context size** — `--ctx-size` is a memory and load-time multiplier. 2048 is a safe baseline; raise it only when you actually need longer conversations.
- **Quantization** — Q4_K_M is small but loses some quality; Q8_0 is roughly twice the size with near-full-precision behavior. Test on representative prompts.
- **Stop sequences** — the right `stop` list prevents wasted generation when the model would otherwise keep producing template tokens.

For deeper tuning advice (variant selection, hardware-specific flags, concurrency), see the upcoming Advanced section.

## Common pitfalls

- _"My alias points at a file that isn't downloaded."_ — saving still succeeds; the failure surfaces only when chat tries to launch llama.cpp. Download the file first or remove the alias.
- _"Chat says 'model not found' even though I see the alias."_ — make sure the `model` field in the request matches the alias name exactly, including any prefix (API models only).
- _"I want to share aliases across machines."_ — copy the YAML files in `$BODHI_HOME/aliases/`. They are plain text and portable.

## Where to go next

- Need to download the file the alias points at? See [Model Downloads](/docs/features/models/model-downloads).
- Cleaning up disk space or auditing local files? See [Model Files](/docs/features/models/model-files).
- Configuring a remote provider instead of running locally? See [API Models](/docs/features/models/api-models).
