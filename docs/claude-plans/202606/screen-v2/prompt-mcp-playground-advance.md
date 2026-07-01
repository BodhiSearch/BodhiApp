# Design prompt — Bodhi MCP Playground · Live, interactive tool calls (elicitation, sampling, completion)

> Paste this into **claude.ai/design**. It extends an existing prototype, the **Bodhi MCP
> Playground**, to handle the three *interactive* things an MCP server can do **during a tool call**:
> ask the user a question (**elicitation**), ask to use the AI (**sampling**), and offer **autocomplete**
> on argument fields (**completion**). This prompt is **self-contained** — everything you need is here.
> Reference screenshots of the official **MCP Inspector** may be attached so you can see the raw source
> experience; **do not copy Inspector's developer-tool look** — re-express each capability in Bodhi's
> calm, semi-technical voice using the existing Bodhi design system.

---

## 0. TL;DR of what to build

Today, running a tool is one shot: fill a form → **Run** → see a result. We're adding the case where
the server **talks back in the middle of the call**. We're doing this the **Inspector way for v1**:
elicitation and sampling each get **their own page**; when a request comes in, focus moves to that
page, the user responds, and focus returns to Tools to show the final result. (We deliberately chose
the page-based model over an inline-in-the-result-area flow to keep the UI simple and stable for v1;
an inline flow may come later.)

Three new moments:

1. **Elicitation** — a tool pauses and asks the user to fill in a small form. This lives on a new
   **Elicitation page**. User submits (or declines / cancels); the tool then resumes and returns its
   real result back on the Tools page.
2. **Sampling** — a tool asks to use an LLM. This lives on a new **Sampling page**. The user sees
   what the server wants to ask the AI, **picks a model**, approves; Bodhi runs it against the user's
   own model, hands the answer back, and the tool finishes (result shown back on Tools).
3. **Completion** — as the user types into a **prompt argument** or a **resource-template** field,
   the server can suggest values (IDE-style autocomplete dropdown). This is *not* a page and *not* a
   pause — it stays inline on the Prompts / Templates screens.

**The core navigation pattern (elicitation & sampling):**

```
On Tools, user clicks Run
  → tool call starts; the server sends back a request
  → focus AUTO-SWITCHES to the Elicitation (or Sampling) page, with that request waiting
  → user responds (Submit/Decline/Cancel  ·  Approve/Decline)
  → focus AUTO-RETURNS to Tools, where the original run resumes and shows its final result
```

---

## Who this is for & the guiding principles

The playground is where a **mostly non-technical** Bodhi user explores what a connected MCP can
actually do — **without writing code or reading protocol internals**. These three features are the
most "alive" the playground gets, so they must feel **safe and in-control, never alarming**.

Carry these principles (already true elsewhere in the playground) into everything below:

1. **Plain language first, jargon on request.** Say "the server is asking you for some details," not
   "an `elicitation/create` request was received." Protocol words live only in the raw/Developer view.
2. **Readable by default, raw one click away.** Every payload (the request schema, the messages the
   server wants the AI to see, the final result) renders **human-first**; a small **Raw / JSON**
   toggle + copy is always available but secondary.
3. **A Developer toggle is the master switch** for the technical layer. When it's off, raw views,
   wire shapes, and JSON-mode toggles stay hidden.
4. **Nothing happens to the user's data or AI without an explicit, clearly-framed choice.** Approve /
   Decline / Cancel are always plain about what each does.
5. **One design system** — same shell, rail, cards, pills, buttons, spacing, theme (light + dark) as
   the rest of the playground. No new visual language.

---

## What Bodhi is (so you scope correctly)

- Bodhi only talks to MCP servers it **hosts and connects on the user's behalf**. The user picks one
  of their **already-connected** MCPs and explores it. There is **no "enter a server URL / connect"**
  step here.
- **Sampling is real in Bodhi**: Bodhi *is* an LLM host, so when a server asks to use the AI, Bodhi
  can genuinely run it against one of the **user's own models** (the same models they use in chat).
  Design it as a real action, not a stub.
- Auth, OAuth, and connection setup all happen on **other** Bodhi screens — never here.

---

## The base you're extending — the playground as it exists today

So you can match it exactly, here is the **current** structure (keep it; extend it):

**Layout** — a three-panel shell:
- **Left sidebar**: an MCP **instance picker** (server glyph + name + status dot, searchable
  dropdown) and a **capability nav** (the "Explore" section) linking **Overview / Tools / Prompts /
  Resources / Templates**, each with a count.
- **Right rail**: a searchable **list** of items for the current capability (e.g. the tools list),
  each a row with a friendly name, one-line description, and (for tools) a status dot.
- **Centre**: the selected item's **detail + run panel**.

**The tool detail/run panel (centre), top to bottom:**
- A **header**: wrench icon, the tool's friendly title, its `code-name` in mono, a small "Tool" tag,
  the description, and a row of **behaviour-hint chips** (see table below).
- An **Inputs card**: an **auto-generated form** built from the tool's input schema — one field per
  argument, with a label, a **required**/**optional** marker, the field's description as inline help,
  and a control matched to the type (text, number, toggle, choice…). A **Run tool** button and a
  **Reset** button sit below.
- A **Result panel** beneath it: an idle/empty state until the tool is run; a **Working…** spinner
  while running; then a **result** with a success/error status pill, a timing meta line
  (`200 OK · 234ms`), a **Result / Raw / Request** tab switcher, a copy button, and a readable
  rendering of whatever came back (markdown, multi-line text, structured data, images, resource
  links). A **"Use in Chat"** hand-off lives here too.

**Behaviour-hint chips** (already present, reuse exactly) — friendly labels derived from the tool's
declared annotations; only show the ones a tool actually declares:

| Declared | Chip label | Tone |
| --- | --- | --- |
| `readOnlyHint: true` | **Read-only** | safe (green) |
| `readOnlyHint: false` | **Makes changes** | warn (amber) |
| `destructiveHint: true` | **Can delete** | danger (red) |
| `destructiveHint: false` | **Non-destructive** | safe |
| `idempotentHint: true` | **Safe to repeat** | safe |
| `idempotentHint: false` | **Repeats add up** | warn |
| `openWorldHint: true` | **Reaches out** | info (indigo) |
| `openWorldHint: false` | **Stays in workspace** | muted |

**The visual vocabulary you must match** (reuse these patterns; names illustrate the existing system):
- **Cards**: `~14px` radius, 1px border, card background, `16–18px` padding.
- **Pills/chips**: `99px` radius, `11.5px` / 600 weight, 1px border, tone-coloured background.
- **Buttons**: primary is indigo-filled white text, `38px` tall; secondary is transparent with a
  muted border. Hover = slight brightness lift, `~120ms` transitions.
- **Status pills**: success = green bg/green text; error = red bg/red text.
- **Tone families**: **safe/green (leaf)**, **warn/amber (saffron)**, **danger/red**, **info/indigo**,
  **accent/pink (lotus)**, **muted/grey** — use consistently with meaning.
- **Type**: Inter for UI, JetBrains Mono for code/codenames.
- **Forms**: `38px` inputs, `~12px` radius, focus ring; required = small pink "required" pill, optional
  = faint "optional"; field description in muted `11.5px` below the input; invalid fields flash a red
  border.
- **Code/raw views**: mono, syntax-highlighted `<pre>` in a bordered, slightly-recessed surface, with
  a copy button.
- **List + detail** rhythm (searchable list/rail on one side, focused detail on the other) — both new
  pages reuse this exact rhythm.

Everything new below must look like it was always part of this.

---

# How these three features REALLY work (so you simulate them correctly)

This section is the protocol reality, in plain terms, so the prototype's simulation is faithful
instead of invented. (Drawn from the MCP spec, version `2025-06-18`, and verified against the official
"everything" test server.)

All three are **server-initiated requests that arrive *during* a tool call**, over the same
connection. The tool call doesn't return until the exchange completes. Mental model:

```
User clicks Run on Tools
  → Bodhi sends  tools/call
      ← server, mid-call, sends back one of:
          • elicitation/create     (asks the user for input)   → handled on the Elicitation page
          • sampling/createMessage (asks to use the LLM)        → handled on the Sampling page
        user responds; Bodhi returns the response to the server
      ← server finishes and returns the tool result
  → final result shown back on Tools
```

A single `tools/call` can involve **zero, one, or several** such round-trips before it finishes.
(Completion is different — a quiet autocomplete, not a pause; see §C.)

---

## A. Elicitation — "the server needs some details from you" (its own page)

**What it is.** Mid-tool-call, the server sends an `elicitation/create` request: a short **message**
("Please provide your contact information") plus a **`requestedSchema`** — a *flat* JSON-Schema object
describing the fields it wants. Bodhi turns that schema into a form, the user fills it, and Bodhi
returns the answer; the tool then resumes.

**The request the server sends** (raw shape, for the Developer/raw view):

```json
{
  "method": "elicitation/create",
  "params": {
    "message": "Please provide your contact information",
    "requestedSchema": {
      "type": "object",
      "properties": {
        "name":  { "type": "string", "title": "Full name", "description": "Your full name" },
        "email": { "type": "string", "title": "Email", "description": "Your email address", "format": "email" },
        "age":   { "type": "integer", "title": "Age", "minimum": 18, "maximum": 120 }
      },
      "required": ["name", "email"]
    }
  }
}
```

**The schema is deliberately simple** — build your form to exactly this restricted subset (nothing
nested, no arrays-of-objects):

| Schema | Render as |
| --- | --- |
| `string` | text input |
| `string` + `format: "email"` | email input, validate email shape |
| `string` + `format: "uri"` | url input |
| `string` + `format: "date"` | date picker |
| `string` + `format: "date-time"` | datetime picker |
| `string` + `minLength`/`maxLength` | text input, enforce length |
| `number` / `integer` + `minimum`/`maximum` | number input, clamp/validate range; honour `default` |
| `boolean` | toggle / checkbox; honour `default` |
| `string` + `enum` (optionally `enumNames`) | **single-select** dropdown; show `enumNames` labels, submit the `enum` value |
| `array` + `items.enum` + `minItems`/`maxItems` | **multi-select** (chips/checkbox group); enforce "pick N–M"; honour `default` |
| `oneOf: [{const, title}, …]` | titled single-select; show `title`, submit `const` |

Every field carries `title` (label) and `description` (inline help); `required` lists the mandatory
ones. **This is the same form-building you already do for tool inputs** — reuse it. The everything
test-server's elicitation exercises *all* of the above at once (name, agree-to-terms boolean,
string-with-default, email/uri/date formats, integer 1–100 default 42, number 0–1000 default 3.14,
single- and multi-select enums with "pick 1–3", titled enums, legacy `enumNames`) — use a rich,
many-typed schema like that in your simulation to show the form-builder off.

**The three ways the user can respond** (a *three-way* model — get it right):

| Action | Plain-language button | What it returns | When |
| --- | --- | --- | --- |
| **accept** | **Submit** (or "Provide & continue") | `{ "action": "accept", "content": { …filled values… } }` | user filled it in and confirmed |
| **decline** | **Decline** | `{ "action": "decline" }` (no content) | user explicitly says "no, I won't give this" |
| **cancel** | **Cancel** | `{ "action": "cancel" }` (no content) | user backs out / dismisses without choosing |

Make the **distinction between Decline and Cancel legible** in plain words: Decline = "I don't want to
answer this" (the tool hears a firm no); Cancel = "stop, never mind" (abandons the run). After the
user responds, focus returns to Tools and the tool **resumes** — returning its real result (or a
graceful "okay, stopped" if declined/cancelled).

### The Elicitation page layout

A **list + detail** page in the same rhythm as Tools:

- **Empty state** (no requests yet): a calm message — *"When a server needs information from you,
  it'll show up here."*
- **Recent Requests (inbox list)** on the list side: a chronological list of elicitation requests,
  newest first. Each row shows a short title (the request `message`, truncated), which MCP/tool it
  came from, a timestamp, and a status chip: **Waiting** (accent/pulsing), **Provided** ✓, **Declined**
  ⊘, **Cancelled**, or **Expired**. A small count badge reflects how many are **Waiting**.
- **Detail side** for the selected request:
  - A plain-language header: **"The server needs some information"** + the request `message`, and a
    clear note of **which MCP / which tool run** is asking and that the run is **waiting** on the answer.
  - The **auto-built form** (the schema table above), with required/optional markers and inline help.
  - A quiet safety line — *"Only share what you're comfortable with. Bodhi never asks for passwords
    here."*
  - The three actions: **Submit · Decline · Cancel**.
  - A collapsed **Raw ⌄** (Developer mode) showing the exact `elicitation/create` payload.
- A **resolved** request becomes read-only: it shows what the user submitted (or that they
  declined/cancelled) and stays in the history list.

**Safety framing (spec requires it):** make it obvious which MCP is asking; servers **must not** ask
for sensitive info (passwords, full card numbers) — frame field treatment so secrets-looking asks
don't feel coerced; always allow Decline/Cancel; never trap the user.

---

## B. Sampling — "the server wants to use the AI" (its own page)

**What it is.** Mid-tool-call, the server sends a `sampling/createMessage` request: it wants an LLM
completion as part of its work (e.g. "summarise this," "classify that"). The server has **no AI of its
own and no API keys** — it borrows the *client's* model. In Bodhi that's real: Bodhi runs it against
one of the **user's own models**. This is an **approval moment with a human in the loop**.

**The request the server sends** (raw shape):

```json
{
  "method": "sampling/createMessage",
  "params": {
    "messages": [
      { "role": "user", "content": { "type": "text", "text": "Summarise the notes below in 3 bullets." } }
    ],
    "systemPrompt": "You are a helpful assistant.",
    "modelPreferences": {
      "hints": [{ "name": "claude-3-sonnet" }, { "name": "claude" }],
      "intelligencePriority": 0.8, "speedPriority": 0.5, "costPriority": 0.3
    },
    "maxTokens": 100,
    "temperature": 0.7
  }
}
```

**What Bodhi returns** once the user approves and the model runs (raw shape):

```json
{
  "role": "assistant",
  "content": { "type": "text", "text": "• point one\n• point two\n• point three" },
  "model": "llama-3.1-8b",
  "stopReason": "endTurn"
}
```

### The Sampling page layout

A **list + detail** page in the same rhythm:

- **Empty state**: *"When a server wants to use the AI, its request will appear here for your
  approval."*
- **Recent Requests (inbox list)**: chronological list, newest first. Each row shows a short title
  (a snippet of what the server wants the AI to do), which MCP/tool it came from, a timestamp, and a
  status chip: **Waiting**, **Approved** ✓, **Declined** ⊘. Count badge = number **Waiting**.
- **Detail side** for the selected request — show, in plain language:
  1. **What the server wants the AI to do** — render `messages` + `systemPrompt` **readably** as a
     small chat-style preview (role-labelled bubbles, markdown rendered), **not** raw JSON. Surface
     `maxTokens` / `temperature` as quiet secondary meta. (`modelPreferences` are advisory *hints* —
     show them only in the Developer/raw view; Bodhi makes the final model choice.)
  2. **A model picker** — a dropdown of the user's available models. **Default it to the model the
     user most recently used in chat** (Bodhi persists the chat's selected model; pre-select that, and
     fall back to the first available model if none is stored). The user can **change** the model for
     this request. (Wiring note for the real build: the chat surface persists the last-used model —
     read that as the default; in the prototype just hard-code a small list like `llama-3.1-8b`,
     `qwen2.5-7b`, `gpt-4o-mini` with the first pre-selected.)
  3. **A clear two-way choice**:
     - **Approve & run** — Bodhi sends the prompt to the chosen model, shows a brief "Asking the
       AI…" state, then renders the **AI's answer inline** (markdown-rendered), and hands that answer
       back to the server so the tool can finish. **The question is genuinely submitted to the LLM** —
       design it as a real call with its own loading and result. Then focus returns to Tools for the
       tool's final result.
     - **Decline** — nothing is sent to any model; the server is told the user said no. Plain framing:
       *"The server won't be able to use the AI for this step."* (The spec models a declined sampling
       as an error like *"User rejected sampling request"* — show it as a calm "You declined," not a
       scary red failure.)
  4. A collapsed **Raw ⌄** (Developer mode) showing the exact `sampling/createMessage` payload.

**Human-in-the-loop, per spec:** the user **sees the prompt before it runs** and **sees the answer**.
Keep an **optional, Developer-only "review before send"** affordance — after the model answers, let a
power user tweak the text before it's returned to the server — but the default friendly path is
approve → run → answer flows straight back.

---

## C. Completion — "type-ahead suggestions for argument fields" (inline, no page)

**What it is.** A *quiet* helper, **not** a pause and **not** a page. When a user is filling in an
argument for a **Prompt** or a **Resource template**, the server can suggest valid values as they type
— exactly like IDE autocomplete. (The `completion/complete` request; it applies to **prompt
arguments** and **resource-template URI variables**, *not* ordinary tool inputs.)

**Request / response shape** (raw, Developer view):

```json
// request — fired (debounced) as the user types into the "language" field of the "code_review" prompt
{ "method": "completion/complete",
  "params": {
    "ref": { "type": "ref/prompt", "name": "code_review" },
    "argument": { "name": "language", "value": "py" },
    "context": { "arguments": { /* already-filled fields, for dependent suggestions */ } } } }
// response
{ "completion": { "values": ["python", "pytorch", "pyside"], "total": 10, "hasMore": true } }
```

(`ref` is either `{type:"ref/prompt", name}` or `{type:"ref/resource", uri}`.)

**Design it as:** an **autocomplete dropdown** under the field, appearing as the user types — a tidy
list of suggestions (cap the visible list; if `hasMore`, show a subtle "more matches as you keep
typing" hint; if `total` is known, you may show "3 of 10"). Keyboard-navigable (↑/↓, Enter, Esc),
mouse-clickable, debounced so it doesn't flicker. Suggestions are **advisory** — the user can ignore
them. For dependent arguments, earlier-filled values pass as `context` to refine later suggestions
(e.g. picking `language: python` narrows `framework` suggestions to `flask`). Silently do nothing if a
field has no completions. This is the **lowest-key** of the three.

---

# The navigation & focus behaviour (the heart of this v1)

This is the part that replaces an inline flow, so design it carefully.

**1. Rail placement.** Add **Elicitation** and **Sampling** as two more links in the existing
**Explore** section of the capability nav, alongside Overview / Tools / Prompts / Resources /
Templates. Each shows a **pending-count badge** that lights up (accent colour) when one or more
requests are **Waiting**, and is quiet/zero otherwise.

```
EXPLORE
  Overview
  Tools
  Prompts
  Resources
  Templates
  Elicitation   [1]   ← badge lights up when a request is waiting
  Sampling
```

**2. Auto-switch on request.** When a running tool triggers a request, **focus auto-switches** to the
Elicitation or Sampling page, landing on that pending request (selected in the inbox, detail open and
ready to act). Make the transition feel intentional, not jarring — a brief, calm cue that "the server
needs you over here" is welcome (e.g. the rail badge pulsing as focus moves).

**3. Auto-return on resolve.** The moment the user resolves the request (Submit/Decline/Cancel for
elicitation; Approve/Decline for sampling), **focus auto-returns to the Tools page**, to the original
tool run, which resumes and shows its final result (or the graceful declined/cancelled outcome). The
resolved request remains in that page's history inbox.

**4. While waiting, the user stays in control.** The rest of the playground stays usable — the user
can still navigate the rail manually. A pending request **parks on its page** until answered (it
doesn't disappear if the user clicks away); the badge keeps it discoverable. (Default model: one
request at a time; but because each page is an **inbox list**, if a tool fires several requests or
multiple runs are in flight, they queue in the list and the user works through them — each resolution
returns focus to the run it belongs to.)

**5. The Tools result while a request is outstanding.** On the Tools page, the originating tool run
shows a calm **"Waiting on you…"** state (with a one-click jump back to the relevant request, in case
the user navigated away), then resolves into the normal result once the exchange completes.

---

## States to design (each deserves the same care as the happy path)

- **Elicitation / Sampling empty states** — calm "nothing's waiting" copy.
- **Inbox with a pending request** — the waiting item stands out (accent), resolved items are quiet
  history.
- **Tool running, request outstanding** — the Tools result panel's "Waiting on you…" state + the
  auto-switch to the request page.
- **Elicitation: submitted / declined / cancelled** — three distinct, calm resolutions, then
  auto-return to Tools.
- **Sampling: asking the AI / answered / declined** — including the brief "Asking the AI…" sub-state
  while the model runs, then auto-return.
- **Error** mid-exchange (server errors after the user answers; model call fails) — a clear,
  non-alarming error shown on the Tools result, with the request history preserved.
- **Empty model list** (sampling but the user has no models) — a gentle "connect a model to let
  servers use the AI" pointer rather than a dead end.
- **Completion: no suggestions** — silently nothing; the field behaves like a plain input.

---

## Cross-cutting consistency (apply throughout)

- **One design system** — match the playground's shell, rail, cards, pills, buttons, spacing, and
  theme (light + dark). The new pages are just more list+detail screens in the existing language.
- **Readable-by-default everywhere** — the elicitation message, the sampling prompt preview, the AI's
  answer, and the final result all render human-first; **Raw / JSON** + copy always available but
  secondary, gated by the Developer toggle for wire-level shapes.
- **Plain language** — translate every protocol term. The user should never need to know the words
  "elicitation," "sampling," or "completion" unless they open the Developer view.
- **Reuse, don't reinvent** — the elicitation/sampling forms are the **same auto-form builder** as
  tool inputs; the AI-answer and final-result renderers are the **same readable renderers** already in
  the playground; the completion dropdown is a standard combobox in Bodhi's style; the two new pages
  reuse the existing **list + detail** layout.

---

## Out of scope (don't design these now)

- An **inline-in-the-result-area** flow (steps woven into the Tools result). We chose the page-based
  model for v1 on purpose; the inline flow may come later.
- **Async / background** variants (where the *client* runs the request as a background task and the
  server polls) — the everything server has "Trigger Async Elicitation/Sampling" tools, but ignore
  them; design only the **synchronous** flow above.
- **Progress for long-running tools** and an **activity/log feed** — separate live pieces, elsewhere.
- **Roots**, **MCP "Apps"** (tools with embedded UI), and **global request metadata** editing.
- **Audio / image** sampling content — design **text** messages and **text** completions; leave a
  natural seam but don't build the media cases.

---

## How to simulate it convincingly in the prototype

Since claude.ai/design has no live MCP server, **fake the round-trips** with timers, but make the
shapes faithful so the real wiring is a drop-in later:

- Add a couple of demo tools to the Tools list whose "run" triggers the flow:
  - **"Request your details"** → after a short delay, the tool's run enters **"Waiting on you…"**,
    focus **auto-switches to the Elicitation page** with a new **Waiting** request using a rich,
    many-typed `requestedSchema`. On **Submit**, wait ~600ms, then auto-return to Tools and show a
    success result like *"✅ Thanks — the server used the details you provided."* On **Decline /
    Cancel**, show the corresponding calm resolution.
  - **"Summarise with AI"** → the run enters "Waiting on you…", focus **auto-switches to the Sampling
    page** with a new **Waiting** request (a couple of `messages` + a `systemPrompt`). On **Approve &
    run**, show "Asking the AI…" ~800ms, render a canned markdown answer, then auto-return to Tools and
    show the tool's final result. On **Decline**, show the declined resolution.
  - A **Prompt** (on the Prompts screen) with an argument like `language` that demonstrates the
    **completion dropdown** as the user types (`py` → `python`, `pytorch`, `pyside`).
- Drive the model picker from a small hard-coded model list with the first item pre-selected (stand-in
  for "the user's last chat model").
- Seed each new page's inbox with **1 pending + a couple of resolved** demo requests so the list,
  the status chips, and the badge are all visible at rest.
- Keep every faked payload reachable via **Raw ⌄** / Developer view in the exact wire shapes shown in
  §A/§B/§C, so a developer can verify the simulation matches the protocol.

If helpful, ask for the attached **MCP Inspector** screenshots of the elicitation form, the sampling
approval form, and the completion dropdown — they show the raw source experience these re-express
(but don't copy their developer-tool look).

---

## Build order (pause between phases for review)

1. **Phase 1 — Elicitation page + the navigation pattern.** The new Elicitation page (inbox list +
   detail with the auto-built form + Submit/Decline/Cancel + safety framing), the rail link with
   pending badge, the Tools "Waiting on you…" state, and the **auto-switch on request / auto-return on
   resolve** behaviour. This establishes the whole page-based model; nail it first.
2. **Phase 2 — Sampling page.** The new Sampling page (inbox + detail: readable prompt preview, model
   picker defaulting to the last chat model, Approve & run → real LLM call → AI answer → back to the
   tool / Decline), reusing the same navigation pattern and the optional Developer "review before
   send."
3. **Phase 3 — Completion.** The inline autocomplete dropdown on prompt-argument and
   resource-template fields — debounced, keyboard-navigable, advisory, silent when empty.

Deliver one phase at a time and pause for review. Each phase should feel finished on its own and read
as native Bodhi.
