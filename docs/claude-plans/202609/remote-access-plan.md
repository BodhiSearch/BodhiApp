# Remote Access (Cloudflare tunnel) — fix the feature and close the design gap

**Status:** Ready for review. Design work and adversarial review complete; decisions locked with the owner.

## Context

The Remote Access feature is implemented but does not work, and the screen does not match the
approved design. Two commits landed (`3cb6331f` slice-1, `ab0bf83b` design mock); a large second
phase sits **uncommitted** in the working tree — `crates/services/src/tunnels/service.rs` grew from
541 to ~1220 lines, plus routes, settings, auth, the React page, `openapi.json` and `ts-client`.

This plan fixes the real defects (not cosmetics) and then brings the screen in line with the design.

## Diagnosis — what is actually broken

Verified against the code, the local `cloudflared` 2026.9.1, and the live machine state.

### 1. Every subdomain change mints a new Cloudflare tunnel

`crates/services/src/tunnels/service.rs:483`

```rust
fn tunnel_name(hostname: &str) -> String {
  let digest = Sha256::digest(hostname.as_bytes());
  let suffix = digest[..6].iter().map(|b| format!("{b:02x}")).collect::<String>();
  format!("bodhi-{suffix}")
}
```

The tunnel identity is a hash of the **hostname**, so picking a different subdomain creates a
brand-new tunnel and abandons the old one — the hostname is not part of a tunnel's identity at all (§3).
The account currently holds four orphans:

```
16d3eaf0-3b5c-4889-b068-c71e51f3cb86  bodhi-13a0b174303f  2026-09-17
ef8628d5-e134-4515-85a5-098269ba43a5  bodhi-65e60206fddd  2026-09-16
41e825a4-41b2-4443-910a-971710a11a2f  bodhi-9e971e707602  2026-09-17
462dac13-5bff-4af1-a89f-bb2de01bce38  bodhi-f03ecb0c5ffe  2026-09-17
```

with matching credential files in `~/.bodhi-dev-makefile/tunnels/`.

### 2. Reusing an existing tunnel never writes its credentials — the likely cause of "it doesn't work"

`service.rs:543 tunnel_id()` runs `cloudflared tunnel list --output json -n <name>` and returns the
found id, but **only the create branch writes** `{bodhi_home}/tunnels/<name>.json`. So whenever the
tunnel exists on Cloudflare and the local credentials file does not — `make app.clear` wipes
`~/.bodhi-dev-makefile`, a different machine, a reinstall — the connector is launched with
`run --credentials-file <missing path>` and fails.

A fixed tunnel name makes this the *dominant* failure mode (the tunnel always exists after the first
create), so it must be fixed in the same change. The recovery command exists and was verified on this
machine: `cloudflared tunnel token --cred-file <path> <TUNNEL>` fetches and writes the credentials
JSON for an existing tunnel.

### 3. A named tunnel is not bound to a hostname

The hostname link is a *route* (`PUT /zones/{zone}/tunnels/{id}/routes` → CNAME →
`<id>.cfargotunnel.com`), and BodhiApp runs the connector with `run --url http://127.0.0.1:<port>`
(ad-hoc ingress, no `config.yml`), so the tunnel carries **no hostname ingress rules** and forwards
everything to the one local origin. Changing the subdomain therefore needs only one more
`cloudflared tunnel route dns <tunnel> <newhost>` call — no modify, no delete, no rename — and one
tunnel can serve several hostnames at once.

### 4. The screen is a functional subset of the design

The mock models **15** states with folding/locked steps, inline confirm and error panels, a creation
progress stepper and a grouped FAQ. `crates/bodhi/src/routes/tunnels/index.tsx` (457 lines) renders
a flat always-expanded ladder, uses two modals plus toasts, has a 4-item `<details>` FAQ, and
contains **one** responsive class in the whole file.

### 4b. The design was updated mid-planning to model Keycloak sync natively

`design/tunnels/ra-app.jsx` now carries three states the earlier revision did not, and they are the
reason the current `auth_sync` DTO is insufficient:

| id | label | `n` | tone |
|---|---|---|---|
| `kc-syncing` | Syncing sign-in redirect | C4 | checking |
| `kc-fail-net` | Keycloak unreachable | C5 | attn |
| `kc-fail-auth` | Keycloak rejected the change | C6 | attn |

The design treats all of these as **live** (`const LIVE = ['kc-syncing','kc-fail-net','kc-fail-auth','live']`),
so steps 1 and 2 stay locked throughout — the tunnel is up regardless of what the sync did.

**Backend consequence.** `TunnelAuthSyncState` is currently `{not_attempted, syncing, synced, failed}`;
a single `failed` cannot drive two different messages and two different remedies. It becomes:

```
not_attempted | syncing | synced | unreachable | rejected
```

mapped from `AuthService::update_tunnel_redirect_uri`: reqwest transport errors (connect, DNS, timeout)
and 5xx → `unreachable`; 401/403 and other 4xx → `rejected`. The FAQ states exactly this split — "this
machine can't reach the Keycloak server right now, or the credentials this instance uses aren't allowed to
edit its own client. The first clears up on retry; the second needs a Keycloak admin."

Header pill gains `Finishing up` (kc-syncing) and `Sign-in broken` (both kc failures); the step-3 pill
becomes composite — `On`, `On · syncing`, `On · sign-in broken`.

Exact copy to implement (from `ra-app.jsx` `KeycloakSync`/`KC_FAIL`):

- syncing — *"Syncing the sign-in redirect URL with Keycloak"* / "Registering `https://<host>/ui/auth/callback` so sign-in works through the tunnel."
- synced — *"Sign-in redirect URL synced with Keycloak"* / "You can sign in at **https://\<host\>**."
- unreachable — *"Couldn't reach Keycloak to update the sign-in redirect"* / "The tunnel is up, but signing in through the address will be refused until the redirect URL is registered."
- rejected — *"Keycloak refused the redirect URL change"* / "This instance isn't allowed to edit its own Keycloak client, so the redirect URL wasn't added. Signing in through the address will be refused until it is."
- both failures: `[Retry sync]` (primary) + `[Why this is needed]` → FAQ `faq-kcsync`.

**Deviation from the mock, deliberate:** the mock writes the callback as `/auth/callback`; the real
constant is `/ui/auth/callback` (`crates/services/src/settings/constants.rs:56`). Use the real path.

### 5. Smaller defects

| # | Defect | Location |
|---|---|---|
| a | `GET /tunnel` spawns `cloudflared --version` **and** does a Cloudflare zone REST lookup on every poll (1s while connecting) | `service.rs` `build_status()` |
| b | Connector exit is only noticed when someone polls; no supervisor | `service.rs:855 refresh_status()` |
| c | `enable()` holds no in-flight lock across its awaits; two concurrent calls each spawn a connector and the loser's child is leaked | `service.rs:953` |
| d | Metrics port is chosen by binding and dropping an ephemeral listener — TOCTOU | `service.rs` `enable()` |
| e | `reconnect()` hardcodes `replace_dns: false` and stamps two different failures with one code | `service.rs` `reconnect()` |
| f | The UI detects a DNS conflict by substring-matching `"dns record already exists"` | `routes/tunnels/index.tsx` |
| g | No tests beyond two inline validator unit tests — no fake-cloudflared harness, no route tests, no component tests, no E2E | repo-wide |

## Decisions locked with the user

| # | Decision |
|---|---|
| D1 | Tunnel name is `bodhi-app-tunnel-<uuid>`, where `<uuid>` is the UUID half of this instance's OAuth client id (`bodhi-resource-<uuid>`, reachable through the `tenant_service.get_standalone_app()` call `sync_redirect` already makes). Stable for the life of the instance, so it is always reused; unique per instance, so two machines sharing one Cloudflare account cannot collide; and self-identifying, so `cloudflared tunnel list` shows which instance created each tunnel. Supersedes the earlier `bodhi-app-tunnel{,-dev}` idea — tunnel names live on Cloudflare's side, not locally, so a constant name is account-global. |
| D2 | Reuse and retarget the tunnel; never delete the tunnel. On a **subdomain change only**, additionally make a best-effort attempt to delete the previous CNAME, so DNS and the Keycloak registration stay consistent (see R3). Failure to delete is logged and never blocks. Disable still deletes nothing. The four pre-existing orphans are cleaned up by hand. |
| D3 | The design is **indicative**: follow it visually and for its state model, but build with the production app's own components and conventions — do not port the mock's JSX or CSS. FAQ content is ours to write. |
| D4 | **Revised** after the design update. Drop the standalone OAuth-callback copy row entirely — the design's own sign-in states (§4b) supersede it. The callback URL appears only in the informational syncing note; an admin who needs it by hand gets the pattern from FAQ `faq-kcsync`. |
| D5 | Add no new comments; remove restating/narrating comments from every file touched. |
| D6 | No backwards compatibility except the database. This feature persists through the existing settings table, so no migration is needed. |
| D7 | **Durable state lives in the database only.** Nothing this feature depends on may be persisted as a file under `BODHI_HOME` — in Docker that volume is ephemeral. The credentials file is eliminated entirely (D8); any scratch file that remains is re-created as part of the flow and never assumed to exist. |
| D8 | Credentials are handed to `cloudflared` **in the child process environment, not on the command line.** argv is world-readable via `ps` / `/proc/<pid>/cmdline`; "never pass a Cloudflare secret as argv" is already a hard constraint in `tunnel/slice-1-prompt.md` and `docs/research/tunnel/named-tunnel-operating-model.md`. The environment achieves the same goal — no file — without the exposure. |
| D9 | `TunnelAuthSyncState` splits `failed` into `unreachable` and `rejected` (§4b), because the design gives each a different message and a different remedy. The Keycloak registration is **never cleared on disable** — the PATCH is gateway-keyed, so a later subdomain change overwrites the previous entry rather than accumulating. The design mock's FAQ sentence "Turning remote access off removes it again" is wrong and is removed from `design/tunnels/ra-faq.jsx` as part of this work. |
| D10 | **Remote Access is a single-instance, native-deployment feature.** Its behaviour under a clustered/replicated deployment is undefined: every replica would resolve the same tunnel name and become an additional connector, and Cloudflare would load-balance one hostname across unrelated instances. Out of scope to fix now — recorded as tech debt, with `BODHI_TUNNEL` required to stay unset (default) for non-native deployments. |

### D7 + D8 in detail — removing the credentials file

`crates/services/src/tunnels/service.rs:521` is the **only** place in `crates/services` that writes durable
state under `BODHI_HOME`:

```rust
let directory = self.settings.bodhi_home().await.join("tunnels");
```

Everything else there is a SQLite path (overridden to Postgres in Docker via `BODHI_APP_DB_URL`) or logs.
The connector credentials JSON is the lone offender — and treating it as persisted state is exactly what
makes defect #2 possible. So defect #2 and D7 collapse into one change: **there is no credentials-reuse
path; credentials are re-fetched from Cloudflare every time.**

Flow on each `enable()` / `reconnect()`:

1. `tunnel list --output json -n <fixed-name>` → tunnel id, or if absent
   `tunnel create --output json --credentials-file <scratch> <fixed-name>` → tunnel id.
2. `tunnel token <fixed-name>` → base64 token on **stdout**. Capture it; never log it.
3. Spawn the connector with the token in its **environment**, not argv, and no `--credentials-file`.
4. Nothing is written to disk at run time; nothing is read back on the next start.

Step 2 is confirmed by our own research, `docs/research/tunnel/named-tunnel-operating-model.md`
§5, which settles the two things that matter:

- `token` **works for locally-managed (credentials-file) tunnels**, not just dashboard ones, and "doesn't
  require the local credentials file to already exist" — it re-fetches from the API. That is precisely the
  recovery this feature was missing.
- With **no** `--cred-file` it "prints the base64-encoded JSON token to stdout (single line, no trailing
  prose)". With `--cred-file` it instead writes the file. So omitting the flag is the file-free path.

Mechanism, in order of preference — both are `run` options in 2026.9.1, both have env-var forms, and both
take precedence over `--credentials-file`:

| Env var | Flag | Note |
|---|---|---|
| `TUNNEL_TOKEN` | `--token` | **Primary.** Consumes the stdout token verbatim — zero files. One thing to confirm: `--token` is also the remotely-managed idiom, so verify cloudflared still honours `--url` ad-hoc ingress rather than trying to fetch remote configuration (a locally-managed tunnel has none). |
| `TUNNEL_CRED_CONTENTS` | `--credentials-contents` | Fallback if the above misbehaves. Still file-free: base64-decode the token ourselves and re-emit it in the credentials shape. The research doc gives both encodings — token `{"a":accountTag,"s":secret,"t":uuid,"e":endpoint}` → credentials `{AccountTag, TunnelSecret, TunnelID, Endpoint}` — so the transform is a four-field rename. |

A scratch file survives in exactly one place: step 1's first-ever `tunnel create`, which has no stdout-only
mode and otherwise defaults to writing into `~/.cloudflared/`. Point it at `$BODHI_HOME/tmp/tunnels/`
(dir `0700`, file `0600`), **check-or-create that directory as part of the flow rather than assuming it
exists**, never read the file back (step 2 re-fetches instead), and delete it immediately after create and
again on disable/shutdown.

The root of trust stays `cert.pem`, an operator-supplied **input** (`~/.cloudflared/cert.pem` or
`BODHI_TUNNEL_ORIGIN_CERT`) that BodhiApp never writes. No Cloudflare secret is stored in our database and
none reaches argv, logs, or an API response.

Note for the fake-cloudflared harness: every `create`/`list`/`route`/`token` invocation also fires a
best-effort background update check, so tests must not assert on exact outbound request counts.

## One-time manual cleanup (owner, outside the code change)

The four orphans were minted before the fix; per D2 the app will never delete them. Three of the four
map back to a hostname (the name is `sha256(hostname)[:6]`, so this was recovered by hashing the
hostnames still persisted in the two dev databases):

| Tunnel | Hostname |
|---|---|
| `bodhi-f03ecb0c5ffe` | `amir.my-tunnel.getbodhi.app` |
| `bodhi-13a0b174303f` | `my-tunnel.getbodhi.app` |
| `bodhi-65e60206fddd` | `my-tunnel.example.com` |
| `bodhi-9e971e707602` | not recoverable — check the Cloudflare dashboard |

```bash
cloudflared tunnel delete bodhi-13a0b174303f
cloudflared tunnel delete bodhi-65e60206fddd
cloudflared tunnel delete bodhi-9e971e707602
cloudflared tunnel delete bodhi-f03ecb0c5ffe
rm -f ~/.bodhi-dev-makefile/tunnels/bodhi-*.json
```

Then remove the leftover CNAMEs in the Cloudflare dashboard for the three hostnames above (records
whose target ends in `.cfargotunnel.com`). Deleting a tunnel does not remove its DNS record.

## Confirmed risks found by adversarial review

Each was traced in code, not speculated. `R3` and `R4` are live defects today, independent of this plan.

### R1 — Two machines, one Cloudflare account, one tunnel (resolved by D1)

`tunnel_id()` resolves purely by name, and since credentials are now re-fetched on demand (D7/D8),
adoption of another machine's tunnel would always *succeed*. Both machines would run connectors for one
hostname and Cloudflare would load-balance between them — sessions bouncing between unrelated instances,
silently. The client-id suffix in D1 removes the shared namespace, so this cannot arise. D10 records the
cluster case that the suffix does *not* solve.

### R2 — `tunnel token --cred-file` overwrite behaviour is undocumented (avoided)

`--help` does not say whether it overwrites or refuses at an existing path. Moot under D8: we call `token`
with **no** `--cred-file` and read stdout, so no file is involved on the run path.

### R3 — Retargeting silently breaks sign-in on the old address (live defect)

`crates/routes_app/src/auth/routes_auth.rs:101` builds the https callback **only** when the request host
equals the saved `BODHI_TUNNEL_HOST`:

```rust
if request_host == tunnel_host && forwarded_proto == "https" {
  format!("https://{tunnel_host}{}", services::LOGIN_CALLBACK_PATH)
} else if let Some(request_host) = request_host {
  format!("{}://{}:{}{}", settings.public_scheme(), request_host, settings.public_port(), …)
}
```

`public_scheme()`/`public_port()` special-case only RunPod, so everything else falls back to the **local**
bind scheme and port. After a subdomain change the old CNAME still points at the same connector, so the old
address loads — and its login builds `http://old.example.com:11135/ui/auth/callback`, a URL the browser can
never reach and Keycloak never registered.

**Fix (three parts):** (a) treat any request arriving with `X-Forwarded-Proto: https` as tunnel traffic and
build `https://<request-host>/ui/auth/callback` from the actual host; (b) best-effort delete the previous
CNAME on subdomain change (D2), so the old address stops resolving instead of loading a page that cannot
sign in; (c) say so in the FAQ and next to the address field — changing the address stops sign-in working
on the previous one.

### R4 — No timeouts anywhere, and `enable()` stores the child last (live defect)

`command_output()` has no `tokio::time::timeout` at all, and `enable()` does all provisioning I/O and spawns
the connector **before** taking the runtime lock to store `RunningTunnel`. Two consequences:

- A concurrent `disable()` in that window sees `running == None`, so the spawned connector is never tracked,
  never killed, and never reaped by `Drop` — the leaked-connector case (defect 5c). The startup reconnect in
  `crates/server_app/src/serve.rs:164` races a user-initiated `enable()` at exactly the moment the app becomes
  ready, so this is reachable in normal use.
- `ShutdownRuntimeCallback::shutdown()` calls `disable().await`. Put a single operation mutex around
  `enable`/`disable`/`reconnect` without timeouts and a stuck `cloudflared tunnel create` stalls process exit
  indefinitely.

**Fix:** wrap every `command_output()` in a bounded `tokio::time::timeout` with a distinct
`TunnelError::ProvisioningTimeout`; store the child (or a `Starting` placeholder) into the runtime **before**
any further work; and have the shutdown path bound its wait rather than blocking on the operation lock.

### R5 — Status reports the local connector, not reachability

`build_status()` derives state solely from the child's `/ready` endpoint, and `dns_conflicts()` runs only
inside `enable()`. A hostname whose CNAME was edited elsewhere still shows as healthy. **Consequence for the
UI:** "On" must mean *the connector is connected*, never "your address is reachable". The design's own copy
already respects this; keep it that way and do not add a "DNS OK"-style badge.

### R6 — Subprocess stderr reaches the wire unfiltered

`enable()` truncates `route.stderr` to 500 chars and returns it as `TunnelError::Provisioning`, which surfaces
in `TunnelStatus.error_message`. Nothing scrubs it. No evidence cloudflared prints secrets there, but apply
the existing `mask_sensitive_value` treatment before it leaves the process. (The audit separately **cleared**
the Cloudflare API token, the origin-cert path, and `get_client_access_token`'s `client_secret` — all already
masked or header-only.)

## Phases

Thin vertical slices, upstream → downstream. Each ends green and is committed before the next starts.
Steps 1–2 of the ladder are unchanged by phases 1–3, so the page stays usable throughout.

**Every phase has two gates.** The *automated gate* is mine: crate tests, component tests, generated-client
sync, and — from Phase 4 — Chrome at both widths driven by the fake `cloudflared` through the existing
`BODHI_TUNNEL_CLOUDFLARED_PATH` setting, so no local UI check ever touches Cloudflare. The *owner gate* is
yours: everything needing the real account, real DNS or real Keycloak. I stop at each phase boundary, name
the owner checks that have come due, and wait for your result before starting the next phase. I never claim
an owner check as passed.

### Phase 0 — Baseline commit (no code change)

Commit the 30 modified files exactly as they stand, so every later phase commit shows only its own diff.
Phase 1 rewrites tunnel identity and credentials inside those same files; without this, its commit would be
indistinguishable from the work it is built on. Run `make format` first, then commit — no behavioural edits,
no cleanup, no test additions.

**Automated gate:** `cargo check` across the touched crates and `npm run build` in `crates/bodhi` still pass,
confirming the baseline is the green-ish state the plan assumes rather than a broken snapshot.

### Phase 1 — Tunnel identity and file-free credentials (`crates/services`)

D1, D7, D8 and defects 1 + 2. Replace `tunnel_name(hostname)` with the client-id-derived name resolved once
at construction; drop the `hostname` parameter from `credentials_path`/`tunnel_id`; implement
resolve-or-create keyed on `list -n <name>`; fetch the run token via `tunnel token` on stdout and pass it in
the child environment; confine `create`'s scratch file to `$BODHI_HOME/tmp/tunnels/` (created per run,
deleted immediately). Build the fake-`cloudflared` harness here, since nothing else can be tested without it.

Because the `--token` question (below, **O0**) cannot be settled without a real tunnel, build the credential
handoff behind one small seam — a function that turns a fetched token into the child's environment — so
switching `TUNNEL_TOKEN` → `TUNNEL_CRED_CONTENTS` after O0 is a change in one place, not a redesign.

**Automated gate:** `cargo test -p services`, including fake-harness cases proving resolve-or-create, reuse
with no local state at all, and that no file is written under `BODHI_HOME` outside `tmp/tunnels/`.

**Owner gate:** O0, then O1 and O2.

### Phase 2 — Lifecycle correctness (`services`, `server_app`)

Defects 5a–e and R4. Operation lock with the child stored first; bounded timeouts on every subprocess call;
cached binary/zone lookups so polling stops shelling out and hitting Cloudflare's API every tick; a
supervisor that notices connector exit without waiting for a poll; metrics port handed to the child without
the bind-and-drop TOCTOU; reconnect failures given distinct codes instead of one string; error codes
centralised as constants shared with the DTO.

**Automated gate:** `cargo test -p services -p server_app`, with fake-harness cases for connector exit,
concurrent enable/disable, and a shutdown that completes while a provisioning call is still in flight.

**Owner gate:** O4.

### Phase 3 — Auth sync and host handling (`services`, `routes_app`)

D9 and R3. Split `TunnelAuthSyncState`; map transport/5xx → `unreachable`, 401/403/4xx → `rejected`; fix the
`routes_auth.rs` fallback to honour `X-Forwarded-Proto: https`; best-effort delete of the previous CNAME on
subdomain change; DNS conflict returned as a structured code rather than a message the UI greps for; scrub
subprocess stderr before it reaches `error_message` (R6). Regenerate `openapi.json` and `ts-client`.

**Automated gate:** `cargo test -p services -p routes_app -p server_app`; `make ci.ts-client-check` shows the
generated clients in sync; a test asserts the PATCH body is byte-identical across repeated enable/disable
cycles, which is as close to O6 as anything automated can get.

**Owner gate:** O5, O6, O7.

### Phase 4 — Page rebuild, structure and states (`crates/bodhi`)

The 15-state model, folding/locked steps, inline confirm and error panels replacing the two modals, the
creation stepper, and the responsive behaviour the current page lacks. Derive `locked` from connection state
rather than `status.enabled` — `enabled` is just `running.is_some()`, so a failed-but-present connector
currently traps the user in a read-only form.

**Automated gate:** component tests per state; Chrome at desktop and at 430px, driven end to end by the fake
`cloudflared` from Phase 1 selected via `BODHI_TUNNEL_CLOUDFLARED_PATH` — real states from the real backend,
no Cloudflare contact.

**Owner gate:** none. Everything here is reachable through the fake connector.

### Phase 5 — Page rebuild, content and polish (`crates/bodhi`)

The Keycloak sync notes with the exact copy in §4b; grouped FAQ written by us (D3) with deep links from each
step; install/update links for the binary-missing state and cert troubleshooting copy, both required by
`prompt-tunnel.md` and absent today; accessibility and focus behaviour.

**Automated gate:** component tests; Chrome check at both widths against the fake connector.

**Owner gate:** O8 — the first check that proves the feature does what it is for.

### Phase 6 — End-to-end, docs, cleanup

One Playwright journey of many `test.step`s against the fake connector, per `docs/conventions/testing.md`;
update the crate `CLAUDE.md`/`PACKAGE.md` files touched; record D10 in the tech-debt doc; remove the wrong
"Turning remote access off removes it again" sentence from `design/tunnels/ra-faq.jsx` (D9); move this plan to
`docs/claude-plans/202609/` with an index entry.

Docs cleanup, precisely — the sidecar files are **not** interchangeable:

| File | Action |
|---|---|
| `202607/this-task-is-to-snazzy-twilight-agent-ad89cb92a5f617d7e.md` | **Keep** — it is the lifecycle design slice (status caching, supervision, error taxonomy, reconnect) and is the working reference for Phase 2. Move it alongside this plan at the end. |
| `202607/staged-gathering-lightning-agent-a4a4509f5487b5f99.md` and `-af42b439898deb3b7.md` | Delete — drafts of the superseded 12/13-phase plan. |
| `202609/remote-access-tunnel-plan.md` | Delete, with its index entry — superseded by this plan. |
| `202609/tunnel-impl.md` | Delete, with its index entry — a reviewer handoff for the uncommitted worktree whose open questions are now answered by D1–D10 and R1–R6. |

**Automated gate:** `make test.e2e` plus the full gate set — `make format`, `make test.backend`, `make test.ui`
— all green before the final commit.

**Owner gate:** O9, deliberately last because it is the only check that mutates real DNS.

## Owner-run verification (yours — real Cloudflare account, real DNS, real Keycloak)

Every item here is run by you, not by me. Each is tagged with the phase it gates; I pause at that boundary
and wait for your result. **O0 is the only one that blocks design rather than confirming it** — it decides
which of the two file-free credential mechanisms Phase 1 keeps.

| # | Gates | Check |
|---|---|---|
| ~~**O0**~~ | Phase 1 → 2 | **ANSWERED 2026-09-18 — yes, `TUNNEL_TOKEN` stands.** Verified against the real account with cloudflared 2026.9.1 on two tunnels that had been created locally from `cert.pem`. `cloudflared tunnel token <name>` returned a token for each; running with only that token logged `Starting tunnel tunnelID=<the local tunnel>` and `url:http://127.0.0.1:18080` in its settings, registered 4 edge connections, and never referenced a credentials file. End to end, isolated on a tunnel with no other connector attached, `http://my-tunnel.sub.example.com/` returned `200` with the throwaway origin's body. The `connector_credentials_env` seam stays on `TUNNEL_TOKEN`; `TUNNEL_CRED_CONTENTS` is not needed. |
| O1 | Phase 1 | `cloudflared` is discovered and `cert.pem` is accepted. Gates everything below it. |
| O2 | Phase 1 | Enable once. Exactly one tunnel named `bodhi-app-tunnel-<uuid>` exists, and the UUID matches this instance's OAuth client id. |
| O3 | Phase 1 | `make app.clear`, then enable again on the same subdomain. The same tunnel must be reused and must connect. **This is the regression that motivated the plan** — if it still fails, Phase 1 is not done. |
| O4 | Phase 2 | Kill the connector out-of-band; the failure surfaces promptly without needing a page refresh. Restart with auto-reconnect on: exactly one attempt, never a retry loop. |
| O5 | Phase 3 | Change the subdomain. The same tunnel is retargeted, the new address works, and the old CNAME is gone — or a warning was logged if the delete failed. |
| O6 | Phase 3 | Enable/disable/re-enable three times, then inspect the Keycloak client: exactly one redirect URI, and unrelated registrations untouched. |
| O7 | Phase 3 | Force a sync failure (wrong client secret). The `rejected` copy appears, `Retry sync` recovers without restarting the connector, and API-key traffic keeps working throughout. |
| O8 | Phase 5 | From a network that is not this LAN, sign in through the public address end to end. |
| O9 | Phase 6 | Pre-create a foreign CNAME, then enable onto it — the replace-DNS confirmation appears and `--overwrite-dns` does the right thing. **Mutates real DNS**; do it once, deliberately, last. |

The one-time orphan cleanup above is independent of these and can happen whenever convenient.

> Hostnames throughout this document use `example.com` placeholders; the real zone is not named in
> this repository. The operational records below describe real account state under that zone.

**Orphan cleanup — done 2026-09-18.** The four `bodhi-<12 hex>` tunnels (`bodhi-13a0b174303f`, `bodhi-65e60206fddd`,
`bodhi-9e971e707602`, `bodhi-f03ecb0c5ffe`) and their four `*.cfargotunnel.com` CNAMEs were deleted from the
`example.com` zone; `cloudflared tunnel list` is now empty and the zone's A/MX/NS/TXT records were untouched. Locally,
`~/.bodhi-dev-makefile/tunnels/` and the saved `BODHI_TUNNEL_HOST` were removed, so the next enable starts clean and
should produce exactly one tunnel named `bodhi-app-tunnel-0cb93fdd-f8f4-471e-aaa8-54feb41c073d`.

**Constraint discovered while verifying O0.** Cloudflare Universal SSL covers `example.com` and `*.example.com` but not
multi-level names, so a hostname like `my-tunnel.sub.example.com` fails the TLS handshake outright and is
reachable only over plain HTTP — useless for OAuth. Three of the four orphan records were of exactly this shape,
created by the slice-1 code before `validate_subdomain` rejected dots. The validation now prevents it
(`service.rs:576`); Phase 5 copy should say plainly that the subdomain is a single label, and why.
