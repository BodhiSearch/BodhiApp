# Tunnel review remediation — implementing the 28 Hunk notes on `70b221d5`

> **Status:** ready. Claims behind Batches 2 and 3 were adversarially verified by two independent agents — one of my own claims came back **refuted** and is corrected inline.
>
> Three mechanical inventories (per-line comment verdicts, the domain replacement map, the docs count map) are deliberately **not** pre-computed here — they are cheap to generate at execution time and would date quickly. The files and the rules to apply are named in Batch 4.

## Context

Commit `70b221d5` ("feat: Cloudflare named tunnel backend, connector lifecycle, and reachability") landed the server side of Remote Access: 46 files, ~4,000 added lines, of which `crates/services/src/tunnels/service.rs` alone is 1,574. It was built as a deliberately minimal viability slice under an explicit "quota is limited — spend it on the demo, not on test scaffolding" directive (`docs/claude-plans/202609/tunnel/slice-1-prompt.md` §8), and the slice's own handover records choosing "the cheap redirect test over a fake-cloudflared harness."

The code then grew past that slice without the test strategy being revisited. A review in Hunk produced **28 notes**. They are not a list of bugs — they are one architectural complaint with a long tail:

> `DefaultTunnelService` talks to the outside world directly — it spawns processes, writes files, opens sockets and reads `PATH` — so the only way to test it is to generate a **shell script** that impersonates `cloudflared`, spawn it for real, and `sleep` until something is observable. That is unrunnable on Windows, slow, and it pins the test suite to the platform instead of to the logic.

Everything else — duplicated settings precedence, a hand-rolled test-builder intercept, over-long comments, docs carrying counts that drift, the company domain in an open-source repo — is hygiene that rides along.

**Outcome sought:** the tunnel service becomes unit-testable through injected interfaces that only *invoke* the external world and hand back raw output, with all parsing and decision logic in Rust that tests can drive; and the surrounding hygiene notes are closed.

**Decisions taken by the owner before planning:**

| Decision | Choice |
|---|---|
| Seam shape | Split traits: CLI + connector (+ a small IO trait). **The seam returns raw output; the caller parses.** |
| Test layers above `services` | **Out of scope.** Already deferred with a note in `docs/claude-plans/techdebt.md` → *Remote Access*. Revisit when clearer. |
| Git | Follow-up commits on `main`, one per batch. `70b221d5` is not amended. |
| The company domain | Repo-wide sweep. |

## Coverage — all 28 notes

| # | Note | Disposition |
|---|---|---|
| 1 | Explain the callback-host logic | Answered below + Batch 3.2 (de-duplicate the branch) |
| 2, 3 | Tunnel host vs request host should differ | Batch 3.1 |
| 4 | Re-verify the deleted test setup | Batch 3.1 — **a real regression was found** |
| 5, 11 | Strip verbose comments | Batch 4.1 + 4.2 (write the rule down) |
| 6 | Extract a router test utility | Batch 3.3 — **scoped down**, rationale given |
| 7 | Company domain out of the repo | Batch 4.5 |
| 8, 9 | Docs carry counts that drift | Batch 4.3 |
| 10 | `services/PACKAGE.md` too verbose | Batch 4.4 (nested `tunnels/PACKAGE.md`) |
| 12 | Should be `self.is_native()` | Batch 2.4 |
| 13 | Hierarchical lookup duplicated | Batch 2.1 — **ordering corrected by verification** |
| 14 | Duplicate lookup / utility method | Batch 2.2 (**one claim refuted**) + 2.3 |
| 15 | Hand-rolled test setup | Batch 2.7 — largely already satisfied |
| 16, 17 | Follow the builder convention, no hidden intercepts | Batch 2.6 |
| 18 | Explain `envs.rs` | Answered below + Batch 2.8 |
| 19 | Not Windows-compatible | Batch 1 — **confirmed**, the fake is a `/bin/sh` script |
| 20, 21 | Injectable runtime, interface + two impls | Batch 1 |
| 22 | Move the prefix to a common place | Batch 1 — it is defined in exactly one place today |
| 23 | Are we still using a fingerprint? | Answered below — **no change needed**, plus a rename |
| 24 | Still writing a credentials JSON? | Answered below — **mostly already done**; residual in Batch 1 |
| 25 | Why log things that might have secrets? | Batch 1 — **you were right, we do** |
| 26 | Unix / non-Unix duplication | Batch 1 |
| 27 | Test through interfaces, not files and commands | Batch 1 |
| 28 | Slow test; use an event tap like the DB | Batch 1 |

Two notes lead to **pushback rather than a change** (#6, and part of #24); one of my own claims was **refuted during verification** (#14). Those are called out where they occur rather than quietly dropped.

## Notes that need an answer, not a change

Four notes are questions. Answering them is part of the deliverable; no code moves.

**#23 — "Are we using fingerprint? I thought we moved to client-id UUID."**
You are right, and the two things are unrelated. Tunnel identity is `bodhi-app-tunnel-<uuid>`, built by `tunnel_name_for()` (`service.rs:605`) from the standalone tenant's OAuth `client_id`, resolved once through a `OnceCell` (`service.rs:611`). The `fingerprint()` at `service.rs:248` is a `DefaultHasher` **cache key** used by `binary_status()` and `zone_fingerprint()` to decide whether a TTL-cached probe is still valid. It never touches naming.
*Small change worth making anyway:* rename `fingerprint` → `cache_key` so the word stops colliding with tunnel identity.

**#24 — "I thought we no longer use a physical JSON file."**
Mostly right. Both long-lived secrets already travel in the environment: the origin cert as `TUNNEL_ORIGIN_CERT` (`service.rs:807`) and the connector run token as `TUNNEL_TOKEN` (`service.rs:906`), never on argv — a test pins that (`test_tunnel_service.rs:500`). The one file left is unavoidable: `cloudflared tunnel create` has no form that emits a new tunnel's secret anywhere but `--credentials-file`. That scratch file is written under `$BODHI_HOME/tmp/tunnels/` with `0700`/`0600`, is **never read back by BodhiApp**, and is unlinked immediately whether `create` succeeded or failed (`service.rs:847`). Passing it as a CLI arg instead is explicitly forbidden by a locked constraint ("never pass a Cloudflare secret as a command-line argument" — process args are world-readable).
*Residual work:* it is a path, not a secret, so it belongs behind the IO trait — which the seam work does anyway.

**#1 — "Explain what the logic is here and why?"** (`routes_auth.rs:87`)
The handler is choosing which host the OAuth `redirect_uri` should point back to, from four candidates, in this order:
1. **Explicit `BODHI_PUBLIC_HOST`** wins outright — the operator has declared the public identity (RunPod-style deployments).
2. **Tunnel host**, but only when *both* `Host == BODHI_TUNNEL_HOST` *and* `X-Forwarded-Proto: https` are present → `https://<tunnel-host>/ui/auth/callback`.
3. **Raw `Host` header** combined with `public_scheme()`/`public_port()`.
4. `login_callback_url()` as an exhaustiveness fallback.

Why step 2 needs both conditions: a Cloudflare named tunnel terminates TLS at the edge and forwards **plaintext HTTP** to the loopback origin. The `Host` header arrives unmodified as the public hostname, so `Host` alone tells you the name but not the scheme — building from `Host` + the local scheme would emit `http://<tunnel-host>/…`, which Keycloak's redirect-URI allow-list rejects. `X-Forwarded-Proto` is the edge's statement that the client leg really was HTTPS. Requiring `Host` to equal the *configured* tunnel host as well stops a forged `X-Forwarded-Proto` from promoting an arbitrary host to HTTPS — a host-confusion / open-redirect guard scoped to the one hostname the admin enabled.

*Change worth making:* branches 3 and 4 are byte-identical in three places in that one function. Extract one helper and the whole thing reads as the four-way precedence it is.

**#18 — "What is the logic here? I don't understand what is happening here."** (`test_utils/envs.rs:261`)
`SettingServiceStub` is a flat `HashMap` fake with no precedence layers. To imitate the real service — where `public_host()`/`public_scheme()`/`public_port()` fall back to `host()`/`scheme()`/`port()` when unset — it takes a shortcut: on a miss, if the key starts with `"BODHI_PUBLIC_"`, it string-replaces that prefix with `"BODHI_"` and looks *that* up instead.

That worked by coincidence for exactly three keys. `BODHI_PUBLIC_URL_REACHABLE` rewrites to `BODHI_URL_REACHABLE`, which does not exist — and the old code did `.get(...).cloned().unwrap()`, so every test calling `url_public()` without explicitly setting the key **panicked**. The commit removed the `.unwrap()` and lets the `None` propagate. Six lines out, three in (one of them the fix, two a comment). **No test case or assertion was removed** — it is a crash fix, not a coverage cut.

*The real smell:* a prefix-guess standing in for an explicit mapping. It silently breaks for any future `BODHI_PUBLIC_*` key that is not a host/scheme/port. Replace the prefix match with an explicit three-key allowlist that mirrors the trait's own methods.

## Batches

Four commits on `main`, in this order. Each is independently reviewable and leaves the suite green.

---

### Batch 1 — Tunnel runtime seam

*Closes #19, #20, #21, #22, #24 (residual), #25, #26, #27, #28.* The largest batch; everything else is small.

**The problem, precisely.** `DefaultTunnelService` reaches the outside world in five ways, all inline:

| Surface | Where |
|---|---|
| One-shot `cloudflared` subcommands | `version_output` `:406`, `command_output` `:799` (`tokio::process`, `kill_on_drop`) |
| The supervised connector child | spawn block `:1417-1447`, `monitor_output` `:910` (`std::thread` readers), `stop_running` `:1008`, `stop_child` `:1019` |
| Filesystem | `read_origin_cert` `:439`, `scratch_credentials` `:774`, `discard_scratch` `:786` |
| HTTP | `zone_name` `:451`, `probe_ready` `:663`, DNS conflict/delete `~:1093`, `~:1153` |
| Environment | `discover_on_path` `:273` (`PATH`), `standard_binary_locations` `:290` (`ProgramFiles`) |

Because of this, `test_utils/tunnels.rs` generates a **POSIX `/bin/sh` script** at runtime (`SCRIPT`, `tunnels.rs:11-71`), `chmod 755`s it, and points `BODHI_TUNNEL_CLOUDFLARED_PATH` at it. It also shells out to `kill -0` to poll for process death (`tunnels.rs:144`). None of that runs on Windows — note #19 is simply correct. Beyond portability it is slow: `connector_stopped()` and `connector_token()` each poll 100 × 20 ms, and `shuts_down_while_a_provisioning_call_is_still_in_flight` makes the fake `sleep 3` for real to hold a call open.

**The seam.** Three traits in a new `crates/services/src/tunnels/runtime.rs`, each `#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]` per house convention (`env_wrapper.rs:7`, `llama_server_proc/src/server.rs:80`). Per the owner's refinement, **every method returns raw output and parses nothing**:

```rust
// crates/services/src/tunnels/runtime.rs
pub struct RawOutput { pub status: Option<i32>, pub stdout: Vec<u8>, pub stderr: Vec<u8> }

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait CloudflaredCli: std::fmt::Debug + Send + Sync {
  /// Invoke the binary and hand back what it wrote. Parses nothing.
  async fn run(&self, binary: &Path, args: &[String], envs: &[(String, String)],
               timeout: Duration) -> Result<RawOutput, TunnelRuntimeError>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ConnectorProcess: std::fmt::Debug + Send + Sync {
  fn spawn(&self, binary: &Path, args: &[String], envs: &[(String, String)])
    -> Result<Box<dyn ConnectorHandle>, TunnelRuntimeError>;
}

pub trait ConnectorHandle: std::fmt::Debug + Send {
  fn try_wait(&mut self) -> Result<Option<i32>, TunnelRuntimeError>;
  fn stop(&mut self) -> Result<(), TunnelRuntimeError>;
  /// Raw stdout/stderr lines. The caller decides what they mean.
  fn output(&mut self) -> Option<Receiver<(OutputStream, String)>>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait TunnelIo: std::fmt::Debug + Send + Sync {
  async fn read_to_string(&self, path: &Path) -> Result<String, TunnelRuntimeError>;
  async fn prepare_scratch_dir(&self, dir: &Path) -> Result<(), TunnelRuntimeError>;
  async fn discard_scratch(&self, path: &Path);
  fn discover_binary(&self) -> Vec<PathBuf>;        // PATH + platform locations
  async fn http(&self, method: HttpMethod, url: &str, bearer: Option<&str>)
    -> Result<(u16, String), TunnelRuntimeError>;   // Cloudflare API and /ready
}
```

Everything the current code does *around* these calls stays in `DefaultTunnelService` and becomes plain unit-testable logic: `read_origin_cert` splits into `TunnelIo::read_to_string` + a pure `parse_origin_cert`; the `tunnel create --output json` and Cloudflare envelopes are parsed from the returned `String`; `parse_metrics_addr` (already pure, `:979`) consumes the raw line stream; and the `/ready` `readyConnections > 0` decision is made from `(u16, String)` rather than inside a probe.

**What this buys, note by note:**
- **#26** — the `#[cfg(unix)]`/`#[cfg(not(unix))]` fork (the `/bin/sh` liveness-pipe wrapper vs. a direct spawn + `kill()`) collapses into the *one* real `ConnectorProcess` impl. `stop_child`'s second fork and the two binary-discovery forks go with it. Service logic becomes platform-free.

  **On "can a Rust library remove this?" — checked; recommendation is no, and the seam is what makes that safe.** `command-group` and `process_wrap` are not in the workspace; `shared_child` is already in `Cargo.lock` and `libc` is a direct dependency. None of them solves the actual requirement. What the `/bin/sh` wrapper buys is *child dies when the **parent** is `kill -9`'d* — demo step 7 of the slice prompt. Those crates give process groups and kill-on-drop, which cover graceful shutdown only; nothing runs in the parent after `SIGKILL` to do the killing. Linux has `prctl(PR_SET_PDEATHSIG)` via the `libc` we already depend on, but macOS has no equivalent — so adopting it would reshape the platform fork, not remove it. Keep the hand-rolled implementation, now isolated in one file behind `ConnectorProcess`, where swapping it later is contained. Worth recording as a property: because of the wrapper, the supervised PID on Unix is the shell, not `cloudflared`.

  **No orphan regression** (the slice prompt's explicit warning). `llama_server_proc` uses `std::process::Command` + `thread::spawn` readers because an async version once orphaned children. The tunnel connector already follows that — `RunningTunnel.child` is a `std::process::Child` with `std::thread` readers. `tokio::process` + `kill_on_drop` is used only for the short, bounded, awaited one-shot subcommands, which is not the case the orphan rule was about. The seam preserves this split rather than unifying it.
- **#22** — `RESOURCE_CLIENT_PREFIX = "bodhi-resource-"` is defined **only** at `service.rs:207` (verified: no other definition in the repo). It is an external Keycloak naming contract, so move it to `settings/constants.rs` beside the other cross-cutting prefixes and have `test_tunnel_service.rs:20` build `CLIENT_ID` from it instead of re-typing the literal.
- **#24** — the scratch-credentials path lifecycle moves behind `TunnelIo`; the file itself stays (see the answer above).
- **#23** — rename `fingerprint` → `cache_key` (`service.rs:248`, `:530`).
- **#25** — `scrub_secrets` (`service.rs:937`) moves to **`crates/services/src/shared_objs/log.rs`**, which already exists as the common log utility (`mask_sensitive_value`, `mask_form_params`, `log_http_request/response/error`) and is already what `auth_service.rs` uses. Then apply it **inside `log_http_error`**, which defends all 14 of its call sites in `auth_service.rs` at once — several of which pass raw upstream response text (`:346`, `:664`, `:749`, `:834`, `:918`). This is the honest answer to "ensure we are not passing secrets in the first place": we *are*. `update_tunnel_redirect_uri` falls back to `body.chars().take(500)` (`auth_service.rs:663`), putting an unparsed Keycloak body into a typed error that is then logged before anything scrubs it. The same `.text().await.unwrap_or_default()`-into-error shape appears across `mcp_service.rs` and `ai_apis/clients/*` — pre-existing and out of scope here; it gets a techdebt entry.

**Tests (#27, #28).** Delete `crates/services/src/test_utils/tunnels.rs` entirely. In `test_tunnel_service.rs`, the split is already favourable:
- **11 tests stay untouched** — they already drive pure associated functions (`validates_cloudflare_hostnames`, `tunnel_name_identifies_the_instance`, `connector_flags_precede_the_run_subcommand`, `reads_the_metrics_address`, `runtime_error_codes_are_stable`, `connector_credentials_travel_in_the_environment`, `reads_cloudflare_origin_certificate`, both `collapses_to_the_public_enum` cases, `scrubs_opaque_secrets`, `classifies_sync_failures`).
- **~16 harness-driven tests are rewritten** against the three mocks. No test spawns a process, writes a file, opens a socket, or sleeps.

For the supervisor tests, copy the pattern the DB layer already uses rather than inventing one: `TestDbService` (`test_utils/db.rs`) publishes a `tokio::sync::broadcast` event after every operation, and tests await it with the `wait_for_event!` macro at `models/test_progress_tracking.rs:14-27`. Give `TunnelRuntime` the same tap, move `wait_for_event!` into `test_utils/` so it is shared, and `the_supervisor_notices_readiness_without_anyone_polling_status` stops sleeping 250 ms × 80. `shuts_down_while_a_provisioning_call_is_still_in_flight` becomes instant: the mock CLI holds a `oneshot` instead of the fake `sleep 3`.

**Call sites.** `DefaultTunnelService::new`/`with_auth` gain the three dependencies — 5 construction sites: `lib_bodhiserver/src/app_service_builder.rs:213`, `services/src/test_utils/app.rs:297`, and `server_app/tests/utils/live_server_utils.rs:249,629,1014` (those three are byte-identical; collapse them into one helper while there).

---

### Batch 2 — Settings and test-utils hygiene

*Closes #12, #13, #14, #15, #16, #17, #18.*

> **Adversarial verification changed this batch.** Two independent verifiers checked the claims it rested on. One was **refuted**; another needs a strict ordering. The naive "delete the duplicated lookups" would have broken two currently-passing tests.

1. **Fix the stub first, then drop the redundant lookup** (#13). `tunnel_enabled()` and `url_public()` (`setting_service.rs:194-242`) each call `get_setting_value(KEY)` — which already runs the full precedence chain, consulting env at `default_service.rs:378` — and then add a second manual `get_env(KEY)` re-query of the *same* `env_wrapper`. For `DefaultSettingService` that second block is provably dead.

   **But it is load-bearing for `SettingServiceStub`**, whose `get_setting_value_with_source` (`test_utils/envs.rs:256-271`) reads only its `settings` map and never its `envs` map. `test_login_initiate.rs:168-170` puts `BODHI_TUNNEL` **only** in the stub's `envs` map — so deleting the fallback makes `tunnel_enabled()` return `false` and `case::tunnel` fails with an `http://…:1135/` callback instead of `https://…`.

   **Required order:** (a) make the stub consult its `envs` map at Environment precedence, matching production; (b) re-run `cargo test -p routes_app`; (c) *only then* delete the trait-level `get_env` fallbacks.
2. **Do NOT remove the terminal default — my earlier claim was refuted** (#14). I had it that `url_public`'s trailing `.unwrap_or(DEFAULT_PUBLIC_URL_REACHABLE)` was unreachable because the key has a registered default (`default_service.rs:248`). **Wrong.** `SettingMetadata::Boolean::parse` returns an unparseable value *unchanged* as `Value::String` rather than erroring (`setting_objs.rs:119-123`), so an env value like `"yes-please"` short-circuits in the Environment branch, never reaches the registered default, and leaves `configured = None` — landing on the literal fallback. `test_setting_service.rs` `#[case::unparseable_is_not_public]` exercises exactly this and passes today.

   **Corrected action:** keep an unconditional terminal default; remove only the redundant `get_env` re-query sitting in front of it. The "present but unparseable" behaviour is real and tested — preserve it.
3. **One bool reader** (#14). The `Value::Bool | Value::String("true")` coercion is copy-pasted in both new methods, and the trait already had three *other* idioms for the same thing (`canonical_redirect_enabled:549`, `on_runpod_enabled:567`). Add one `get_setting_bool(key) -> Option<bool>` trait default and route all four through it — keeping the "present but unparseable → `None`" semantics item 2 established, so the callers' own defaults still fire.
4. **Use `is_native()`** (#12). `tunnel_enabled`'s fallback compares `get_setting(BODHI_APP_TYPE)` against the raw literal `"native"`, duplicating `AppType`'s strum serialization. `is_native()` already exists directly above it at `setting_service.rs:190`. Call it. Caveat to handle explicitly: `app_type()` does `.parse().unwrap()` and panics on a malformed value, whereas the literal compare degrades silently — so add a non-panicking `app_type` path rather than a silent string fork.
5. **Sync the stub's metadata table** (#15 adjacent) — **confirmed by both verifiers.** `default_service.rs` classifies five keys as `SettingMetadata::Boolean` (`BODHI_LOG_STDOUT:52`, then `BODHI_CANONICAL_REDIRECT | BODHI_PUBLIC_URL_REACHABLE | BODHI_TUNNEL | BODHI_TUNNEL_AUTO_RECONNECT` at `:61-64`). The stub's `get_metadata` (`test_utils/envs.rs:293-304`) classifies only two — the three tunnel-era keys fall through to `_ => String`, and the file does not even import them. Add them, plus a regression test that round-trips `"true"`/`"false"` through the stub and asserts a real `Value::Bool` comes back, so the two tables cannot drift silently again.
6. **Follow the builder convention** (#16, #17). `AppServiceStubBuilder::build` (`test_utils/app.rs:103-116`) inlines `self.setting_service = Some(self.default_setting_service())` next to three calls that use the established named-method form (`with_db_service`, `with_session_service`, `with_tenant_service`). Add `with_setting_service()` and call it in the same ordered list, so the dependency is declared the way the other three are instead of as an inline intercept.
7. **Reuse the in-file fixture** (#15). The new settings tests at `test_setting_service.rs:1006+` correctly reuse `make_service_from_parts`, `noop_settings_repo`, `bodhi_home_setting` and `EnvWrapperStub` — so this note is largely already satisfied. The one gap: they inline a `Setting { key: BODHI_APP_TYPE, … }` literal where the file's own `env_type_system_setting()` helper (`:1122`) sets the precedent. Add the sibling `app_type_system_setting()` and use it.
8. **Replace the prefix guess** (#18). In `test_utils/envs.rs`, swap the `key.starts_with("BODHI_PUBLIC_")` string-rewrite for an explicit three-key allowlist (`HOST`, `SCHEME`, `PORT`) mirroring the trait's own `public_host`/`public_scheme`/`public_port`, so the next `BODHI_PUBLIC_*` key cannot silently resolve to a non-existent counterpart.

---

### Batch 3 — routes_app host logic and tests

*Closes #2, #3, #4, #6.*

1. **Restore the lost case and make the assertions non-vacuous** (#2, #3, #4) — **all three confirmed by both verifiers, one via explicit mutation analysis.**
   - **Coverage was genuinely lost.** The deleted `test_auth_initiate_handler_network_host_usage` asserted on `"192.168.1.100:1135"`. The current file contains **no dotted-quad literal at all**; `#[case::network_ip]` passes `"localhost:1135"` — which is the *loopback* path the other deleted test covered.
   - **The case is mislabelled**: `network_ip` exercises a hostname.
   - **The tunnel-match assertion is vacuous.** Both tunnel cases pass the identical literal `"tunnel.example.com"` as *both* `tunnel_host` and `request_host`. And `case::configured_but_disabled_tunnel` sets `tunnel_enabled=false`, so it never reaches the equality check at all — leaving exactly one case that does, trivially. A verifier traced the mutation: replace the `request_host == tunnel_host` comparison with a hardcoded `true` and **all three cases still pass**, because the format string downstream interpolates the real `tunnel_host` regardless.

   Fix: rename `network_ip` → `loopback`; add a real LAN-IP case; add a **host-mismatch** case (`tunnel_enabled=true`, `tunnel_host="tunnel.example.com"`, `request_host="other.example.com"`, `x-forwarded-proto: https`) asserting it falls through to the untrusted branch; and add a **matching-host-but-not-https** case for the other half of the `&&`. Those last two are the cases that actually encode the host-confusion guard described under #1.
2. **Collapse the triplicated fallback** (#1's cleanup). Branches 3 and 4 of the callback-URL selection are byte-identical in three places inside one function (`routes_auth.rs:87-124`). Extract one helper; the function then reads as the four-way precedence it is.
3. **Router test helper — scoped honestly** (#6). `Router::new()` appears **208 times** across `crates/routes_app/src`, including 18 times in `test_setup.rs` alone, and `test_utils/router.rs` only offers the heavyweight `build_test_router()` (full app, real services). So inline construction is the crate's long-standing convention, not something this commit introduced — extracting it everywhere would be a crate-wide refactor well outside this review. Scoped proposal: add a small `single_route_router(path, method_router, state)` helper to `routes_app/src/test_utils/router.rs` and adopt it **in `test_setup.rs` only**. If that reads well, adopting it elsewhere becomes a separate mechanical pass.

---

### Batch 4 — Comments, docs, and the company domain

*Closes #5, #7, #8, #9, #10, #11.*

1. **Comment sweep** (#5, #11). Run the `comments-cleanup` skill over the files `70b221d5` touched, against the rule the slice-1 prompt already stated: *non-obvious why only, one or two lines*.

   One calibration note, because a first pass disagreed with itself on this: judged as "is it why-content?", almost every comment the commit added passes — they explain rationale, not mechanics. Judged against *your* rule, which also bounds length and forbids duplication, several fail. A four-line doc comment on a constant is a **tighten**, not a keep. The clearest cases are `constants.rs:22-26` and `:35-38`, and `:35-38` restates what `tunnel_enabled`'s own doc comment (`setting_service.rs:194`) already says. `service.rs` adds ~34 comment lines across 12 blocks and `tunnel_objs.rs` ~21 — that is the bulk of the sweep. Expect mostly tightening, not deletion; say so honestly in the commit rather than claiming a purge.

   Also worth knowing: `routes_app/src/tunnels/routes_tunnels.rs` has **zero** comments and needs nothing.
2. **Write the rule down.** There is currently **no comment policy anywhere in the repo** — not in `CLAUDE.md`, `crates/CLAUDE.md`, `MDFILES.md`, or `docs/conventions/`. That is why this keeps recurring. Add two lines to `crates/CLAUDE.md` so the next session inherits it instead of the reviewer re-deriving it.
3. **De-churn the docs** (#8, #9). `MDFILES.md` already names "information that changes frequently" as LOW-VALUE, and the evidence is stark: the AppService accessor count is stated as **18** (`services/PACKAGE.md:9`), **20** (`services/PACKAGE.md:168`) and **21** (`services/CLAUDE.md:49` — bumped by this commit). Three numbers, two files, one fact. Remove the counts and point at the trait definition instead. Same treatment for the other hits (`bodhi/src/CLAUDE.md:108`, `services/src/test_utils/PACKAGE.md:41`, `routes_app/PACKAGE.md:185`, `routes_app/CLAUDE.md:47`, and at least one in `techdebt.md`).
4. **Progressive disclosure for the tunnel docs** (#10). Move the seven verbose bullets this commit added to `services/PACKAGE.md:77-88` into a new `crates/services/src/tunnels/PACKAGE.md` (+ `CLAUDE.md`), leaving a one-line pointer behind. This follows existing precedent rather than inventing a tier — `routes_app/src/middleware/`, `crates/bodhi/src/`, `services/src/test_utils/` and `server_core/src/test_utils/` all already carry nested pairs.
5. **Remove the company domain** (#7). **24 occurrences across 7 source files — all test fixtures or planning docs, no production code.** `example.com`-style replacements throughout, keeping fixture/assertion pairs consistent: `test_setup.rs:195` and `:210` must change together, and the msw handlers in `crates/bodhi/src/test-utils/msw-v2/handlers/tunnels.ts` must stay in sync with `routes/tunnels/index.test.tsx` and `remoteAccessState.test.ts`, which assert on their values. Files: those three TS files, `routes_app/src/setup/test_setup.rs`, `services/src/tunnels/test_tunnel_service.rs` (two *negative* `validate_hostname` cases — trivially safe), and `docs/claude-plans/202609/remote-access-{plan,info-discovery}.md`.

   Two spots need judgement rather than substitution:
   - `remote-access-plan.md:424-425` states a real Cloudflare constraint — Universal SSL covers a zone and its *single-level* wildcard but not multi-level names. The technical point survives the placeholder intact; rewrite, don't drop.
   - `remote-access-plan.md:199-203` and `:418-422` are an **operational record** of tunnels and CNAMEs actually deleted from a real account on 2026-09-18. Substituting makes the record non-literal. Your instruction is unambiguous ("not mentioned anywhere"), so substitute and add a one-line note that hostnames were placeholder-substituted — flagging it because it is the one place the sweep costs something.

   Note `getbodhi.app` also appears nearby; that is the public product domain and is left alone.

   **This plan file is itself in scope for the sweep** once written — it currently names the domain once, above.

---

## Verification

Gate checks, in full — none skipped.

**Per batch, before committing:**
1. `make format`
2. Upstream-to-downstream, as the crate chain requires: `cargo test -p services` → `cargo test -p routes_app` → `cargo test -p server_app` → `cargo test -p lib_bodhiserver`
3. `make test.backend`, **teed to a file and grepped** — it is slow and must not be re-run just to re-read its output
4. `cd crates/bodhi && npm test` for Batch 4 — the msw handlers and the component tests share the domain literals, so they must change together or the UI suite breaks
5. `graphify update .` so the knowledge graph lands in the same commit

**Batch-1 specific — the claim being made is "these tests no longer touch the outside world", so prove it:**
- Time `cargo test -p services tunnels` before and after; record both numbers. The ~16 rewritten tests should drop from seconds to milliseconds.
- Grep the rewritten `test_tunnel_service.rs` for `sleep`, `Command`, `spawn`, `TempDir` and `std::fs` — the expected result is zero hits.
- Confirm `test_utils/tunnels.rs` is deleted and nothing references `FakeCloudflared`.

**No API shapes change in any batch**, so `openapi.json` and the TypeScript client should regenerate byte-identical. Run `cargo run --package xtask openapi && cd ts-client && npm run generate` and confirm an empty diff — a non-empty one means a batch changed a contract it should not have.

**Not run:** `make test.e2e` — there is no tunnel or Remote Access E2E spec to exercise (see below), and no batch touches an existing spec.

## Deliberately out of scope

- **Test layers above `services`.** No `server_app` tunnel integration test and no Playwright spec. Already deferred with reasons in `docs/claude-plans/techdebt.md` → *Remote Access* → "No end-to-end coverage". **Note the seam does *not* unblock it:** that item is blocked on the Cloudflare API base URL being injectable into a *real out-of-process server*, which `mockall` cannot do — it still needs the non-production setting the owner declined. Batch 1 changes nothing there; the techdebt entry stands as written.
- **The repo-wide `.text() → typed error` audit.** `mcp_service.rs` and `ai_apis/clients/*` carry the same raw-body-into-error shape as `auth_service.rs`. Batch 1 defends the log path centrally; auditing every construction site is its own pass → new techdebt entry.
- **Extracting a router helper across all 208 sites** (#6) — scoped to `test_setup.rs`, rationale above.
- **Amending `70b221d5`** — follow-up commits by owner decision.

## Observations, no action

- `crates/bodhi/.tool-versions` (node 22.14.0 → 24.16.0) is unrelated to this feature, but it is a *correct* catch-up: `d4268ddb` bumped the root file on 2026-09-13 and left the nested one stale. Already committed and right; noted only because it violates the "keep commits focused" rule.
- `crates/routes_app/src/tunnels/routes_tunnels.rs` is clean — 6 thin handlers, zero comments, nothing to change.
- `slice-1-job-done.md` is now stale against the shipped code: it describes a credentials-file run invocation, which `70b221d5` replaced with `TUNNEL_TOKEN`. Worth a one-line correction when someone next touches that folder.

## Execution note

This file was created at `docs/claude-plans/202607/` by the harness, but by the folder convention (`docs/claude-plans/CLAUDE.md`) it belongs in `202609/` — today is 2026-09-19. First step of execution: move it to `docs/claude-plans/202609/tunnel/` and add its `index.md` entry there, dated today.
