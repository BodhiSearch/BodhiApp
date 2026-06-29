# App-Access-Request — Allow Listing of All Models & MCPs

## Goal
Add two standalone **listing permissions** to the **`App-Access-Review.html`** consent screen:

- **Allow to List All Models**
- **Allow to List All MCPs**

Render each as a **checkbox** placed as the **first row** of its section — one in **Model access**, one in **MCP access** — above the existing per-resource rows (model slots / MCP rows).

This is the only change. Keep everything else on the page exactly as it is.

---

## Mental model (read first)
Listing and inference/connect are **separate, orthogonal permissions** — think S3 `ListBucket` vs `GetObject`:

- **List Models** (`ListBucket`) → the app can enumerate the full catalog via `/v1/models`.
- **Model access** (the existing per-slot *All / Specific* choice, `GetObject`) → which models the app can actually run inference on.

How they interact:
- **All Models** already exposes the whole catalog, so it implicitly covers listing.
- **Specific Models** + **List unchecked** → `/v1/models` returns **only** the specific models granted; all other models are invisible to the app.
- **Specific Models** + **List checked** → `/v1/models` returns the **full** catalog; each model's metadata carries a flag for whether inference is allowed (only the granted models are runnable).

The exact same relationship applies to **List MCPs** vs **MCP (connect) access** (`/v1/mcps`).

---

## UI requirements
1. **Model access section** — add a checkbox as the **first row**, before the model slots:
   - Label: **Allow to List All Models** with a mono endpoint hint `/v1/models`.
   - Sub-text: *"Standalone permission — the app can enumerate every model in the catalog. Off → it only sees the models granted for inference below."*
2. **MCP access section** — add the same as the **first row**, before the MCP rows:
   - Label: **Allow to List All MCPs** with `/v1/mcps`.
   - Sub-text: *"Standalone permission — the app can discover every MCP server. Off → it only sees the servers granted a connection below."*
3. It is a **checkbox** — not a switch, and **not** an option inside the All / Specific control.
4. One checkbox **per section** (section-level), not per model slot / per MCP row.
5. It is **fully independent**:
   - Do **not** fold it into the existing access control.
   - Do **not** auto-enable or lock it under any condition. It stays freely toggleable and is tracked as its own parameter.
   - Default **off**.
   - (Optional) A soft, non-blocking hint such as *"'All' already lists everything"* is fine where a section-level *All* is selected — but never change, check, or disable the checkbox because of it. Note: the current page uses per-slot All/Specific with no single section-level *All*, so this hint generally won't apply here — render the checkbox plainly.

---

## Reuse what already exists
A ready-made checkbox component is already in the codebase and is used by **New App Token** for exactly this — reuse it for visual + behavioral consistency:

- Component: **`ListingToggle`** in `models/model-access-picker.jsx` (exported to `window`).
- Styles: `.map-listing` / `.map-listing-check` in `models/model-access-picker.css` (already loaded by `App-Access-Review.html`).
- Props: `on`, `onToggle`, `readOnly`, `redundant` (soft "All already lists" hint), `label`, `desc`, `code`, `granted`, `isNew`. For this task only `on`, `onToggle`, `label`, `desc`, `code` are needed.

---

## Files to touch
- `users/access-request-app.jsx`
  - Add `listModels` / `listMcps` boolean state (default `false`).
  - Render `<ListingToggle …>` as the first child inside the **Model access** section (right after its `section-label`) and inside the **MCP access** section.
- No new CSS is required if `ListingToggle` is reused (its styles already ship in `models/model-access-picker.css`).
- **Do not modify** `App-Access-Review.html` (markup is script-driven).

---

## Out of scope (do NOT add)
- The token upgrade / exchange flow, "previously granted / new" highlighting, role cards, scenario switcher, or read-only view — none of that. This task is **only** the two listing checkboxes.
- Do not change **New App Token** or **App Tokens** (they already implement this pattern; match their look, don't edit them).

---

## Acceptance criteria
- [ ] Model access section shows **Allow to List All Models** (`/v1/models`) as its first row, as a checkbox.
- [ ] MCP access section shows **Allow to List All MCPs** (`/v1/mcps`) as its first row, as a checkbox.
- [ ] Both default to **off** and toggle independently of the All/Specific access controls.
- [ ] Neither checkbox appears as an option inside the All/Specific control.
- [ ] Selecting "All" never auto-checks or disables the listing checkbox.
- [ ] Copy makes clear it is a standalone listing permission, with the endpoint hint shown.
- [ ] Page loads with no console errors; the rest of the page is visually unchanged.
