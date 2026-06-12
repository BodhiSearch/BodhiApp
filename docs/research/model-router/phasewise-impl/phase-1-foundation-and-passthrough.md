# Phase 1 — Foundation & Pass-Through Routing

> Read [`README.md`](./README.md) first for scope, non-goals, and global invariants. Technical design context: proposal §1 (domain model), §2 (storage), §5 (resolution & routing hook), §8 (API surface), §9 (frontend).

## Goal
A user can **create, view, edit, and delete a model-router** alias from the UI, and a chat request addressed to that router is **forwarded end-to-end to its first enabled target**. No failover yet — with a single healthy target the behavior is indistinguishable from the finished feature. This phase proves the composite alias is a first-class alias: it persists, validates, lists, is selectable in chat, and forwards a real completion.

## Functional requirements

### Defining a model-router
- A model-router has a **unique name** (unique across *all* alias kinds — local, API, and other routers) and an **ordered list of targets**.
- Each **target** consists of:
  - a reference to an **existing alias** (a local model alias or a remote API-model alias);
  - a **pinned model** — the concrete model that will be requested from that alias;
  - an **enabled flag** (default enabled) that includes/excludes the target from the active sequence **without deleting it**.
- A target **may not** reference another model-router, and **may not** reference the router being created (no nesting, no self-reference).

### Model-pinning rules (per referenced alias type)
- Referenced alias is a **local** model alias → the pinned model is fixed (the alias's own model); the UI shows it, not a chooser.
- Referenced API alias exposes **all** provider models (route=all) → the pinned model is **free-text** (validated against the alias's prefix rule).
- Referenced API alias exposes a **selected subset** → the pinned model is chosen from a **dropdown** of that alias's available models.

### Validation (at create/update)
- Name is non-empty and unique across aliases.
- Every referenced alias **exists** and is a local or API alias (not a router).
- Every pinned model is **valid for its referenced alias** per the rules above.
- The referenced API alias's **format supports chat completions** (formats without a chat-completions surface are rejected with a clear message).
- A router with **zero targets, or all targets disabled, is allowed to be saved** (see global invariants). Validation does **not** block this.

### Management UI (full)
- A dedicated create/edit experience for model-routers, reachable from the models area, consistent with the existing API-model management UI.
- Users can **add, remove, and reorder** targets; the order is the fallback priority.
- Each target row has an **enable/disable toggle** (the deselect-without-delete control) and a model picker whose behavior matches the pinning rules above.
- Model-routers appear in the **aggregate models list** and are **selectable in the chat model picker** like any other alias.

### Routing (pass-through only this phase)
- A chat completion addressed to a model-router routes to the **first enabled target in order** and returns that target's response **verbatim** (whether success or error).
- **Disabled targets are skipped** when choosing the first target.
- If the router has **no enabled targets**, the request returns a clear, typed router error (request-time, per global invariant).
- If the first enabled target's referenced alias has since been **deleted** (dangling reference), this phase may surface a typed error for that target. (Falling through past a dangling reference is introduced in Phase 2.)
- **Observability headers** are present on the response: the alias and model that served it, the strategy (`fallback`), and an attempt count (always 1 this phase).

## Out of scope (this phase)
- No failover / fall-through on error (Phase 2).
- No health tracking, cooldown, or recovery (Phase 3).
- No router "test"/probe capability (Phase 4).
- No request surfaces other than `/v1/chat/completions`.

## Acceptance gates (test-first)
Write these tests first; the phase is done when they pass and nothing regresses.

**Persistence & validation (service/integration):**
- Given a valid model-router payload, when created, then it is persisted and retrievable, and tenant isolation holds (a second tenant cannot see or fetch it).
- Creating with a duplicate alias name (against any alias kind) is rejected.
- Creating with a target that references a non-existent alias, another router, or itself is rejected with the corresponding typed error.
- Creating with an invalid pinned model (not offered by a selected-subset API alias; not matching a route=all prefix) is rejected.
- Creating with a referenced API alias whose format lacks a chat surface is rejected.
- Creating with zero/all-disabled targets **succeeds** (allowed).

**Resolution (integration):**
- A created model-router appears in the aggregate models listing and is resolvable by name.

**Pass-through routing (integration):**
- Given a router whose first enabled target points at a stubbed upstream returning a success, when a chat request is sent to the router, then the stubbed response is returned and the observability headers report that target, strategy `fallback`, attempt count 1.
- Given the first target is disabled and the second is enabled, the request is served by the second (disabled skipped).
- Given all targets disabled, the request returns the typed "no active target" router error.
- An upstream error from the chosen target is returned verbatim (no fallback this phase).

**Frontend (component):**
- The router form validates required fields, lets the user add/reorder/remove targets, toggles enable/disable per target, and adapts the model picker to the referenced alias type.

**End-to-end (Playwright, black-box, UI-only):**
- A user creates a model-router via the UI with one enabled target, selects it in chat, sends a message, and receives a completion. Assertions are made through the UI only.

## Demo script
Create a model-router named e.g. `my-stack` with one target pointing at a working provider alias; confirm it appears in the model list and chat picker; send a chat message and get a reply. Add a second target, disable the first, send again — served by the second. Disable both, send — clear error.
