# UI V2 Migration — Tech Debt / Failing Tests

Tracked failing tests + deferred fixes surfaced during the migration. Update as items are resolved
(strike through / move to a "Resolved" section) or as new ones appear in later batches.

## Failing tests carried from Batch 0 (Foundation)

### 1. Backend live tests — pre-existing, NOT a Batch-0 regression
Confirmed failing identically on a clean `git stash` tree (so unrelated to the id_token / settings /
AppInfo changes). Root cause: API-model create request schema drift — the test posts a body the
current `ApiModelRequest` rejects with `missing field 'name'`.

- `cargo test -p server_app --test test_live_anthropic` — multiple cases panic at
  `crates/server_app/tests/test_live_anthropic.rs:41` (and `:198`):
  `Failed to create Anthropic API alias: ... missing field 'name'`.
- `cargo test -p server_app --test test_live_model_router` — cases panic at
  `crates/server_app/tests/test_live_model_router.rs:34`.

**Status:** deferred. Fix the test request bodies (add `name`) or the schema — separate from the UI
migration. Backend gate otherwise: 51 `test result: ok` blocks.

### 2. E2E failures (`make test.e2e`) — Batch 0 run: 214 passed / 5 failed / 16 skipped (46.3m)
The 5 failures are 3 distinct specs, each failing in BOTH `standalone` and `multi_tenant` projects.
Clustered on live-upstream / streaming / setup — not shell-chrome (all mock-server Models/tokens/
MCP/nav E2E tests pass, so shell navigation is intact).

| Spec | Projects | Likely cause | Status / action |
|---|---|---|---|
| `tests-js/specs/api-models/api-models-forward-all.spec.mjs:38` › `forward_all_with_prefix: create, sync models, chat, prefix uniqueness, delete` | standalone + multi_tenant | Requires a real OpenAI key (`ApiModelFixtures.getRequiredEnvVars`) + live `api.openai.com` sync; fails fast (~9s). | **Env/credential-dependent.** Likely pre-existing. Verify on clean tree; otherwise needs a valid `OPENAI_API_KEY` in the E2E env. |
| `tests-js/specs/chat/chat.spec.mjs:72` › `multi-chat management and error handling @integration` | standalone + multi_tenant | Real streaming + error handling (~55s, timeout-ish). Chat is UNMIGRATED (old chat UI now inside the new shell). | **TO VERIFY before the Chat batch (Batch 5):** clean-tree stash-compare. If it passes clean but fails with Batch 0 → real shell full-height/scroll regression on the chat page → fix. If it fails clean too → pre-existing streaming flakiness. |
| `tests-js/specs/setup/setup-api-models.spec.mjs:133` › `API Models Setup - Happy Path Model Creation` | standalone | Setup wizard flow. | **Out of scope** (setup is deferred, renders bare). Likely pre-existing / env. Verify on clean tree. |

**Recommended next action:** `git stash` the Batch-0 changes and re-run ONLY these 3 specs against
the clean tree to classify each as pre-existing vs regression. Priority is the **chat** spec, since
chat renders inside the shell and is the only one touching shell-affected layout.

## Other deferred items
- Repo-wide ESLint: ~289 pre-existing errors in files untouched by the migration (import/order in
  `stores/initStores.ts`, `types/chat.ts`; a missing `react/display-name` rule in
  `tests/mocks/framer-motion.tsx`; etc.). Batch-0 files are lint-clean. Not addressed here.
- `make test.backend` requires Docker (PostgreSQL); pg-variant DB tests panic at fixture setup if
  Docker is down. Start Docker before the backend gate.
