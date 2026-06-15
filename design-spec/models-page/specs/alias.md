# Create local alias — functional spec & context

*Companion doc: `shared-primitives.md` (read first). Sibling: `models.md`.*

*Audience: Claude / AI coding agents first, developers second. Wireframe lives in `design/models-page/project/screens/alias.jsx`. Four width variants: `AliasStandalone`, `AliasOverlay`, `AliasMedium`, `AliasMobile`, exposed via `window.AliasScreens`.*

---

## 1. What this page is

The flow for creating a **local alias** — a user-named config bound to a downloaded (or to-be-downloaded) GGUF file with llama.cpp runtime params. This is the llama.cpp side of the "model-shaped thing" universe; the API-model / provider side has its own flow (`screens/api.jsx`).

The output of this flow is one row in the Bodhi DB alias table, which then appears as an `alias` kind row on the Models page.

---

## 2. Section structure (the most important decision)

The form is divided into **four collapsible sections**, in this exact order:

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Identity                                                 │
│    · alias name, description, tags                          │
├─────────────────────────────────────────────────────────────┤
│ 2. Model file                                               │
│    · HF repo + QuantPicker + snapshot                       │
│    · FitCheckCard ("fits your rig · ~38 t/s expected")      │
│    · DownloadProgressStrip (if quant not yet downloaded)    │
├─────────────────────────────────────────────────────────────┤
│ 3. Preset & Runtime args                    [MERGED]        │
│    · PresetGrid (18 tiles)                                  │
│    · ArgsEditor (plain textarea, --flag value lines)        │
│    · ArgsPalette (click → append flag from llama-server --help)│
├─────────────────────────────────────────────────────────────┤
│ 4. Request defaults                                         │
│    · temperature, top-p, stop, system prompt                │
└─────────────────────────────────────────────────────────────┘
```

### Hard rules about this structure (DO NOT CHANGE without explicit user approval)

- **Exactly four sections, in this order.** The numbering is load-bearing — the `AliasRail` sticky nav and the `AliasMediumAnchors` jump list index them.
- **Section 2 owns the file.** Filename, quant, snapshot all live here. Nothing about the file leaks into Section 1.
- **Section 3 is the merged preset + args.** Not two sections. See §5 for the rationale.
- **Section 4 is request defaults only** — the stuff that applies at chat-request time (sampling temp, system prompt), not at server-startup time (`--n-gpu-layers`, `--ctx-size`).

---

## 3. Section 1 — Identity

Fields:
- **alias name** (required, unique)
- **description** (optional, short)
- **tags** (optional, chip input)

What was **removed** from this section during design:
- **snapshot** — moved to Section 2 (it's a property of the file, not the identity)
- **family / capability badges** — now computed from the underlying model file in Section 2 + preset in Section 3

Keep Section 1 small on purpose. Users name things last as often as first; don't block them with too many identity fields.

---

## 4. Section 2 — Model file

### QuantPicker IS the file selector

The earlier design had a separate **Filename** text input + a **Quant** dropdown. Removed — they were double-entry for the same decision. Now:

- `QuantPicker` is a grid of chips over the selected repo's quants (e.g. `Q3_K_M · Q4_K_M · Q5_K_M · Q6_K · Q8_0`).
- Clicking a quant chip selects the file — that is, `owner/repo:quant` becomes the filename.
- No separate Filename input. Do not re-add one. If a user asks "where do I type the filename?" the answer is: the quant grid.

### Snapshot placement

Snapshot (the HF git revision / commit SHA) lives **inside Section 2**, next to the quant grid. Rationale: the same `owner/repo:Q4_K_M` can point to different snapshots; it's a property of the file, not the alias name. Keep it here.

### FitCheckCard

Below the quant grid, a live "fits your rig" indicator (`FitCheckCard` primitive). Shows: expected t/s, GPU layer count that fits, fit quality chip (green / yellow / red). Updates as the user picks a quant.

### DownloadProgressStrip

If the selected quant is not yet downloaded, a `DownloadProgressStrip` appears showing streaming progress. The alias can be created before the download completes — download is async, alias creation is immediate.

---

## 5. Section 3 — Preset & Runtime args (MERGED)

This section is where the biggest design churn happened. The final shape:

### 5.1 Why they were merged

Originally two sections (Section 3 = Presets, Section 4 = Server-args). Merged because:

- **Feedback loop**: picking a preset should update the args textarea *live* so users see what changed. If they're in separate sections they have to scroll + mentally diff.
- **Custom state**: if users edit args manually, the preset chip row shows `Custom` automatically. Separate sections made this state confusing.
- **Cognitive one-ness**: "how should this model run" is one question. Preset is the quick answer; args are the full answer. Same decision.

Do not re-split them.

### 5.2 The raw-args pivot (why the editor is a textarea)

The earliest design had one typed input per llama.cpp flag (`ctx_size` as a number input, `n_gpu_layers` as a slider, …). Rejected after user feedback:

> "we have found keeping this form in sync with llama.cpp is challenge, hence right now we just take in the raw params separated by newline"

Keeping a typed form in sync means:
- Every llama.cpp release could add/rename/deprecate flags
- We'd be on a maintenance treadmill we didn't want
- Power users already know the flag names; a typed form slows them down

The raw-args editor gives up on owning the shape and instead **helps without owning**:

- Plain textarea. One `--flag value` per line.
- `ArgLine` rendering: flag in one span, value in another, hover-tooltip with help text from parsed `llama-server --help`, wavy underline on unknown flags (`args-line-warn` class).
- `ArgsPalette` on the right: scrollable list of known flags from `ARGS_HELP`. Hover shows `+ append` affordance. Click appends a skeleton `--flag <value>` line.
- No validation beyond "is this flag known to our parsed --help".

### 5.3 The preset catalogue (18 entries — exact)

Source of truth: `PRESET_CATALOGUE` in `primitives.jsx`. Do not remove entries without explicit user discussion — each maps to a validated user intent.

1. **Default** — sensible baseline
2. **Chat** — conversational (short replies, low latency)
3. **Coding** — code generation (large ctx, deterministic sampling)
4. **Agent** — tool-use (structured output, parallel call support)
5. **Reasoning** — chain-of-thought (high ctx, longer gen, moderate temp)
6. **RAG (short docs)** — optimised for short retrieval context
7. **RAG (long docs)** — optimised for long retrieval context
8. **Vision** — VLM settings
9. **Embed** — embedding-model settings
10. **Max Performance (tok/s)** — throughput-first
11. **Max Context** — largest ctx that fits
12. **Parallel — Medium** — batched requests, moderate concurrency
13. **Parallel — Max** — batched requests, max concurrency
14. **Hardware Use — Medium** — balanced VRAM use
15. **Hardware Use — Max** — maximum VRAM use
16. **Long-ctx** — extended context window setting
17. **Small** — minimal footprint
18. **Custom** — auto-set when user edits args; cannot be selected directly

`Custom` is a **computed state**, not a user choice. When any preset is active and the user edits the args editor, the active preset chip flips to `Custom` automatically.

### 5.4 ArgsEditor component contract

File: `primitives.jsx` (window-exported).

Behaviour:
- One line = one arg pair: `--flag value`.
- Known flags render with a help tooltip on hover over the flag name.
- Unknown flags get `args-line-warn` class → wavy red underline, tooltip says "unknown flag · not in parsed --help".
- The "focused" line gets a caret (`args-caret`).
- Default lines (preset-provided, not user-edited) get a visual dim (`default` class).

```jsx
// Simplified ArgLine rendering (actual impl in primitives.jsx)
const ArgLine = ({line, spec}) => {
  const tooltip = spec
    ? `${spec.flag}...\n${spec.desc}...`
    : 'unknown flag · not in parsed --help';
  const flagCls = line.warn ? 'args-line-warn' : 'args-line-flag';
  return (
    <div className={`args-line${line.focused?' focused':''}${line.default?' default':''}`}>
      <span className={flagCls} title={tooltip}>{line.flag}</span>
      {line.value != null && <><span className="args-line-value"> {line.value}</span></>}
      {line.focused && <span className="args-caret"/>}
    </div>
  );
};
```

### 5.5 ArgsPalette component contract

Right-side companion to the editor. Lists known flags (grouped by category — sampling / server / gpu / batching). Hover any row shows `+ append`. Click appends a skeleton line to the editor.

This is the **primary discovery mechanism** — users who don't know flags browse here rather than reading docs. Do not hide it.

---

## 6. Section 4 — Request defaults

Fields:
- **temperature** (slider, 0.0–2.0)
- **top-p** (slider, 0.0–1.0)
- **stop** (chip input, repeatable)
- **system prompt** (textarea)

These apply at **chat-request time**, not at server-startup time. This is the boundary that separates Section 3 (runtime args = server startup) from Section 4 (request defaults = per-chat invocation).

If a user confuses the two (e.g. "why is my system prompt not taking effect?" — check Section 4, not 3), the boundary is the first place to look. Keep it clean.

---

## 7. Four width variants

| Variant | Use case | Key chrome |
|---|---|---|
| `AliasStandalone` | Full-page create flow (demo: all sections open) | `AliasRail` sticky nav on the left, all 4 sections stacked |
| `AliasOverlay` | Overlay-on-top-of-Models-page create | `OverlayShell` chrome, condensed sections |
| `AliasMedium` | Tablet-width | `AliasMediumAnchors` jump list at top, sections stacked |
| `AliasMobile` | Phone | `PhoneFrame`, single-column, sections stacked, each collapsible |

### Standalone-specific note

The standalone variant intentionally shows **all sections expanded** for wireframe demo purposes. Real product would default each section to collapsed except the active one. Do not "fix" this in the wireframe — the expansion is the demo.

### Mobile-specific note

Mobile shows sections stacked with independent collapse. The `AliasRail` left-nav is replaced by a top-of-page chip row that scrolls to the active section.

---

## 8. LiveConfigJson (the right-edge companion)

On desktop + medium variants, a `LiveConfigJson` panel shows the current alias config as JSON, live-updating as the user edits. Purpose:

- Developer transparency — users who will eventually copy/paste this into a config file see what they're building.
- Debugging aid — if the form state gets into a weird shape, the JSON makes it visible.

It is **read-only** in the wireframe. In production it could be made editable (bidirectional), but that is out of scope.

---

## 9. Decisions archive (context not in the wireframe)

AI agents picking this up should know these — the wireframe only shows the "after".

1. **Pivot from holistic form to raw args** — see §5.2. The biggest single design decision. The challenge of keeping a typed form in sync with llama.cpp's evolving flag set is explicitly why the raw-args editor exists. Do not propose re-typing.

2. **Snapshot moved from Identity to Model file.** Was originally in Section 1. Moved because snapshot is a property of the file. See §3 and §4.

3. **Filename field removed.** Was originally a text input next to the quant dropdown. Removed because the `QuantPicker` grid already selects the file. See §4.

4. **Presets and Server-args merged.** Was two sections. Merged to create a live feedback loop between preset selection and args editor. See §5.1.

5. **Preset catalogue expanded from ~5 to 18.** The 18 entries each map to a validated user intent. See §5.3 for the full list. Adding more is fine; removing requires user discussion.

6. **Custom preset is a computed state, not a user choice.** It flips in automatically when the user edits args. Do not add a `Custom` tile that users can pick directly; that breaks the computed-state model.

7. **Plain textarea, not a code editor.** `ArgsEditor` is rendered by React as spans per line, but the underlying UX is plain text. We do not bring in Monaco / CodeMirror / Prism. Rationale: this is a wireframe; adding a heavyweight editor would make the wireframe a hundred times heavier for no demo value.

8. **Hover tooltips + wavy underline for errors, no modal error dialogs.** Unknown flags get the wavy underline (`args-line-warn`) and the tooltip says "unknown flag". No blocking dialog. Rationale: users already know raw flags can be wrong; the feedback is inline and quiet.

9. **4 variants is intentional.** Standalone + Overlay + Medium + Mobile. Dropping one would lose coverage of a real deployment shape (standalone = first-run onboarding; overlay = in-flow from Models page; medium = tablet; mobile = phone).

---

## 10. Verification checklist (after changes to alias.jsx / primitives.jsx)

1. Reload wireframe. Navigate to `Create local alias` tab.
2. **Three variants** (standalone, overlay, medium, mobile) render without console errors.
3. **Section count = 4** in each variant. Section order matches §2.
4. **Section 2**: QuantPicker renders as a grid of quant chips. Snapshot is visible **in** Section 2, not Section 1.
5. **No Filename input** anywhere.
6. **Section 3** = single section with PresetGrid on top + ArgsEditor below (or side-by-side on standalone).
7. **PresetGrid has 18 tiles.** Spot-check presence of: `Default`, `Chat`, `Coding`, `Agent`, `Reasoning`, `RAG (short docs)`, `RAG (long docs)`, `Max Performance (tok/s)`, `Max Context`, `Parallel — Medium`, `Parallel — Max`, `Hardware Use — Medium`, `Hardware Use — Max`, `Long-ctx`, `Small`, `Custom`.
8. **ArgsEditor** renders lines as `[flag] [value]` with hover tooltip. Insert an unknown flag → wavy underline + "unknown flag" tooltip.
9. **ArgsPalette** renders on the right (standalone/medium) or collapsed (mobile). Click a flag → appends to editor.
10. **Picking a preset** updates the editor visibly (preset → args live sync).
11. **Editing args while a preset is active** flips the active chip to `Custom`.
12. **LiveConfigJson** on standalone/medium shows the current state as JSON, live-updating.

---

## 11. Out of scope / deferred

- **Monaco/CodeMirror** for the args editor. Deferred — see §9.7.
- **Bidirectional LiveConfigJson** (editable on both sides). Deferred.
- **llama.cpp flag auto-completion as you type.** The palette covers discovery; inline completion is a future enhancement.
- **Preset migration.** If the preset catalogue changes in a future revision, no migration path for existing aliases is defined. Out of scope for the wireframe.
- **Template alias spawning.** "Create alias based on existing alias X" — not wired here.
- **Multi-file aliases** (e.g. split-GGUF with shards). Current design assumes one file per alias.
