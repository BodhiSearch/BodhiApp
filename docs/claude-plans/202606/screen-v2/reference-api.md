# UI V2 Migration — Reference API & id_token

Some V2 screens show data the BodhiApp backend doesn't own (the first is the **MCP Discover
catalog** — server publishers, verified badges, install counts, tool lists, usage stats). That data
comes from an **external reference API**, called **directly from the frontend**.

## The reference API

- **Service:** `https://api.getbodhi.app/` (configurable; env-overridable for tests).
- **Called directly from the frontend** — no backend proxy. The decision: we don't want the backend
  proxying these calls; the frontend hits the reference API and identifies the user so the service
  can do per-user rate-limiting / analytics.
- **Built separately, in parallel.** The reference APIs are a separate workstream; they'll be
  available in dev/prod independently. The migration is **never blocked** on them (see
  "per-batch prerequisite" below).
- **Configurable base URL** surfaced to the frontend so dev/prod/test can point at different
  origins. This indirection is also what lets tests point the client at a mocked origin.

## Identity: the OAuth `id_token`

- The frontend sends the user's OAuth **id_token** as `Authorization: Bearer <id_token>` to the
  reference API. The id_token reveals **identity only** — secure system access uses the access token
  elsewhere — so exposing it to the frontend for this purpose is acceptable.
- **Backend gap (Batch-0 work):** today the OAuth id_token returned by the IdP at login is
  **discarded** — only access/refresh tokens are persisted. So Batch 0 must thread the id_token
  through login, persist it in the session, and surface it to the frontend. Surface it on the
  current-user response (the natural home for user identity). Explore the current auth/session/user
  code before implementing — signatures change.
- **Endpoint config:** the reference-API base URL is a new `SettingService` setting
  (`BODHI_REFERENCE_API_ENDPOINT`, default `https://api.getbodhi.app/`, env-overridable) surfaced on
  the app-info response. (The user's first instinct was to put it on `/bodhi/v1/info`; pick whatever
  config/info endpoint is the best fit after exploring — keep it light, don't over-engineer.)

## Per-batch prerequisite (NOT a Batch-0 deliverable)

Reference APIs are handled **per batch, when a batch's screens need them** — not all built up front.
The MCP batch is the first consumer. When a batch needs a reference-API capability, its prerequisite
step (before implementing the UI):

1. **Define the TS interface** for the data, derived from the prototype's data shape (the
   `design/*-app.jsx` for that screen carries representative sample data — read it to infer fields).
2. **Write an interface spec doc** in this folder (e.g. `mcp-discover-reference-api.md`): endpoint
   paths, request/response schemas, auth via id_token. This is the hand-off so the API team builds
   it in parallel.
3. **Build & test the UI against an MSW external-origin stub** so the batch is self-sufficient.

## Testing the reference API (MSW external-origin stub)

- MSW intercepts by **absolute URL**, so external origins are first-class. The frontend client reads
  its base URL from app-info; in tests, the app-info mock returns a **mock origin** (e.g.
  `http://localhost:9000`) and the stub handlers register against that origin. The client therefore
  points at the mock purely through the normal config flow — **no env hacks, no client branching**.
- The id_token flows from the logged-in-user mock; stub handlers can assert the `Authorization`
  header is present.
- If a real reference API isn't available where it's genuinely needed (e.g. CI e2e), the batch plan
  must state how that's handled (stub origin, tagged/gated e2e, or defer that screen) — **explicitly,
  never silently**.

A small typed `fetch`-based reference-API client + a hook that composes (base URL from app-info) +
(id_token from current-user) is part of the Batch-0 scaffold; the actual domain hooks
(e.g. an MCP-catalog hook) are added by the batch that needs them. See [architecture.md](@architecture.md)
and [process.md](@process.md).
