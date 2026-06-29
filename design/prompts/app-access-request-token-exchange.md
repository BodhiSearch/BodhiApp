# App-Access-Request — Token Exchange (Permission Elevation)

## Goal
Support **token exchange to elevate permissions**. A 3rd-party app submits its **existing token** and requests **additional** permissions. The consent screen (`App-Access-Review.html`) must:
- highlight permissions **already granted** (and still allow **downgrading** them),
- show **newly requested** permissions that can be **approved, denied, or partially approved** (by selecting which specific permissions to grant),
- and — because app tokens are **immutable** — **issue a new token and invalidate the submitted one** on approval.

## Depends on
Builds on `app-access-request-allow-list-models-mcps.md` (the standalone listing checkboxes). Implement that first; this layers the exchange / elevation behavior on top.

---

## Core behavior
- The page supports two modes: a **new** access request (today's behavior) and an **upgrade** request (token exchange). Distinguish by whether a previous token was submitted.
- In **upgrade** mode:
  - **Previously-granted** permissions are highlighted (green "previously granted") and remain **fully editable** — the approver can still **downgrade** them.
  - **Newly-requested** permissions are highlighted (amber "new access" / "new").
  - Every grant is a toggle / selection; **Approve commits whatever is left enabled** → *partial approval* = switch off the new items you don't want (or downgrade granted ones).
  - The request is **additive** (an app can't ask for *less* than it holds), but the approver can always reduce.
- **Immutability / issuance**: approving the exchange **mints a new token** and **invalidates the submitted one**. Make this explicit (banner + button copy + the invalidated token id).

---

## Where the granted-vs-new highlighting applies (every layer)
- **List all models / List all MCPs** checkbox (granted vs newly requested).
- **Model inference** tier and **MCP connect** tier (All / Specific).
- **Per-category model slots** and the **models inside them** (granted chips vs newly added chips).
- **Per-MCP rows** (granted connection vs new connection).
- **Role / scope** (e.g. User → Power User).

---

## UI specifics
- **Exchange banner** at the top of an upgrade request: app tokens are immutable; approving issues a new token and invalidates the submitted one (show its id); the approver may approve / deny / partially approve and may still downgrade previously-granted items.
- **Cues**: green "✓ previously granted" pill + subtle green accent on granted items; amber "new access" / "new" pill + accent on newly requested items. Reuse the existing classes in `models/model-access-picker.css` (`.map-granted-pill`, `.map-new-pill`, and the granted/new row accents) — they already exist.
- **Action bar (upgrade)**: primary button **"Approve & issue new token"** with a line noting it invalidates `<prevTokenId>`; secondary **"Deny upgrade"**.
- **Action bar (new)**: unchanged from today ("Approve N resources" / "Deny & return to app").

---

## Data model (prototype)
- A `PREV_GRANT` object describing what the submitted token already holds: `listModels`, model inference tier + per-slot granted model ids, `listMcps`, connect tier + granted MCP ids, role.
- The form initializes to the **requested (elevated)** state; compare against `PREV_GRANT` to derive granted-vs-new at each layer.
- Suggested demo scenario (Research Copilot): previously had *List all MCPs* + specific inference on 3 slots, role User; now also requests *List all models*, adds a model to a slot, connects Exa + Notion, and elevates to Power User. This exercises new/granted at every layer.

---

## Reuse what exists
- `models/model-access-picker.jsx` — `ModelAccessPicker` already accepts `grantedMode` / `grantedIds` / `readOnly` and renders granted/new cues; `ListingToggle` already accepts `granted` / `isNew`. Reuse both.
- CSS pills/accents already in `models/model-access-picker.css`. Only the **exchange banner** and the **scenario switcher** need new styles (in `users/access-request.css`).

---

## Demo affordance (review aid)
This page doesn't use Tweaks. Provide a lightweight way to view both states for review — e.g. a `?mode=upgrade` URL param and/or a small, clearly-labeled top-bar scenario switcher ("New request / Upgrade"). It's a demo control, trivially removable for production; keep it **outside** the consent card.

---

## Files to touch
- `users/access-request-app.jsx` — add upgrade mode + `PREV_GRANT` + granted/new derivation + exchange banner + action-bar variants + scenario switcher.
- `users/access-request.css` — exchange-banner style + scenario-switch style.
- Reuse `models/model-access-picker.jsx` / `.css` as-is (already capable).
- **Do not modify** `App-Access-Review.html`.

---

## Out of scope (separate prompts)
- Reflecting the exchange on **App Tokens** — showing the old token as "exchanged" (no active toggle) with a link to its replacement (`?selected=<id>`), and a read-only "View full request". Track as a follow-up prompt.
- A read-only (`?mode=view`) view of an issued token.
- The third-party app's own in-app UI that fires the exchange.

---

## Acceptance criteria
- [ ] Upgrade mode shows previously-granted permissions with a green "previously granted" cue, still editable (can be downgraded).
- [ ] Newly-requested permissions show an amber "new" cue.
- [ ] Cues appear at every layer: listing checkboxes, inference/connect tiers, slot models, MCP rows, role.
- [ ] Partial approval works by toggling individual permissions off; **Approve commits the remaining set**.
- [ ] Approve copy makes the immutable swap explicit ("issue new token", invalidates `<prevTokenId>`).
- [ ] The new (non-upgrade) request flow is unchanged.
- [ ] A review affordance exists to view the upgrade state; the consent card stays clean.
- [ ] Page loads with no console errors.
