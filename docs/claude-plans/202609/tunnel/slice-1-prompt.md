# Slice 1 — Prove a Cloudflare named tunnel can carry BodhiApp, including login

Paste this as the first message of a fresh Claude Code session in the BodhiApp repo. It is self-contained.

**Read this section first.** This prompt gives you the *goal*, the *constraints*, and the *facts that were expensive to establish*. It deliberately does **not** tell you what modules to create, how to shape types, where code should live, or what to name things. Those are yours to decide after you have read the code. If you find a better design than anything implied here, take it — and say so in the wrap-up file.

There is a 13-phase implementation plan at `docs/claude-plans/202609/tunnel/cloudflare-tunnel-implementation-plan.md`. **Do not follow it and do not read it.** We deliberately resequenced: it builds up to the risky part, and we want the risky part proven first. It survives only as a source of research pointers, which are reproduced below.

---

## 1. What this slice is for

BodhiApp runs on someone's own machine. We want it reachable from the public internet on a stable HTTPS URL, through a **Cloudflare named tunnel** on the user's own Cloudflare zone, with **no inbound ports opened** and **no BodhiApp-owned server infrastructure**.

Everything else about this feature — three tiers of Cloudflare authentication, binary download, automatic Keycloak sync, restart resume, hardening — is deferred. We are answering one question: **does this actually work end to end, including OAuth login through the tunnel hostname?** That last part is where the real risk sits, and it is why this is the first slice rather than the fifth.

Build the smallest honest thing that answers it. Resist adding capability that the demo below does not require.

## 2. Done is this demo, run live

The owner (a human) will watch this. Nothing counts as done until it has actually happened on a real Cloudflare zone.

1. Start BodhiApp on the desktop. A **Tunnels** entry is visible in the app's navigation.
2. On that page, the user provides the hostname they want (something on a zone their Cloudflare account controls, e.g. `bodhi.example.com`) and enables the tunnel.
3. The app provisions the tunnel and starts carrying traffic. The page shows the resulting **public URL** and a truthful connection state, without a manual page refresh being required to see it settle.
4. The page also shows the **exact OAuth redirect URI** that logging in through this hostname will send to Keycloak, in a form the user can copy.
5. The user pastes that redirect URI into the Keycloak client by hand, in the Keycloak admin console. **This slice performs no Keycloak API calls whatsoever.**
6. From a browser on a different network (e.g. a phone on cellular), the user opens the public URL, sees BodhiApp, clicks login, completes the Keycloak flow, **lands back logged in**, and can hold a working chat conversation with a streaming response.
7. Back on the desktop, disabling the tunnel stops the connector and the public URL stops serving. No `cloudflared` process is left behind, and none is left behind if the app is killed outright either.
8. Restarting the app shows the tunnel disabled. Enabling again reuses the same tunnel and hostname rather than creating a second one, and the URL is unchanged.

Step 6 is the point of the whole slice. If you have to choose what to polish, protect that one.

## 3. What you may assume, and what you must not

**Assume** the `cloudflared` binary is installed and on `PATH`. Do not build detection fallbacks, install guidance, or a downloader.

**Assume** the user has already run `cloudflared tunnel login` once and that `~/.cloudflared/cert.pem` exists. Do not drive that interactive browser login. If the cert is missing, fail with a message that says plainly what the user must run. That is the whole treatment.

**Do not assume** anything about Keycloak automation. Registering the redirect URI is a human action in this slice. Your job ends at *showing the user the exact string*.

## 4. Hard constraints

These are settled decisions, not preferences. Each has a reason; ignoring one will cost a rebuild.

- **Never set `BODHI_PUBLIC_HOST` (or the scheme/port siblings) to the tunnel hostname.** The instance legitimately serves several origins at once — loopback, LAN, and now the tunnel — and those settings mean "there is exactly one canonical origin". Setting them arms a `301` that would redirect local visitors away to the tunnel. See §6 for the middleware anchor.
- **`cert.pem` stays where `cloudflared` put it**, at `~/.cloudflared/cert.pem`. Do not move, copy, or relocate it, and do not override `HOME` in production code to change where it lands. Moving it breaks the user's own `cloudflared` CLI. There is an environment variable for pointing at an origin cert explicitly; prefer that over relocation.
- **Never pass a Cloudflare secret as a command-line argument.** Process arguments are world-readable on a shared machine.
- **The feature is gated by an env-overridable setting, `BODHI_TUNNEL_ENABLED`, defaulting to whether this is a native desktop install** (`SettingService::is_native()`, `crates/services/src/settings/setting_service.rs:189`). It is off in containers. See §6 for why this bites you during live testing.
- **No backwards-compatibility shims**, except that a committed database migration is immutable. If a shape is wrong, change it.
- **Trunk-based.** Commit straight to `main`. No branches, no PRs.

## 5. Facts already verified — do not re-derive these

These came out of reading `cloudflared` 2026.9.1 source and the BodhiApp codebase directly. They are reliable. Where a pointer is given, read that section rather than re-researching the topic; the research lives in `docs/research/tunnel/`.

**On the Cloudflare CLI and credentials** — `named-tunnel-operating-model.md` records the selected lifecycle, connector invocation, `/ready` signal, and `cert.pem` contract. `cert.pem` alone is sufficient for creating the tunnel and routing the DNS name on a single-owner account.

**On what reaches your origin through the tunnel** — `bodhiapp-integration-and-risks.md` records the current request-origin contract:
- `X-Forwarded-Proto: https` **does** arrive at the loopback origin. The Cloudflare edge adds it and `cloudflared` passes it through untouched.
- The `Host` header arrives **unmodified, as the public hostname**, because `cloudflared` only rewrites it when explicitly configured to, which we do not do. Consequently `X-Forwarded-Host` is **absent**.
- Therefore: matching `Host` against the configured tunnel hostname is the deterministic way to tell "this request came in over the tunnel". No header is unique to tunnel traffic.
- **Do not use `--http-host-header`.** It is unnecessary given the above, and setting it starts injecting headers that change this picture.

**On streaming and edge limits** — `bodhiapp-integration-and-risks.md`: server-sent events work through the named tunnel; the relevant risk is a timeout on long non-streaming requests.

## 6. Codebase facts you will need

Verified at the current `main`. Anchors may drift a line or two — confirm as you go.

**The login redirect is the crux, and it is currently wrong for this use case.** `crates/routes_app/src/auth/routes_auth.rs:88-101` composes the OAuth callback URL. When no explicit public host is configured, it builds the callback from `settings.public_scheme()` and related settings rather than from the origin the request actually arrived on. A user reaching the app at `https://bodhi.example.com` would therefore be sent to Keycloak with a callback pointing somewhere else, and the login would break or land them on the wrong origin. **Making login work through the tunnel means this composition has to become origin-aware.** How you do that, where the logic lives, and what else should share it, is your design call. Note that `crates/routes_app/src/setup/routes_setup.rs` composes redirect URIs too, in a related way — worth reading before you decide whether this is one concern or two.

**The canonical-URL middleware is currently harmless, and must stay that way.** `crates/routes_app/src/middleware/redirects/canonical_url_middleware.rs:26-29` short-circuits entirely unless an explicit public host is set. That is the safe default and the reason for the first hard constraint in §4.

**Child-process supervision has a precedent to study.** `crates/llama_server_proc/` manages `llama-server` as a child process. It deliberately uses `std::process` with reader threads rather than async process APIs: an earlier async version orphaned child processes when the parent exited, and this design fixed it. **Do not regress that.** Whether the tunnel connector belongs in a new crate, inside that one, or somewhere else entirely is yours to decide after reading it.

**Shutdown wiring is a single slot today.** `crates/server_app/src/server.rs:61` takes one optional shutdown callback, and `crates/server_app/src/serve.rs:41` already occupies it for llama. Two things now need to happen on shutdown. Also consider what happens when the process dies *without* running that path at all, given demo step 7.

**The gate defaults off in your dev loop.** The dev server runs as a container app type, so `is_native()` is false and the feature gate is off there. Every live run and every automated browser test needs the env var set explicitly, or you will debug a feature that was never enabled. Rebuild the dev-server binary after backend changes, or new routes are silently missing.

**Navigation** is configured in `crates/bodhi/src/components/shell/shell-nav-config.tsx`. Existing entries show the shape, including how sub-pages hang off a group. Where "Tunnels" belongs in that structure is your call.

**Screen conventions**: the current Remote Access surface and repository paths are summarized in `docs/research/tunnel/bodhiapp-integration-and-risks.md`. Use current production components and tests as the detailed convention source.

## 7. Explicitly out of scope

Quick tunnels, Tailscale, self-hosted tunnels. Cloudflare API tokens and OAuth — this slice is CLI-and-cert only. Binary detection, install guidance, downloading. **Any Keycloak API call.** Automatic restart resume. Deleting the tunnel or its DNS record from the app. Windows-specific process hardening. Multi-tenant. Reboot survival. Real-Cloudflare automated CI tests.

If you find yourself needing one of these to make the demo work, that is a genuine finding — note it in the wrap-up rather than quietly building it.

## 8. How to work

**You are on Sonnet and quota is limited.** Spend it on the demo, not on exhaustive test scaffolding. Prefer one decisive live check over three speculative test layers.

- Work upstream to downstream across crates: service layer, then routes, then server wiring, then the UI. Run the touched crate's tests as you leave it.
- Regenerate the TypeScript client after changing any API shape, before touching the UI.
- **Test depth is your judgment call this slice**, weighted toward what protects the demo. Real coverage for the origin-aware redirect composition is worth it — it is pure logic, it is the crux, and it is cheap to test. A full fake-`cloudflared` harness may not be, if it costs more than it protects here. Say what you chose and why in the wrap-up.
- When you do write tests: assert on error **codes**, never on message text. Never skip a test because an env var is missing — fail loudly instead. Browser tests are black-box: no reaching into the page, and no `if`/`else` branching on configuration.
- Comments explain non-obvious *why* only, in a line or two.
- Update the `CLAUDE.md` / `PACKAGE.md` of any crate you touch, in the same commit.
- Before committing: format, the touched crates' tests, the backend suite (run it **once**, tee it to a file, then grep the file — do not re-run a slow command just to re-read its output), the UI tests, and the browser tests if you touched them. Then refresh the code knowledge graph so it lands in the same commit.
- Commit to `main` when the demo passes.

**Stop and ask the owner** when you hit something only they can do: anything needing their Cloudflare dashboard, their zone, their Keycloak admin console, or a second device. Hand them a short numbered checklist and wait for the result. Never record an owner-run step as verified yourself.

**A note on the demo.** Steps 1 through 4 and 7 and 8 you can drive yourself in Chrome against a real zone once the owner has confirmed the hostname to use. Steps 5 and 6 are theirs. Ask early for the hostname, since nothing works without it.

## 9. Decisions that are yours

Not exhaustive, and not questions to send back — make a call, implement it, and record the reasoning:

- What the tunnel's lifecycle states are, which of them the user sees, and how the page learns about changes.
- What, if anything, needs to persist across a restart for step 8 to hold, and where it lives. Note the immutable-migration rule if you reach for the database.
- Whether the connector is supervised beyond "start it and watch it", and how much failure handling earns its place in a viability slice.
- How the page behaves when the feature gate is off, and when the cert is missing.
- How much of this is reusable if a second tunnel provider ever appears. A seam is fine; speculative abstraction is not.

## 10. Wrap up by writing a handover file

When you are done — or genuinely blocked — write `docs/claude-plans/202609/tunnel/slice-1-job-done.md`. The owner feeds this straight back to the planning session to size the next slice, so write it for someone who did not watch you work. Keep it under roughly 60 lines and make it honest rather than tidy.

Cover:

1. **What now works**, stated as what a person can do, not as what you implemented.
2. **Demo results, step by step** against §2. For each: passed, failed, or not attempted, and who ran it. If step 6 did not happen on a real device over a real network, say so plainly — it is the whole point of the slice.
3. **Design decisions you made** and why, especially anywhere you diverged from what this prompt implied. Name the files a reader should open to see the shape of it.
4. **Anything in §5 or §6 that turned out to be wrong or incomplete.** These facts get reused by later slices, so a correction here is high value. Include how you established the correction.
5. **What is deliberately unfinished or fragile**, and what would break first under real use.
6. **What you had to touch that this slice did not anticipate**, and whether that touch is complete or is now half-done.
7. **Commits** made, and the state of the working tree if anything is uncommitted.
8. **What you would do next**, and what you would need in order to do it.

Add an entry for the file in `docs/claude-plans/202609/index.md`, following the existing entries' shape. That index and the parent one already exist; do not recreate them.
