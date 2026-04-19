# Create API Model — functional spec & context

*Companion docs: `shared-primitives.md` (read first). Siblings: `models.md`, `alias.md`.*

*Audience: Claude / AI coding agents first, developers second. Wireframe lives in `design/models-page/project/screens/api.jsx`. Four width variants: `ApiStandalone`, `ApiOverlay`, `ApiMedium`, `ApiMobile`, exposed via `window.ApiScreens`.*

---

## 1. What this page is

The flow for configuring an **API model** — a user-named alias bound to an external inference provider (OpenAI, Anthropic, Groq, OpenRouter, etc.) with API-level settings (format, key, prefix, forwarding, which underlying models to expose).

The output is one row in the Bodhi DB api-model table (or multiple rows if the user selects multiple models), plus the provider connection record. Both then appear on the Models page — api-models as file-first `[api-alias]` rows, the connection as a `[provider-connected]` summary row.

API model creation is **simpler than local alias creation** because there's no runtime config and no llama.cpp flags. The wireframe reflects that: one flat form, not an accordion.

---

## 2. Form anatomy — flat, grouped by section heads

The form is **one vertical flow** with `h3`-style section heads and dashed dividers. No `ParamSection` accordions, no collapsible sections — every field is visible at once (except the conditional Model Selection block).

```
Create New API Model
Configure a new external AI API model.

── 1 · Provider connection ──────────────────────────
API format  [required]     [OpenAI — Completions     ▾]
Base URL    [required]     [https://api.openai.com/v1]
                           Enter the complete API endpoint URL…

☑ Use API key
  [••••••••••••••••••  👁]
  Your API key is stored securely

── 2 · Request routing ──────────────────────────────
☑ Enable prefix
  [openai/]
  Add a prefix to model names. Example: openai/gpt-4

Request forwarding mode
  ○ Forward all requests with prefix
  ● Forward for selected models only

── 3 · Model selection ──── (conditional) ──────────
Select which OpenAI models you'd like to use…

Selected Models (3)                          [Clear All]
[gpt-4-turbo ×] [gpt-5-mini ×] [gpt-5.3-codex ×]

Available Models            [Fetch Models] [Select All (2)]
🔍 [codex                                          ×]
☐ gpt-5.1-codex-mini
☐ gpt-5.1-codex-max
☑ gpt-5.2-codex
─────────────────────────────────────────────────────
[🔌 Test connection]              [Cancel] [Create API Model]
```

### Hard rules (DO NOT change without explicit user approval)

- **Exactly 3 sections, in this order.** Numbering `1 · Provider connection` / `2 · Request routing` / `3 · Model selection` is load-bearing. `ApiRail` and `ApiMediumAnchors` reference these anchors.
- **Flat layout, not ParamSection.** Alias uses collapsible sections because there are 4 of them with dense bodies; API create has 3 light sections — collapsing would add ceremony. Keep it flat.
- **"Use API key" and "Enable prefix" are toggles**, not always-on inputs. The checkbox affordance is intentional — production has it, and it communicates "this is optional" without hiding the input.
- **Forwarding mode is a RADIO**, not a dropdown. Two clear labels, visible options, active state obvious.
- **Model Selection is conditional** — hidden entirely when `Forwarding = "Forward all"`. See §4.

---

## 3. Section-by-section

### Section 1 — Provider connection

Fields:
- **API format** (required) — dropdown via `ApiFormatPicker`. Options from `API_FORMATS` constant in `primitives.jsx`:
  - `openai-responses` · OpenAI — Responses
  - `openai-completions` · OpenAI — Completions (default selection in the demo)
  - `anthropic-messages` · Anthropic — Messages
  - `anthropic-oauth` · Anthropic — OAuth
  - `google-gemini` · Google — Gemini
  - `openrouter` · OpenRouter
  - `hf-inference` · HuggingFace — Inference
  - `nvidia-nim` · NVIDIA — NIM
  - `groq` · Groq — OpenAI-compatible
  - `together` · Together AI
- **Base URL** (required) — text input; helper: *"Enter the complete API endpoint URL for your provider"*. Each format carries a `defaultBaseUrl` that the UI should auto-fill; the user can override.
- **API key** — `ApiKeyField` primitive. "☑ Use API key" checkbox + masked input + 👁 eye toggle. When the toggle is off, the input is disabled (greyed). Helper under the input: *"Your API key is stored securely"*.

### Section 2 — Request routing

Fields:
- **Model prefix** — `PrefixField` primitive. "☑ Enable prefix" checkbox + text input (default `openai/`) + helper: *"Add a prefix to model names (useful for organization or API routing). Example: `openai/gpt-4`"*.
- **Request forwarding mode** — `ForwardingModeRadio` primitive. Two options:
  - `○ Forward all requests with prefix` — any request with the prefix gets forwarded to this provider, regardless of the downstream model.
  - `● Forward for selected models only` — only the explicitly selected models get forwarded. Enables Section 3.

### Section 3 — Model selection (CONDITIONAL)

Only visible when `Forwarding = "Forward for selected models only"`.

Components:
- **Caption** — *"Select which OpenAI models you'd like to use. Only the selected set will be forwarded through the alias prefix."*
- **`ModelMultiSelect` primitive** wraps:
  - **Selected Models (N)** header with `Clear All` link (only shown when N > 0).
  - **Selected chips strip** — `[model-name ×]` chips, × removes. Monospace font on the model name to signal identifier-ness.
  - **Available Models** header with `Fetch Models` + `Select All (N)` actions (N = unselected-filtered count).
  - **Search input** — filters the available list; `🔍` prefix, `×` clear affordance when filled.
  - **Available list** — scrollable, checkbox per model. Selected state highlighted with lotus-soft background.

### Footer

- **Left**: `🔌 Test connection` button (ghost / small).
- **Right**: `Cancel` + `Create API Model` (primary).

---

## 4. Conditional Model Selection — the one interaction rule

`Forwarding mode = "Forward all requests with prefix"` → **Section 3 is hidden entirely.** Not greyed, not collapsed — removed from the DOM.

`Forwarding mode = "Forward for selected models only"` → **Section 3 appears** with the full model picker.

### Why hidden, not greyed

`shared-primitives.md §5` says our convention is to **grey out** non-applicable filter groups to keep layout stability. We break that here, intentionally.

- A filter group (e.g. `Cost · api` in My Models) greys because it's a **fixed concept** in the filter taxonomy — hiding it would mean the sidebar jumps around and users lose their bearings.
- Section 3 is a **form-specific affordance** that has no meaning in "forward all" mode. Greying it would imply the user should think about it once they "understand the mode" — which is the wrong mental model. "Forward all" means "no per-model selection needed"; hiding Section 3 makes that obvious without words.

If you're tempted to grey it: don't. If you're tempted to show a "selected models N/A in Forward All mode" caption: also don't. The hidden state is the point.

---

## 5. Four variants

| Variant | Chrome | Demo state |
|---|---|---|
| `ApiStandalone` | Full page, `ApiRail` sticky nav on the left | Forward for selected, 3 preselected, `codex` search filter active |
| `ApiOverlay` | `OverlayShell` chrome (reached from Models' `+ ▾ Add model → Add API provider`) | Same as Standalone |
| `ApiMedium` | `TabletFrame`, `ApiMediumAnchors` top | **Forward all** (short form, Model Selection hidden) |
| `ApiMobile` | Two `PhoneFrame`s | (1) Forward for selected (main); (2) Forward all (short form) |

### Why two mobile frames

To demonstrate the conditional behavior in a static wireframe. One frame shows the full form with model picker; the other shows the shorter no-selection form. Both share the same body renderer (`ApiMobileBody`); only initial state differs.

### Demo state matches production screenshot

`ApiStandalone`, `ApiOverlay`, and mobile frame (1) pre-fill:
- API format: `OpenAI — Completions`
- Base URL: `https://api.openai.com/v1`
- API key: toggle on, masked
- Prefix: toggle on, `openai/`
- Forwarding: `selected`
- Selected models: `gpt-4-turbo`, `gpt-5-mini`, `gpt-5.3-codex`
- Search: `codex` (filters available list to the 5 codex-matching entries)

This gives reviewers the same visual as the production screenshot (`/Users/amir36/Downloads/download (8).png`).

---

## 6. Primitives in `primitives.jsx`

All exported via the existing `Object.assign(window, {...})`.

| Primitive | Signature | Notes |
|---|---|---|
| `ApiFormatPicker` | `({value, onChange})` | Wireframe: Field-styled closed state only. Real popover/menu is implementation work. |
| `ApiKeyField` | `({enabled, value, masked, onEnabledChange, onValueChange, onToggleMask})` | Checkbox + Field + eye icon. Disabled state greys the inner field. |
| `PrefixField` | `({enabled, value, onEnabledChange, onValueChange, example})` | Same pattern as `ApiKeyField`. Example helper uses `<code>` tint. |
| `ForwardingModeRadio` | `({value, onChange})` | Hand-drawn radio dots; active option gets lotus-soft background + hard shadow. |
| `ModelMultiSelect` | `({selected, available, search, onSearch, onSelect, onDeselect, onFetch, onSelectAll, onClear})` | Pure controlled component — state lives in `api.jsx`. |
| `ApiRail` | `({active})` | Provider / Routing / Models anchors. Active computed from the current forwarding mode in `api.jsx`. |
| `ApiMediumAnchors` | `({active})` | Top-of-page chip strip for tablet variant. |
| `API_FORMATS` | `const` | 10 entries. Each `{code, label, defaultBaseUrl}`. |
| `FIXTURE_OPENAI_MODELS` | `const` | 12 entries. Used by `ModelMultiSelect` for the demo available-list. |

---

## 7. Decisions archive (context not visible in the wireframe)

1. **Option A kept; B + C dropped (2026-04-19).** The user confirmed preference for Option A's flat one-form UX. Variant B (stepper) and Variant C (provider-aware editor with cost-per-model + OAuth card) were rejected as speculative. If the OAuth lifecycle needs UI later, it lives in `ConnectedProviderPanel`, not here.

2. **Missing production fields added.** The original Option A was missing: "Use API key" toggle, "Enable prefix" toggle, Forwarding Mode as a radio (not dropdown), and the entire Model Selection UI. All of these were explicitly called out in the screenshot review and are now wired via the new primitives.

3. **Flat form, not accordions.** Create local alias uses `ParamSection` for 4 dense sections. API create has 3 light sections; accordions would add ceremony for no benefit. Section heads + dividers do the grouping job.

4. **Conditional Model Selection hides (not greys).** See §4. Breaks our filter-grey-out convention intentionally. This is the one behavioral detail future agents most commonly trip on.

5. **4-variant parity with alias.jsx.** Standalone / Overlay / Medium / Mobile. The chrome is specific to the create-flow pattern and matches how alias.jsx handles the same responsive set.

6. **Primitives in primitives.jsx (not inline).** Follows the pattern alias.jsx + discover.jsx established. Primitives are exported via the single `Object.assign(window, {...})` call; order in the object matters only for readability.

7. **Static demo state covers both forwarding modes.** Standalone / Overlay / Mobile-1 demo `selected`; Medium / Mobile-2 demo `all`. Ensures every reviewer sees the conditional behavior without needing to click anything.

8. **Pre-filled search `codex` + selected models match the screenshot.** Chosen deliberately so the wireframe looks like the reference image the user provided.

---

## 8. Verification checklist

1. Reload `http://localhost:8000/`. Tab: `Create API model`.
2. **Variant deck** shows 4 variants in this order:
   - A · Standalone · full page
   - A · Overlay
   - A · Medium · tablet
   - A · Mobile
3. **Standalone variant**:
   - 3 section heads: `1 · Provider connection`, `2 · Request routing`, `3 · Model selection`.
   - API format reads `OpenAI — Completions`; Base URL reads `https://api.openai.com/v1`.
   - "Use API key" checkbox checked; masked input visible with 👁.
   - "Enable prefix" checkbox checked; input reads `openai/`.
   - Radio: "Forward for selected models only" active.
   - Selected chips: `gpt-4-turbo` / `gpt-5-mini` / `gpt-5.3-codex`.
   - Available list filtered by `codex`: 5 entries (codex-latest, gpt-5.1-codex-mini, gpt-5.1-codex-max, gpt-5.2-codex, gpt-5.3-codex), with gpt-5.3-codex checked.
   - Footer: `🔌 Test connection` left; `Cancel` + `Create API Model` right.
4. **Toggle to "Forward all requests with prefix"** → Model Selection section disappears (0 `.api-models-card` in that variant).
5. **Toggle back to "Forward for selected"** → Model Selection reappears.
6. **Overlay variant** — `OverlayShell` chrome, same field state as Standalone.
7. **Medium variant** — `Forwarding = all` by default; Model Selection hidden; "Tap 'Forward for selected' to reveal" callout visible.
8. **Mobile variant** — 2 PhoneFrames; frame 1 Forward-selected, frame 2 Forward-all.
9. **No console errors** on reload.
10. **Cache-buster** at `?v=29` on all loaded scripts.

---

## 9. Out of scope / deferred

- **"Add API model from connected provider" dedicated screen.** The `+ ▾ Add model` menu has this entry; for now it routes to the same api form with provider pre-picked. Dedicated picker screen deferred.
- **Per-API-model override UI** (temperature defaults, system prompt defaults at the API-model level). Alias has this in its Section 4; api.jsx does not. Deferred — not clear yet where in the form it would live.
- **OAuth lifecycle** (`anthropic-oauth`, `google-gemini` OAuth tokens, expiry/re-auth). `API_FORMATS` lists these formats but the form assumes key-based auth. A future pass will handle OAuth in both `api.jsx` and `ConnectedProviderPanel`.
- **Real validation** on Base URL / API key format. Wireframe is visual only.
- **Test Connection action state** (success / failure / rate-limited). Button exists; no outcome UI.
- **Fetch Models async state** (loading spinner, error). Button exists; no state transitions rendered.
- **Per-provider capability hints** (e.g. "this provider doesn't support vision" → dim the `vision` capability filter). Not wireframed.
- **OpenAI Responses vs Completions format distinction**. Listed as separate formats; the form treats them identically. A real implementation would differ in request shape.
