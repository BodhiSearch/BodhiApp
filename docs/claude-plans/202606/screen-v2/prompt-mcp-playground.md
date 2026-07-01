# Design prompt — Bodhi MCP Playground (feature-complete, phased)

> Paste this into claude.ai/design. It iterates on the existing **`Bodhi MCP Playground.html`**
> prototype. Reference screenshots of **MCP Inspector** (the official, feature-rich MCP web client)
> are attached so you can see the source experiences we want to adapt. **Do not copy Inspector's
> look** — it's a developer tool. Re-express each capability in Bodhi's calm, semi-technical voice
> using the existing Bodhi design system (the same theme, shell, rail, cards, typography, and
> tokens already in the playground and the other Bodhi MCP screens).

---

## Who this is for & the guiding principle

The playground is where a **mostly non-technical** Bodhi user explores what a connected MCP can
actually do — run its tools, try its prompts, read its data — **without writing code or reading
protocol internals**. Treat MCP jargon as something to translate, not surface.

The one principle that governs every screen:

> **Lead with a friendly, plain-language experience. Keep the raw technical view available but
> tucked away — collapsed by default, one click to reveal.** Never delete the technical layer; just
> make it optional.

The existing prototype already nails this with a **"Developer" toggle** in the header that reveals
a raw request/response view. Carry that same idea into every new capability you add: a human view
by default, a raw/JSON view a click away.

### Headline principle: readable responses

A close second only to the principle above — and arguably the single biggest reason a non-technical
user comes here:

> **Whatever an MCP sends back, render it the way a human would want to read it — never as raw,
> escaped JSON by default. Always keep the raw JSON one click away for anyone who wants it.**

MCP responses are messy in predictable ways: a tool typically returns **text** that is really
**markdown** or **multi-line, formatted prose**, and that text is usually **wrapped inside a JSON
envelope** (so a naive view shows `"# Title\n\n- item\n- item"` as one escaped line). The default
view must **unwrap and render** that content properly — markdown as formatted markdown (headings,
lists, tables, code blocks, links), plain multi-line text with its line breaks intact, structured
data as a clean key/value or table view, images as images. The same applies to **resource contents**
and **prompt-message previews**, not just tool results. In every case, offer a **"Raw" / "JSON"
toggle** (and a copy button) so the underlying payload is always reachable — but it is the
*secondary*, opt-in view, never the default.

## What Bodhi is (so you scope correctly)

- Bodhi only talks to MCP servers it **hosts and connects on the user's behalf**. So there is **no
  "enter a server URL" or "connect" step** in the playground — the user simply **picks one of their
  already-connected MCPs** and starts exploring. (The current landing screen — pick-an-MCP cards +
  an active-MCP switcher in the rail — is exactly right; keep it.)
- Connection setup, OAuth, and API-key auth all happen on **other** Bodhi screens. The playground
  **never** shows auth/login/connect flows. If an MCP isn't connected, it simply isn't pickable here.

## Reuse what already works

The current prototype is a strong base — **keep its structure and only extend it**:

- The **pick-an-MCP landing** and the **active-MCP switcher** in the rail.
- The **Overview** screen: connection status pill, the at-a-glance meta (endpoint / transport /
  authentication), and the "what you can do here" capability tiles.
- The **left rail** with an "Explore" section linking the capability areas.
- The **list + detail** layout inside each area (searchable list on one side, a focused detail/run
  panel on the other).
- The **auto-generated input form** with required/optional markers and per-field helper text.
- The **result area** with a human-readable view, a raw view, and (in Developer mode) the underlying
  request — presented as a small set of switchable views with status, timing, and a copy action.
- The **"Use in Chat" / "Send to Chat"** actions that hand a tool or prompt off to the chat surface.

Everything below is **additive**. Where the prototype already has something, refine it rather than
replace it.

---

# Build this PHASE BY PHASE

**Please deliver one phase at a time and pause for review between phases.** Earlier phases are the
common, high-value path; later phases are richer and rarer. Don't fold a later phase's complexity
into an earlier screen. Each phase should feel finished on its own.

---

## Phase 1 — Overview & Tools (the everyday path)

This is what the vast majority of users will touch, so it must be the most polished.

**Overview.** Refine the existing screen. Keep it scannable: is it connected, what is this MCP, and
what can I do with it (the capability tiles that also act as jump-offs into each area). Translate
protocol meta (endpoint, transport, auth kind) into calm, secondary, plain-language detail — present
but never shouting. Some MCPs won't expose every piece of meta; design graceful empty/omitted states
rather than blank fields.

**Tools** — the heart of the playground. A searchable list of the MCP's tools, each with a friendly
name and one-line description; selecting one opens a detail panel that:

- Auto-builds an **input form from the tool's schema** — sensible controls per field type (text,
  number, toggle, choice, etc.), clear required vs. optional treatment, and the field's own
  description as inline help. Keep a **Run** and a **Reset**.
- Surfaces, near the tool's title, small **behaviour hints** that MCP tools can advertise — e.g.
  whether a tool only **reads** vs. **changes things**, whether it's safe to **repeat**, whether it
  reaches **beyond this workspace**. These matter a lot to a cautious non-technical user (Inspector
  shows them as a row of read-only / destructive / idempotent / open-world chips). Present them as
  **friendly, reassuring labels** — lean toward words like "Read-only" / "Makes changes" rather than
  protocol vocabulary — and only show the ones a tool actually declares.
- After running, shows a **result** with a clear success/error state, how long it took, and a copy
  action. **Rendering the result readably is one of the most important things this whole playground
  does** (see the cross-cutting "Readable responses" principle below) — MCP tools return text that is
  frequently **markdown** or **multi-line/formatted text** (often wrapped in JSON), and the default
  view must render that **the way a human expects to read it** (headings, lists, paragraphs, tables,
  code blocks), not as an escaped one-line blob. It also needs to handle the wider **range of things
  a tool can return**: structured/tabular data as a tidy view, **images**, **links to resources**,
  and **multi-part results**. And, as today, a **raw** view and (Developer mode only) the underlying
  **request** — as quietly-switchable views, raw collapsed by default.
- Keeps the existing **"Use in Chat"** hand-off.

Also keep an **optional, collapsed-by-default "advanced" affordance** on the run panel for power
users to attach extra request metadata — hidden unless Developer mode is on, so it never clutters
the default experience.

---

## Phase 2 — Prompts, Resources & Templates (explore the rest of the MCP)

The other three standard capabilities, in the same list+detail rhythm as Tools.

**Prompts** — ready-made requests the MCP offers. A list of prompts; the detail panel **fills in the
blanks** (an argument form, same form treatment as Tools), then lets the user **preview the messages
the prompt produces** (rendered as friendly chat-style message bubbles by role) and **send it to
chat**. Keep raw/underlying views available but collapsed.

**Resources** — data the MCP can read. A list of resources with a plain title and a quiet
address/URI subtitle; the detail panel shows light meta (address, kind/type) and a **Read** action
that renders the contents in a **human-friendly view** (key/value for structured data, sensible
rendering for text/markdown/images) with the raw view one click away.

**Templates** (resource templates) — parameterised resources, i.e. resources with **blanks in their
address** (e.g. "a page, by its ID"). The detail panel should make the parameterised address legible,
offer a small **fill-in form** for the blanks, preview what it **resolves to**, then **resolve & read**
— reusing the same result rendering as Resources.

Across this phase, design **clear empty states** — an MCP may offer tools but no prompts, or
resources but no templates. The rail's capability links should reflect what's actually available
(e.g. counts, or a muted/disabled state when an area is empty) rather than leading users into dead
ends.

---

## Phase 3 — Live interactions (when the MCP talks back)

Richer MCPs don't just respond — they can **ask the user something**, **ask to use the AI**, **report
progress**, and **stream activity**. These are the standout moments; design them to feel safe and
in-control, never alarming. They appear in response to actions the user already took (e.g. running a
tool), so weave them into the existing flow rather than bolting on a separate console.

**Elicitation — the MCP asks the user for input mid-task.** Sometimes a tool needs more information
to continue, so the server sends back a small request for input. This is the single best
demonstration of the whole interactivity story, so make it shine:

- Render the server's request as a **friendly, auto-generated form** — exactly the same form-building
  you already do for tool inputs, driven by the schema the server sends (text, choices, toggles,
  numbers, with titles, help text, and validation limits like "pick 1–3").
- Give the user a clear, **three-way response**: provide the info and continue, **decline**, or
  **cancel** — with plain-language framing of what each does.
- Keep the **raw request** (the schema/JSON) available as a collapsed/secondary view next to the
  friendly form, for anyone who wants to see exactly what was asked.
- Make it obvious **which action triggered the request** and that the original task is **waiting** on
  the user's answer.

**Sampling — the MCP asks to use the AI.** A server can request an LLM completion as part of its
work. Present this as an **approval moment**, not an automatic action: show, in plain language, what
the server wants the AI to do, and let the user **approve or decline**. A pending-requests inbox with
an empty "nothing's waiting" state is the right shape. Raw details collapsed by default.

**Progress & long-running work.** Some tools take a while and report progress as they go. Show a calm
**in-progress state with progress** (and a way to know it's still working) that resolves into the
normal result when done — so a slow tool never looks frozen.

**Activity & logs (gentle, optional).** MCPs can emit log messages and notify when their data
changes. Surface this as a **quiet, secondary activity feed** the user can glance at or ignore —
clearly informational, not demanding attention, and comfortable to leave collapsed. A simple control
to set how chatty the logs are (verbosity) belongs here too. This is the most "developer-ish" of the
phase, so keep it the most understated.

---

## Out of scope for now (please don't build these yet)

To keep us focused on the common path, **leave these out** — they're niche, and we'll decide on them
later. If a natural seam for them appears, leave room, but **don't design them now**:

- **Roots** (telling the server which local folders it may access).
- **MCP "Apps"** (tools that embed their own custom UI).
- **Global request metadata** editor (key/value pairs attached to every request).
- **Async / background task** management (a separate task list with polling).

---

## Consistency pass (apply throughout)

- **One design system.** Everything must read as native Bodhi — same shell, rail, cards, buttons,
  pills, spacing, and theme (including light/dark) already used in the playground and the other MCP
  screens. No new visual language.
- **Plain language first, jargon on request.** Prefer human words; reveal protocol terms only in the
  Developer/raw views.
- **The Developer toggle is the master switch** for the technical layer — when off, raw views,
  request bodies, and advanced metadata affordances stay hidden everywhere.
- **Readable-by-default rendering** (the headline principle) applies *everywhere* a payload is shown
  — tool results, resource contents, prompt previews, and elicitation requests: render markdown /
  multi-line text / structured data for humans, with a raw-JSON toggle + copy always available but
  secondary.
- **Empty, loading, and error states** deserve the same care as the happy path — calm, helpful, and
  on-brand.
- If you notice **inconsistencies** in the existing prototype (a control, label, or pattern that
  drifts from the rest of Bodhi), please **fix them in passing** and call out what you changed.
