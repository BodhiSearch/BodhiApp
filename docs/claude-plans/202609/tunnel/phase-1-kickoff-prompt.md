# Kickoff — Cloudflare tunnel, Phase 0 (SPI) and Phase 1 (gate + detect + page shell + test spine)

> **Superseded (2026-09-16) — do not paste this one.** The owner resequenced delivery to tackle the riskiest part first. The live starting prompt is [`slice-1-prompt.md`](slice-1-prompt.md). This file is kept for the Phase 0 (Keycloak SPI) detail, which is still accurate and will be reused when redirect-URI automation is picked up.

Self-contained prompt for a **fresh Claude Code session**. Paste it as the first message. Everything you need to locate is named below; do not re-derive it from scratch.

---

## 1. What you are doing

Implementing the first two phases of **Remote access via Cloudflare named tunnel** in BodhiApp:

- **Phase 0** — a new self-service redirect-URI endpoint in the sibling repo `keycloak-bodhi-ext`, released and verified live on both hosted Keycloaks. BodhiApp is untouched. This is a **parallel track**: start it first (it has release + Railway lead time) but it only gates Phase 5, so it does not block Phase 1.
- **Phase 1** — in BodhiApp: the `BODHI_TUNNEL_ENABLED` feature gate, a new `crates/cloudflared_proc/` leaf crate that detects the `cloudflared` binary, a `GET /bodhi/v1/tunnel/detect` endpoint, an admin-only "Remote Access" page under Settings, and the **Phase-1 slice of the test spine** (the fake `cloudflared` stub, a minimal `server_app` live harness, the Playwright fixtures/page-object/spec). Phase 1 touches **nothing** that reads the Phase-0 SPI — `spi_redirect_uris` and `getClientRedirectUris` land in Phase 5 with their first consumers, which is what makes "Phase 0 does not block Phase 1" literally true. Build only what §4.2 of the plan lists under *Phase 1*.

Land each phase as **one commit straight to `main`** (trunk-based; no branches, no PRs).

## 2. Read first, in this order

1. **The plan** — `docs/claude-plans/202609/tunnel/cloudflare-tunnel-implementation-plan.md`. Read all of it once; §0, §2 and the Phase 0 / Phase 1 entries in §3 and §4 are the working text. **§0 is load-bearing**: two items the earlier drafts treated as open risks are resolved, and the plan deletes the contingencies they implied. Do not reintroduce them.
2. **Current research** under `docs/research/tunnel/`:
   - `named-tunnel-operating-model.md` for Cloudflare prerequisites, credentials, commands, and supervision
   - `bodhiapp-integration-and-risks.md` for settings, auth, API, UI, and validation contracts
   - `quick-tunnel-decision.md` for the account-less tunnel no-go decision
3. **Repo conventions** — root `CLAUDE.md`, `crates/CLAUDE.md`, `MDFILES.md`. Testing skills: `.claude/skills/test-services/SKILL.md` and `.claude/skills/test-routes-app/SKILL.md` — load the matching one before writing tests at that layer.

Do **not** read the three superseded drafts in the scratchpad; the plan supersedes them.

## 3. Ground rules

- Work **upstream → downstream** within a phase: `cloudflared_proc` → `services` → `routes_app` → `server_app` → `lib_bodhiserver` → `bodhi` UI. Run `cargo test -p <crate>` as you leave each crate.
- `make build.ts-client` after any DTO/OpenAPI change, before touching the UI.
- Comments only for non-obvious *why*, 1-2 lines. Never restate what the code shows.
- No backwards compatibility except the database. Committed migrations are immutable.
- Assert error codes via `body.error.code` / `body["error"]["code"]`, never message text.
- Never `test.skip()` for a missing env var or a missing `node` — **throw** so the failure is loud.
- E2E is black-box: no `page.evaluate`, no context `fetch`, no `if/else` branching. A different server configuration means a separate `test()`, not a conditional.
- Update the `CLAUDE.md` / `PACKAGE.md` of every crate you touch, in the same commit.
- `bodhiserver_dev` runs as `AppType::Container`, so the gate defaults **off** there: every `make app.run.live` invocation and every Playwright server config needs `BODHI_TUNNEL_ENABLED=true`. Rebuild the dev-server binary after backend changes or new routes are silently missing.
- **CI is live on every push to `main` touching `crates/**`** — `build.yml` (backend + a Playwright job) and `playwright.yml` both fire (plan §4.5). Whatever you land must pass on a fresh `ubuntu-latest` checkout, not just on this machine: the stub's committed exec bit, the temp-`HOME` isolation, and the live tests against the shared `INTEG_TEST` auth server all run there.
- **Maintain the plan indexes** per `docs/claude-plans/CLAUDE.md` ("Maintenance rules (MUST follow)"). `docs/claude-plans/202609/index.md` already exists, with entries for the implementation plan and this kickoff, and `docs/claude-plans/index.md` already carries the `202609/` pointer — do not recreate either. Add an entry to `202609/index.md` for **every** file you add under `202609/tunnel/` (retrospectives, next-phase kickoffs), in the same commit as the file. Date = the file's git-creation date (today for a new file); never change an existing entry's date.

## 4. Phase 0 — exit criteria

From the plan's Phase 0 entry. Repo: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/keycloak-bodhi-ext`.

- `GET` and `PUT /realms/{realm}/bodhi/resources/redirect-uris` exist, service-account self-service via `checkForServiceAccount` (`ResourceService.java:303`), acting only on `getIssuedFor()`'s own client. Empty list → 400. Non-service-account or no token → 401.
- `RedirectUrisEndpointTest` covers all eight named tests in the plan, including `testSetRedirectUrisCannotMutateAnotherClient` and `testSetRedirectUrisKeepsWebOriginsPlus`.
- The SPI repo's own gate passes: **`make test`** (`Makefile:20` — clean, compile, all tests including integration).
- Commit (SPI repo): `feat(resources): GET/PUT redirect-uris self-service endpoint for resource clients`.
- **Then STOP — owner checkpoint.** Releasing via `make release-server` (`Makefile:197`) cuts and pushes a public tag, and confirming Railway Image Auto Updates (or clicking Deploy) for the `main-id` **and** the separately-managed `test-id` services is a dashboard action. Neither is yours to perform. Hand back with: the commit, the tag you propose, and a one-line request for the owner to cut the release and confirm both Railway services. Do not claim a deploy you did not make.
- **Resume only after the owner confirms**, to run the two probes yourself: **dual live verification on both hosts** — an unauthenticated `PUT` returns **401, not 404** (proves the route exists), and an authenticated `PUT` → `GET` round-trip with a **throwaway** resource client's service-account token returns 200 with matching sets.

## 5. Phase 1 — exit criteria

From the plan's Phase 1 entry.

- `BODHI_TUNNEL_ENABLED`, `BODHI_TUNNEL_CLOUDFLARED_PATH`, `BODHI_TUNNEL_ORIGIN_CERT` in `SETTING_VARS` but **not** in `EDIT_SETTINGS_ALLOWED`; `tunnel_enabled()` defaults to `is_native()` and an env value overrides it.
- `crates/cloudflared_proc/` exists as a leaf crate with detect / version / cert-path logic and its own `CLAUDE.md` + `PACKAGE.md`; the dependency chain is updated in the root `CLAUDE.md` and `crates/CLAUDE.md`.
- `GET /bodhi/v1/tunnel/detect` is registered in `admin_session_apis`, rejects unauthenticated and non-admin callers, and answers `200` with `available: false` when the gate is off.
- Settings → Remote Access renders all three states (found / missing → install guide / gate-off → unavailable), verified **live in Chrome**.
- The **Phase-1 slice** of the test spine exists and is self-tested: the fake `cloudflared` at `crates/lib_bodhiserver/tests-js/fixtures/bin/fake-cloudflared.mjs` implementing §4.1's contract (including the JSONL invocation log that records secret **presence only**), `server_app/tests/utils/tunnel_harness.rs` with **`fake_cloudflared()` and a minimal `live_tunnel_server(fake)` only**, `tests-js/fixtures/tunnelFixtures.mjs`, `pages/RemoteAccessPage.mjs`, and `specs/settings/remote-access-tunnel.spec.mjs` with its first step. The seven stub self-tests in the plan pass. Do **not** add `poll_tunnel_status`, `raw_get_with_host`, the throwaway-resource-client fixture, `spi_redirect_uris` or `getClientRedirectUris` — they belong to Phases 4 and 5 (plan §4.2) and would be dead code here.
- Every named test in the plan's Phase 1 test list exists and passes at its layer.
- Commit: `feat(tunnel): feature gate, cloudflared detection, Remote Access page shell and test spine`.

## 6. The loop, per phase

1. **Implement**, upstream → downstream, in the file paths the plan names.
2. **Tests per layer** — write them as you go, not at the end: `cloudflared_proc` unit (spawns the real stub), `services` unit (`MockTunnelProvider`, never spawns a process), `routes_app` oneshot (`build_test_router` for auth tiers, `AppServiceStubBuilder` + `MockTunnelService` for handlers), `server_app` live HTTP, UI vitest + MSW, and the growing Playwright spec.
3. **Live-verify in Chrome** — `BODHI_TUNNEL_ENABLED=true make app.run.live`, then walk the plan's live-check list for the phase. This is not optional and not replaceable by tests. The plan tags each live check **agent-executable** or **owner-only**: run every agent-executable one yourself; for an owner-only one (real Cloudflare zone, a phone on another network, a Windows box) stop, hand the owner the numbered checklist, and wait for the pasted result **before** committing. Never mark an owner-only check done yourself.
4. **Gate checks, all of them** — `make format`; `cargo test -p <touched crates>`; `make test.backend 2>&1 | tee /tmp/tb-phase1.log` (run once, grep the file afterwards — do not re-run it to re-read output); `cd crates/bodhi && npm test`; `make test.e2e` (from `crates/lib_bodhiserver/tests-js`, since the spec is touched); finally `graphify update .` so the knowledge graph lands in the same commit. Nothing is skipped.
5. **Commit** to `main` with the subject the plan gives — including the `index.md` maintenance from §3. Rebase onto `origin/main` before pushing.
6. **Write a short retrospective** next to the plan: `docs/claude-plans/202609/tunnel/phase-<N>-retrospective.md` — what shipped, what the live check actually showed (and which parts the owner ran), **the wall-clock time of the growing `remote-access-tunnel.spec.mjs` lifecycle test** and whether its `test.setTimeout` still fits (plan §4.3), anything the plan got wrong (especially: for Phase 0, the verified SPI release tag and whether Railway auto-updated `test-id`), and any risk row in §5.1 whose default changed. Keep it under ~40 lines. Add its `index.md` entry.
7. **Propose the next phase's kickoff** — write `docs/claude-plans/202609/tunnel/phase-<N+1>-kickoff-prompt.md` in the shape of this file, carrying forward anything the retrospective changed, and add its `index.md` entry. Then stop and hand back.

## 7. Watch out for

- **§0 of the plan.** `route dns` with `cert.pem` alone is resolved-yes; the manual-CNAME path is a defensive error path for restricted team-member accounts, not the expected outcome. `X-Forwarded-Proto` and `Host` are resolved-reliable at the origin; `Host`-vs-tunnel-hostname is the tunnel signal, and there is no `--http-host-header` fallback anywhere in this design.
- **`cert.pem` stays at `~/.cloudflared/cert.pem`.** Never move it, never copy it into `$BODHI_HOME`, never override `HOME` in production code to relocate it. Pass `TUNNEL_ORIGIN_CERT` on non-`run` commands instead. (Test harnesses may point `HOME` at a temp dir — that is a test-isolation measure, not a production behavior.)
- **The run argv in §2.7 is the contract.** All `run` flags after `run`; no `--origincert` on `run`. The fake tolerates either flag position so the first real spawn in Phase 4 can validate it — do not "fix" the pin by inference.
- **Never send a full-replace redirect-URI `PUT` to the shared `INTEG_TEST_RESOURCE_CLIENT_ID`** (`live_server_utils.rs:166-167`). Every live redirect-URI test uses a throwaway resource client created and cleaned up by the harness. Phase 1 builds that harness; Phase 5 depends on it.
- **The Playwright config lives at `crates/lib_bodhiserver/playwright.config.mjs`** — the crate root, **not** under `tests-js/`. The `webServer` pattern to copy is `test-mcp-auth-server` (`:133-145`), not `test-mcp-oauth-server`. It runs `workers: 1`, `fullyParallel: false`, `retries: 0` (`:29,34,35`) with a 120 s per-test timeout (`:12`), so give the lifecycle test an explicit `test.setTimeout` and report its wall-clock time in the retrospective.
- Use `ports kill <n>` for port/process housekeeping, and cap that kind of detour at about 30 seconds.
