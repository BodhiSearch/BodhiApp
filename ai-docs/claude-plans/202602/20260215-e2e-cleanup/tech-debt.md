# E2E to server_app Migration: Tech Debt

## Tracked Items

### 1. Reintroduce E2E happy path after oauth-test-app refactor
Once oauth-test-app is refactored, add back a smoke E2E test for the full OAuth + toolset access user journey. This ensures UI wiring continues to work end-to-end.

### 2. Token + chat API test in server_app
Add server_app test combining API token creation with real LLM chat completion. Needs llama.cpp binary, builds on existing `test_live_agentic_chat_with_exa.rs` pattern. This validates the full OAuth token → chat completion flow without Keycloak.

### 3. Holistic review of toolsets-auth-restrictions.spec.mjs
Deeper restructuring needed:
- Remove stale `scope_toolset-*` terminology
- Adopt first-party access-request semantics
- Consolidate test setup
- Separate discussion phase, to be addressed closer to implementation
