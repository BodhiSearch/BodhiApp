# 24 — Frontend (Settings V2) and Playwright E2E map for Remote Access (Cloudflare Tunnel)

**Date:** 2026-09-15
**Scope:** Named tunnels only (per product decision — quick tunnels and Tailscale are out of scope). This corrects the "both, phased" recommendation in `bodhiapp-cloudflare-tunnel-feasibility.md`; that doc's Phase 1 (quick tunnels) should be treated as superseded, not a build target.

All paths below are relative to the repo root unless noted. `crates/bodhi/src/*` is the frontend; `crates/lib_bodhiserver/tests-js/*` is the Playwright suite.

## 1. There is no backend contract yet — this doc plans against the gap

Confirmed in this pass (not previously stated this precisely): `AppInfo` (`ts-client/src/types/types.gen.ts:238-267`) has a single `url: string` field, no `origins` list and no `app_type`/`is_native` flag. The frontend has **zero** existing Tauri/native detection (`grep -rln "__TAURI__\|isTauri\|app_type\|AppType" crates/bodhi/src` → no hits outside tests). So:

- The "list all origins, flag the tunnel one" UI (from the task brief) needs a backend contract addition — e.g. `AppInfo.origins: { url: string; kind: 'loopback'|'lan'|'public'|'tunnel' }[]` — before the Remote Access screen can render it from `useGetAppInfo()`.
- Gating the nav entry to native-only-by-default needs either a new `AppInfo` field or a dedicated `/bodhi/v1/tunnel/status` response to carry an `available: boolean` (native + flag-enabled) the frontend can check — today the frontend cannot tell native from container at all.
- Only one existing consumer of `AppInfo.url` will be touched by any origins-list contract change: `routes/users/-components/InviteLinkAction.tsx:19` (`` `${appInfo.url}/ui/login/?invite=${appInfo.client_id}` ``) — keep `url` as the primary/public origin so this call site doesn't need to change.

## 2. (a) Where the Tunnel screen lives

### Settings V2 today is a flat key=value list — not a fit for a multi-step wizard

`routes/settings/index.tsx` + `routes/settings/-components/SettingsPageV2.tsx` render one screen: a searchable/filterable list of individual `SettingInfo` rows (`useListSettings()`, `hooks/settings/useSettings.ts:9-11`) grouped by a static `SETTINGS_CONFIG` (`routes/settings/index.tsx:32-113`), each row opening a generic `SettingRailPanel` (`routes/settings/-components/SettingRailPanel.tsx`) via `useShellChrome` (`SettingsPageV2.tsx:215`). This is right for individual `BODHI_*` env-style settings; it is not right for a stateful, polling, multi-step "Remote Access" screen (Connect → Detect → Provision → Status).

Precedent for "rich sub-screen under a nav group, not crammed into the generic list": the `settings` nav group already sits *beside* `mcp`/`users`/`api-keys` groups that each have their own multi-page domain (`components/shell/shell-nav-config.tsx:34-112`). **Recommendation: give Remote Access its own route+nav sub-page, sibling to `app-settings`, inside the existing `settings` nav group** (it's conceptually an instance-wide admin toggle, matching the product decision that "an admin enables it for the whole instance").

### Nav entry

Add to `SHELL_NAV` settings group's `subPages` array, `components/shell/shell-nav-config.tsx:106-112`:

```ts
{
  id: 'settings',
  label: 'Settings',
  icon: 'settings',
  href: '/settings/',
  subPages: [
    { id: 'app-settings', label: 'App Settings', icon: 'settings', href: '/settings/' },
    { id: 'remote-access', label: 'Remote Access', icon: 'radio-tower', href: ROUTE_SETTINGS_TUNNEL, adminOnly: true },
  ],
},
```

`adminOnly: true` and (initially, until desktop/container is exposed via API) `hideInMultiTenant: true` are both enforced client-side at `components/shell/ShellNav.tsx:33-38` (`!(isMultiTenant && sp.hideInMultiTenant) && !(sp.adminOnly && !isAdmin)`, sourced from `useGetAppInfo().deployment` and `useGetUser().role`/`isAdminRole` at `ShellNav.tsx:27-30`) — this is purely a nav-hiding filter, **not** an authorization boundary; the route itself and its API calls must still be admin-gated server-side (`admin_session_apis`, per `01-bodhi-app-codebase-map.md §4`).

Add the route constant next to the other domain constants, `src/lib/constants.ts:31-33`:
```ts
export const ROUTE_SETTINGS_TUNNEL = '/settings/remote-access/';
```

### Route file + page shell

New file `crates/bodhi/src/routes/settings/remote-access/index.tsx`, following the same shape as `routes/settings/index.tsx:7-10`:
```ts
export const Route = createFileRoute('/settings/remote-access/')({
  staticData: { section: 'settings', subPage: 'remote-access' },
  component: RemoteAccessPage,
});
```
Wrap with `AppInitializer authenticated allowedStatus="ready"` exactly like `routes/settings/index.tsx:130-136`. Sub-components go in `routes/settings/remote-access/-components/` (TanStack ignores `-`-prefixed dirs, per `crates/bodhi/src/CLAUDE.md` "Sub-components co-located in `-components/`").

Chrome: publish via `useShellChrome({ breadcrumb, sidebar, rail, railHeader })` (same hook as `SettingsPageV2.tsx:215`), not a bespoke layout. For the right-hand detail rail (origins list, Keycloak sync status, cloudflared version), **use the shared `DetailRail` primitives** (`components/detail-rail/DetailRail.tsx:5-51`: `DetailRail`, `DetailRailBody`, `DetailRailSection`, `DetailRailRows`, `DetailRailRow`) instead of writing new rail markup — that file's own comment says it exists specifically to de-duplicate a `Row` helper that nine other rail panels each re-declared (`DetailRail.tsx:3`); a tenth reimplementation would repeat exactly what it was built to stop.

### Component patterns to copy for the step flow

No dedicated "Steps"/wizard component exists in `components/ui/` (only `components/ui/badge.tsx`). The closest working precedents, both worth copying directly rather than inventing new UI language:

- **Connect/action card** — `routes/mcps/new/-components/OAuthConnectPanel.tsx:1-46`: a summary `<div className="rounded-lg border p-3 ...">` + a `Button` with `Loader2` spinner while `isConnecting`, `data-testid="auth-config-oauth-connect"`. Use this shape for "Provision named tunnel" / "Sync Keycloak redirect URI" actions.
- **Status pill** — `routes/mcps/playground/-components/ConnectionStatus.tsx:21-50`: `<span className={`pg-pill ${tone}`} data-testid="..." data-test-state={status}>` with a `LABEL` record keyed by a status union, `Loader2` spinner while transitional. This is the right pattern for tunnel status (`disabled | detecting | missing_binary | provisioning | active | error`), and it already follows the playwright skill's `data-test-state` convention.
- **Simple status chip** (alternative, lighter-weight) — `routes/users/access-requests/-components/StatusChip.tsx:1-11`: `<span className="ua-status {status}" data-testid="request-status-{status}">` + `ShellIcon`.

## 3. Hook files to add (`hooks/tunnel/`, camelCase per `feedback_hook_file_naming.md`)

Mirrors the `hooks/settings/` and `hooks/mcps/useMcpOAuth.ts` shape exactly (both confirmed read this pass):

- `hooks/tunnel/constants.ts` — endpoints + query-key factory, same shape as `hooks/settings/constants.ts:1-8`:
  ```ts
  export const ENDPOINT_TUNNEL_STATUS = `${BODHI_API_BASE}/tunnel`;
  export const ENDPOINT_TUNNEL_ENABLE = `${BODHI_API_BASE}/tunnel/enable`;
  export const ENDPOINT_TUNNEL_DISABLE = `${BODHI_API_BASE}/tunnel/disable`;
  export const ENDPOINT_TUNNEL_CLOUDFLARED = `${BODHI_API_BASE}/tunnel/cloudflared`; // detect/provision
  export const tunnelKeys = { all: ['tunnel'] as const };
  ```
  (Exact endpoint paths are for the backend-API research stream to settle under `admin_session_apis`, per `01-bodhi-app-codebase-map.md §4`; naming here just follows the `routes_app` `ENDPOINT_*` / hooks `constants.ts` convention.)
- `hooks/tunnel/useTunnelStatus.ts` — `useGetTunnelStatus(options?: { enablePolling?: boolean })` wrapping `useQuery<TunnelStatus>(tunnelKeys.all, ENDPOINT_TUNNEL_STATUS, undefined, { refetchInterval: options?.enablePolling ? 2000 : false, refetchIntervalInBackground: true })`. This is a direct copy of the **only** existing "toggleable live-status polling" hook in the codebase, `useListDownloads` (`hooks/models/useDownloads.ts:24-35`, `refetchInterval: options?.enablePolling ? 1000 : false`) — same pattern the task brief points at ("model download progress ... to copy for tunnel status polling").
- `hooks/tunnel/useTunnelActions.ts` — `useEnableTunnel()`, `useDisableTunnel()`, `useProvisionCloudflared()` as `useMutationQuery` calls that `invalidateQueries({ queryKey: tunnelKeys.all })` on success, modeled on `useUpdateSetting`/`useDeleteSetting` (`hooks/settings/useSettings.ts:13-51`) and `useOAuthLogin` (`hooks/mcps/useMcpOAuth.ts:61-78`).
- `hooks/tunnel/index.ts` — barrel, same shape as `hooks/settings/index.ts:1-2`.

`hooks/useQuery.ts:19-88` (`useQuery<T>`, `useMutationQuery<T,V>`) is the shared generic layer these build on — no changes needed there.

## 4. (b) MSW handler + component test plan

### MSW handler: `test-utils/msw-v2/handlers/tunnel.ts`

Copy the `typedHttp` + `components['schemas'][...]` pattern verbatim from `handlers/settings.ts:1-22` and `handlers/info.ts:1-40` (both read this pass):

```ts
export function mockTunnelStatus(overrides: Partial<components['schemas']['TunnelStatus']> = {}, { delayMs, stub }: { delayMs?: number; stub?: boolean } = {}) { ... }
export function mockTunnelStatusDisabled() { return mockTunnelStatus({ state: 'disabled' }); }
export function mockTunnelStatusMissingBinary() { return mockTunnelStatus({ state: 'missing_binary' }); }
export function mockTunnelStatusProvisioning() { return mockTunnelStatus({ state: 'provisioning' }); }
export function mockTunnelStatusActive() { return mockTunnelStatus({ state: 'active', public_url: 'https://bodhi.example.com' }); }
export function mockEnableTunnel() { return [typedHttp.post(ENDPOINT_TUNNEL_ENABLE, ...)]; }
export function mockTunnelError(...) { /* mirrors mockSettingsError, settings.ts:72-90 */ }
```

`components['schemas']['TunnelStatus']` does not exist yet — it's generated from the OpenAPI spec (`cargo run --package xtask openapi && cd ts-client && npm run generate`, per root `CLAUDE.md`), so this handler file is blocked on the backend-API stream defining and generating that schema first. If a `tunnel/{id}` sub-resource shape is used, mind the documented MSW v2 ordering gotcha (`crates/bodhi/src/CLAUDE.md` "MSW v2 Handler Ordering": sub-path handlers registered before wildcard `:id` handlers).

**Schema finalized in `21-codebase-settings-network-and-info.md` §6, use that verbatim** — correct this file's naming before implementing: the response type is `TunnelStatusResponse` (not `TunnelStatus`, which is the lifecycle *enum* nested inside it as `status`), the field is `status` (not `state`, and it has no `missing_binary`/`active` variants — those are `disabled | provisioning | enabled | error`; "missing binary" is reported separately by `GET /bodhi/v1/tunnel/detect`'s `CloudflaredDetection`, polled independently from tunnel status), and the endpoint is `ENDPOINT_TUNNEL` (`GET /bodhi/v1/tunnel`, not `ENDPOINT_TUNNEL_STATUS`). Every `state:`/`'active'`/`'missing_binary'` reference below (mocks, hook, component tests) needs updating to match.

### Component tests

- `routes/settings/remote-access/index.test.tsx`, modeled directly on `routes/settings/index.test.tsx:1-60` (`ShellHarness` + `createWrapper()` + `setupMswV2()`, seeding `mockAppInfoReady()` + `mockUserLoggedIn({ role: 'resource_admin' })`): one test per `TunnelStatus.state` — disabled (shows Enable CTA), `missing_binary` (shows install guide, `data-testid="tunnel-cloudflared-missing"`), `provisioning` (status pill `data-test-state="provisioning"`, polling active), `active` (shows public URL + Keycloak-sync indicator), `error` (shows retry).
- `hooks/tunnel/useTunnelStatus.test.ts` / `useTunnelActions.test.ts` — same shape as `hooks/settings/useSettings.test.ts` and `hooks/info/useInfo.test.ts`: assert query key, the `enablePolling` → `refetchInterval` toggle, and mutation → `invalidateQueries` on success.
- `components/shell/ShellNav.test.tsx` (127 lines today, **no existing `adminOnly` coverage** — confirmed via read, this is a genuine gap, not a copy target) — add a case asserting the new `remote-access` sub-page is filtered out for a non-admin role and for `multi_tenant` deployment, exercising the existing filter at `ShellNav.tsx:33-38` rather than new logic.

## 5. (c) E2E design

### One growing spec, `test.step` blocks, POM reuse

New spec `crates/lib_bodhiserver/tests-js/specs/settings/remote-access-tunnel.spec.mjs`, shaped like `specs/settings/network-ip-setup-flow.spec.mjs:1-40` (`beforeAll` builds `authServerConfig`/`testCredentials`/`authClient` via `getAuthServerConfig()`/`getTestCredentials()`/`createAuthServerTestClient()`, then `createServerManager(serverConfig)` per test). New page object `pages/RemoteAccessTunnelPage.mjs extends BasePage`, using the existing black-box nav helper `navViaShell('settings', 'remote-access')` (`pages/BasePage.mjs:27-48` — already generic over `section`/`subPage`, no change needed) instead of `page.goto()`.

Per the user's own E2E conventions already in memory (not re-derived here, just applied): **black-box only** (no `page.evaluate`/context `fetch`), **never `test.skip()` for missing env** — `throw` in `beforeAll`/`beforeEach` instead (exactly as `network-ip-setup-flow.spec.mjs:38-40` throws when `getLocalNetworkIP()` returns null), and **no if/else branching inside one test** — split by state into separate `test()` blocks instead of branching on env within a shared test body.

### Steps runnable with a fake `cloudflared` stub (every CI run, no Cloudflare account needed)

1. `detect: cloudflared missing` — spawn the server with a `PATH`/`BODHI_TUNNEL_CLOUDFLARED_PATH` that resolves to nothing; assert the UI's `missing_binary` state and install-guide copy.
2. `detect: cloudflared found via stub` — spawn with `BODHI_TUNNEL_CLOUDFLARED_PATH` pointing at the checked-in fake binary; assert the UI reads back a version string the stub prints for `cloudflared --version` (or equivalent detect invocation the backend settles on).
3. `guide: install instructions render` — pure UI assertion, no process interaction.
4. `enable: provisions and starts` — click Enable; the fake binary writes a canned tunnel URL to its stdout/log immediately (no real network call) so the Rust-side process-manager's log-scraping has something to match — the same technique `waitForListening`'s stdout regex already uses for `bodhiserver_dev` itself (`test-helpers.mjs:144-179`, `bodhiserver_dev: listening on (\S+)`), so the tunnel supervisor is expected to use an analogous pattern. Assert status flips `disabled → provisioning → active`.
5. `status polling reflects fake tunnel URL` — assert the status pill's `data-test-state="active"` and the displayed URL matches the stub's canned value.
6. `Keycloak redirect URI synced` — after step 4, reuse the **existing** admin-token + client-lookup flow already built for this exact purpose: `auth-server-client.mjs`'s `addRedirectUri()` (`utils/auth-server-client.mjs:466-516`) does `GET /admin/realms/{realm}/clients?clientId=...` then `PUT .../clients/{id}`; write a small `getClientRedirectUris()` read-only sibling (same `GET`, no `PUT`) and assert the fake tunnel URL is present in `redirectUris` — this directly exercises the "sync only on enable/disable" requirement without needing a new backend introspection endpoint.
7. `disable: stops process and clears status` — click Disable; assert `state: 'disabled'` and (extending step 6's helper) that the redirect URI was removed.

All of 1–7 need **zero** real Cloudflare credentials and should run on every PR.

### Step requiring real Cloudflare (opt-in, not every PR)

8. `named tunnel against a real Cloudflare test zone` — `cloudflared` resolved from real `PATH` (not the stub), real `INTEG_TEST_CLOUDFLARE_API_TOKEN`/`_ACCOUNT_ID`/`_ZONE_ID`/`_DOMAIN`; creates a tunnel + DNS record on a disposable subdomain, asserts the public HTTPS URL round-trips a request to the server's `/ping`, then tears the tunnel/DNS record down in a `finally`/`afterEach` so a dedicated CI test zone doesn't accumulate stale records across failed runs.
- Gate this behind the same opt-in mechanism the suite already has for external-dependency tests: `playwright.config.mjs:16,27-28`'s `isScheduledRun` / `@scheduled` tag (`grepInvert: /@scheduled/` unless `--grep @scheduled` is passed) — tag it `@scheduled` or a new `@cloudflare-live`, run on a cron workflow rather than every PR, matching the existing "external Keycloak flakiness" mitigation pattern already in the user's own memory (`feedback_e2e_external_keycloak_flakiness.md`).
- Still **throw**, never skip, when the Cloudflare env vars are absent in a run that explicitly requested this tag.

### Is a real named-tunnel E2E realistic?

Plausible but non-trivial new infra, not just test code:
- **No `CLOUDFLARE_*` secrets exist today** — confirmed by reading `.github/workflows/playwright.yml:130-151` (only `INTEG_TEST_CLIENT_ID/SECRET`, Keycloak users, `OPENAI`/`OPENROUTER` keys) and `crates/lib_bodhiserver/tests-js/.env.test.example` (same set, no Cloudflare entries). Adding this step means: new GitHub repo secrets (`INTEG_TEST_CLOUDFLARE_API_TOKEN`, `_ACCOUNT_ID`) and vars (`INTEG_TEST_CLOUDFLARE_ZONE_ID`, `_DOMAIN`), wired into the workflow's `env:` block the same way the existing ones are, plus a new `.env.test.example` placeholder line.
- **Needs a dedicated, disposable Cloudflare zone** — never the production `getbodhi.app` zone — so a failed teardown in CI can't leave stray DNS records on a domain real users depend on.
- **Runner egress to Cloudflare's edge for the QUIC/HTTP2 tunnel protocol** — UNVERIFIED in this repo (no existing spec makes an outbound tunnel-protocol connection from a GitHub-hosted runner); should work in principle (outbound-only, same as any HTTPS client) but confirm empirically before relying on it in CI, not just locally.

## 6. (d) Stub binary: passing the path + shipping it in `tests-js/`

### How the server manager passes it through

`BodhiAppServer.startServer()` (`utils/bodhi-app-server.mjs:25-47`) spawns `BODHISERVER_DEV_BIN` with `env: { ...process.env, ...env }` (line 34), where `env` comes from `buildEnvFromConfig(this.serverConfig)` (`test-helpers.mjs:54-125`). That function already forwards **arbitrary** keys two ways without any code change needed for a single new var:
- `envVars` (spread first into `env`, `test-helpers.mjs:82,88`) — "Unknown keys pass through verbatim so callers can still inject ad-hoc INTEG_TEST_* vars" (the file's own comment, `test-helpers.mjs:52`).
- `systemSettings` (spread **last**, so it can override anything else, `test-helpers.mjs:83,106`).

So a spec does, with no `test-helpers.mjs` edit required:
```js
const serverManager = createServerManager({
  ...SetupFixtures.getServerManagerConfig(authServerConfig, port),
  envVars: { BODHI_TUNNEL_CLOUDFLARED_PATH: FAKE_CLOUDFLARED_BIN },
});
```
For reuse across the growing spec's `test.step`s, add a small factory to `fixtures/setupFixtures.mjs` (sibling to `getNetworkIPServerConfig`, `fixtures/setupFixtures.mjs:30-38`) — or a new `fixtures/tunnelFixtures.mjs`, matching the existing per-domain fixture-file split (`fixtures/mcpFixtures.mjs` is the precedent for a dedicated domain fixture file rather than overloading `SetupFixtures`).

### Shipping the stub

Ship it as a checked-in script under `tests-js/fixtures/bin/fake-cloudflared.mjs` (Node, `#!/usr/bin/env node` shebang, `chmod +x` so it's directly spawnable) or `.sh` — sibling to the `fixtures/` domain files, and directly analogous to the pattern already used for `test-mcp-oauth-server/`, `test-mcp-auth-server/` (checked-in Node fake servers under `tests-js/` that stand in for external services, wired as `webServer` entries in `playwright.config.mjs:116-155`). Resolve its absolute path the same way `BODHISERVER_DEV_BIN` is resolved (`test-helpers.mjs:18-26`, `join(__dirname, ...)`), and export it as a new `FAKE_CLOUDFLARED_BIN` constant alongside `BODHISERVER_DEV_BIN` in `test-helpers.mjs`'s export list (`test-helpers.mjs:217-229`).

**Open dependency, not an E2E-layer decision alone:** the stub's exact CLI surface (what `--version` prints, what `tunnel run --token ...` / `tunnel login` need to print and to which stream, whether it daemonizes or stays foreground) must match whatever the Rust-side `cloudflared` subprocess manager actually spawns and scrapes — that manager doesn't exist yet (`01-bodhi-app-codebase-map.md §8`: "greenfield module"; `20-codebase-child-process-and-app-lifecycle.md` covers the closest existing precedent, the `llama_server_proc` child-process pattern). Pin the stub's contract down together with that implementation, not independently — a stub built to guesses now will need rewriting once the real supervisor's log-scraping regex and argv shape are fixed.

**Cross-reference:** this is now pinned — see `10-cloudflared-cli-named-tunnel-lifecycle.md`, "Follow-up: Pin the exact cloudflared invocation..." section (§A–D at the end of that doc) for the definitive argv, the `/ready`-polling-is-primary decision (log lines are a diagnostic fallback only), and the exact stub behavior (§D) `fake-cloudflared.mjs` must implement, including binding a real `/ready` HTTP listener.

## 7. Summary table — new/changed files

| File | Change |
|---|---|
| `crates/bodhi/src/lib/constants.ts` | add `ROUTE_SETTINGS_TUNNEL` |
| `crates/bodhi/src/components/shell/shell-nav-config.tsx` | add `remote-access` subPage under `settings` |
| `crates/bodhi/src/routes/settings/remote-access/index.tsx` (new) | route + page shell |
| `crates/bodhi/src/routes/settings/remote-access/-components/*.tsx` (new) | step components, reusing `OAuthConnectPanel`/`ConnectionStatus`/`DetailRail` patterns |
| `crates/bodhi/src/hooks/tunnel/{constants,useTunnelStatus,useTunnelActions,index}.ts` (new) | data layer |
| `crates/bodhi/src/test-utils/msw-v2/handlers/tunnel.ts` (new) | MSW mocks — **blocked on OpenAPI schema for `TunnelStatus`** |
| `crates/bodhi/src/routes/settings/remote-access/index.test.tsx`, `hooks/tunnel/*.test.ts`, `components/shell/ShellNav.test.tsx` (extend) | component/unit tests |
| `crates/lib_bodhiserver/tests-js/specs/settings/remote-access-tunnel.spec.mjs` (new) | growing E2E spec |
| `crates/lib_bodhiserver/tests-js/pages/RemoteAccessTunnelPage.mjs` (new) | POM |
| `crates/lib_bodhiserver/tests-js/fixtures/tunnelFixtures.mjs` (new) | server-config factories |
| `crates/lib_bodhiserver/tests-js/fixtures/bin/fake-cloudflared.mjs` (new) | stub binary |
| `crates/lib_bodhiserver/tests-js/test-helpers.mjs` | export `FAKE_CLOUDFLARED_BIN` |
| `.github/workflows/playwright.yml`, `tests-js/.env.test.example` | new `INTEG_TEST_CLOUDFLARE_*` secrets/vars — only if the real-Cloudflare step (§5, step 8) is built |
| `ts-client` (upstream, backend stream) | `AppInfo.origins`, `TunnelStatus` schema — everything above depends on these existing first |

## Sources

All findings are from reading this repository's own source in this session (no external sources needed for this doc's scope); the file:line citations throughout are the primary evidence. Prior research consulted for context (not re-cited per claim): `docs/research/tunnel/01-bodhi-app-codebase-map.md`, `docs/research/tunnel/20-codebase-child-process-and-app-lifecycle.md`, `docs/research/tunnel/bodhiapp-cloudflare-tunnel-feasibility.md`, `docs/research/tunnel/00-consolidated-research.md`.

---

## Follow-up: Spike-verify GitHub Actions runner egress for Cloudflare's tunnel (QUIC/HTTP2) protocol

**Date:** 2026-09-15. Extends §5 "Is a real named-tunnel E2E realistic?" above.

### Scope note: the literal live spike was not executed in this session

The task asked for a live, disposable Cloudflare zone + API token, a throwaway GitHub Actions workflow, and a real tunnel-create/route-dns/run/curl/teardown cycle. **That was not run.** This environment has no Cloudflare account, API token, or zone available (checked: no `CLOUDFLARE_*`/`CF_*` env vars, no macOS Keychain entry for `cloudflare`, 1Password CLI not connected to the desktop app). Provisioning a "disposable zone" for real means either registering a new throwaway domain or delegating a spare subdomain on an existing Cloudflare account — both are billing/account actions outside what this session can do autonomously without the product owner's explicit go-ahead and credentials. `gh` is authenticated (`anagri`, `repo` scope) so the GitHub-Actions-workflow half of the spike is executable on request, but it is gated on the Cloudflare half.

**What this section does instead:** answers the same question — will registration complete on a GitHub-hosted runner — from the strongest available secondary evidence (official docs + real, currently-maintained GitHub Actions that already do this), gives a confidence verdict per runner OS, and hands over a ready-to-run workflow + exact steps so the literal spike takes an engineer with Cloudflare credentials well under 10 minutes once this doc's gap is handed off.

### GitHub-hosted runner network egress — no UDP-specific block found

- GitHub-hosted runners have **unrestricted general outbound network access by default** — confirmed by GitHub's own roadmap issue: *"GitHub hosted runners today allow unrestricted outbound network access. Any workflow can reach any host on the internet, regardless of GITHUB_TOKEN permissions, secret scoping, OIDC, or SHA pinning."* (github/roadmap#821). Egress *filtering* is an opt-in feature GitHub is still building (`github-early-access/actions-native-egress-firewall`) — irrelevant unless BodhiApp's own workflow opts in.
- Linux/Windows GitHub-hosted runners are **Azure `Standard_DS2_v2` VMs**; the one concretely documented protocol-level restriction found is that **Azure blocks ICMP by default** on these VMs (so `ping`/traceroute-style diagnostics fail in workflows) — this is unrelated to UDP application traffic on port 7844 and doesn't extend to QUIC. No source found documenting a UDP-specific egress block on GitHub-hosted runners.
- **Not the same population as self-hosted runners**: the "restrictive iptables block outbound" reports found in this pass are specifically about *self-hosted* runner base images/VNETs, not GitHub-hosted ones — do not conflate the two when reading community threads about "runner firewall" issues.

### What cloudflared actually needs on the wire (sharpens doc 10 §4.3's `--protocol` table)

Per Cloudflare's own firewall-configuration doc (`developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/configure-tunnels/tunnel-with-firewall/`, fetched 2026-09-15): cloudflared's edge registration needs outbound **port 7844**, **UDP for QUIC** and **TCP for the HTTP/2 fallback**, to the `region1.v2.argotunnel.com`/`region2.v2.argotunnel.com` anycast ranges (e.g. `198.41.192.0/24`-ish and `198.41.200.0/24`-ish blocks, IPv6 `2606:4700:a0::/48` and `2606:4700:a8::/48`), with SNI-enforcing firewalls additionally needing `cftunnel.com`/`h2.cftunnel.com`(TCP)/`quic.cftunnel.com`(UDP) allowed. **This confirms the failure mode the gap worried about (blocked UDP egress) has a designed-in mitigation, already noted in doc 10 §4.3**: `--protocol auto` (the default) starts with QUIC and **falls back to HTTP/2 over the same port 7844 via TCP** if QUIC/UDP doesn't connect — so even a runner with UDP 7844 filtered (not observed, but unverified in the negative) would very likely still register successfully over TCP, just via a different protocol and with a fallback log line. This means "does QUIC work" and "does registration work at all" are different, and less brittle, questions than doc 24's original framing implied — the E2E assertion (a round-trip HTTPS request through the tunnel) doesn't actually depend on which of the two transports won.

### Real prior art: this exact thing already runs on GitHub-hosted runners today

| Project | What it runs, where | Tunnel type | Evidence found |
|---|---|---|---|
| [`valeriangalliat/action-sshd-cloudflared`](https://github.com/valeriangalliat/action-sshd-cloudflared) | `./cloudflared tunnel --no-autoupdate --url tcp://localhost:2222` directly on the runner, backgrounded, then greps its log for the assigned `https://*.trycloudflare.com` hostname | Quick/`trycloudflare` (not named — but the **same QUIC/HTTP2 edge-registration handshake** doc 24 is worried about; quick vs. named differ only in how the hostname is assigned after registration, not in the transport-layer connection) | Source read directly (`setup-ssh` script, `entrypoint.sh`-equivalent). README states `ubuntu-latest` and `macos-latest` "also supported." Repo has 43 stars, 0 open issues, pushed 2025-05, metadata updated into 2026-03 — i.e. actively used and not reporting connectivity breakage. Comments in the script reflect real runner-specific debugging (e.g. a macOS-runner `~/.bash_profile` quirk), which is the kind of detail that only shows up after actually running it on real runners. |
| [`AnimMouse/setup-cloudflared`](https://github.com/AnimMouse/setup-cloudflared) (`/tunnel` sub-action) | `cloudflared --pidfile ... --logfile ... tunnel run` (i.e. a **named, credentialed** tunnel — credentials + config + tunnel ID supplied as base64 secrets) on the runner | **Named** — structurally identical to what doc 24 step 8 proposes | Source read directly (`tunnel/action.yaml`, `scripts/autostart/{Unix-like.sh,Windows.ps1}`, `scripts/sign-in/Unix-like.sh`). Its own CI (`.github/workflows/test-tunnel.yaml`) runs a `{ubuntu-latest, windows-latest, macos-latest} × {production, trycloudflare}` matrix against a real domain (`44444444.xyz`) with a Python `http.server` behind it — i.e. **this maintainer already built the identical spike doc 24 describes, across all three OSes, including Windows.** **Could not confirm a green run in this session**: the GitHub Actions API returned `total_count: 0` runs for both of the repo's workflows, and the live test URL (`setup-cloudflared.44444444.xyz`) returned a Cloudflare 530 (no active tunnel) at fetch time — expected, since the tunnel is only up while the workflow runs, but it means I have the *design*, not an observed *result*. **Treat as UNVERIFIED-but-strong-signal**: a maintainer publishing and marketing a named-tunnel GitHub Action with this exact test matrix is far stronger evidence than nothing, but is not the same as a logged successful run. |

### Verdict, by runner OS

| Runner | Confidence registration succeeds | Basis |
|---|---|---|
| `ubuntu-latest` | **High** | Unrestricted egress (GitHub's own statement) + real, adopted quick-tunnel action running unmodified on it + a named-tunnel action explicitly designed and shipped for it. |
| `macos-latest` | **High** | Same quick-tunnel action explicitly supports and is used on `macos-latest`; same named-tunnel action's CI matrix targets it too (design-level, run not confirmed). |
| `windows-latest` | **Medium** | Only evidence is `AnimMouse/setup-cloudflared`'s CI *design* (matrix includes `windows-latest`, with its own `Windows.ps1` sign-in/autostart scripts) — no independently-confirmed successful run found, and `action-sshd-cloudflared` explicitly does **not** support Windows (for an unrelated password-auth reason, not network). No evidence of a Windows-specific UDP/QUIC block either. Treat Windows as the one to actually spike first if budget only allows one extra OS beyond `ubuntu-latest`. |

**Net conclusion for doc 24 §5's open question:** the original UNVERIFIED framing ("should work in principle... confirm empirically") was appropriately cautious but, on the evidence gathered here, the risk is low for `ubuntu-latest` (the only OS doc 24's E2E design actually needs — Playwright specs run on Linux CI per the existing `playwright.yml`). **This does not need to block committing to the opt-in `@cloudflare-live` tier design in doc 24 §5.** It does still need one literal empirical run before the tier ships (registration latency, exact log line to grep, and whether the CI IP ranges hit any Cloudflare rate-limit are all things no amount of secondary evidence settles) — treat that as a fast, low-risk spike to schedule once Cloudflare credentials exist, not a blocking unknown for the architecture decision.

### Ready-to-run spike (for whoever has Cloudflare credentials)

**Cloudflare token** — per doc 11 §3, create via the prefilled-link flow with `Cloudflare Tunnel · Edit` (Account) + `DNS · Edit` (Zone) + `Zone · Read` (Zone) permission groups, scoped to a disposable test zone (never `getbodhi.app` production).

**Throwaway workflow** (`.github/workflows/_spike-tunnel.yml`, delete after the spike):
```yaml
name: spike-cloudflare-tunnel
on: workflow_dispatch
jobs:
  spike:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    env:
      CLOUDFLARE_API_TOKEN: ${{ secrets.SPIKE_CF_API_TOKEN }}
      CLOUDFLARE_ACCOUNT_ID: ${{ secrets.SPIKE_CF_ACCOUNT_ID }}
    steps:
      - name: Download cloudflared
        shell: bash
        run: |
          case "${{ matrix.os }}" in
            ubuntu-latest) f=cloudflared-linux-amd64;;
            macos-latest) f=cloudflared-darwin-amd64.tgz;;
            windows-latest) f=cloudflared-windows-amd64.exe;;
          esac
          curl -fsSL -o cloudflared "https://github.com/cloudflare/cloudflared/releases/latest/download/$f" || \
          (curl -fsSL -o cf.tgz "https://github.com/cloudflare/cloudflared/releases/latest/download/$f" && tar xf cf.tgz)
          chmod +x cloudflared* 2>/dev/null || true
      - name: Create tunnel + route DNS
        shell: bash
        run: |
          date +%s > /tmp/t0
          ./cloudflared tunnel create spike-$RANDOM --output json > tunnel.json
          id=$(jq -r .id tunnel.json)
          ./cloudflared tunnel route dns "$id" spike-${{ matrix.os }}.<your-test-zone>
          echo "TUNNEL_ID=$id" >> "$GITHUB_ENV"
      - name: Run tunnel + local server, curl through it
        shell: bash
        run: |
          python3 -m http.server 8080 &
          ./cloudflared tunnel --url http://localhost:8080 --no-autoupdate run "$TUNNEL_ID" \
            --loglevel info --output json > cloudflared.log 2>&1 &
          for i in $(seq 1 30); do
            grep -q '"message":"Registered tunnel connection"' cloudflared.log && break
            sleep 2
          done
          echo "seconds to register: $(( $(date +%s) - $(cat /tmp/t0) ))"
          curl -fsS --retry 5 --retry-delay 3 https://spike-${{ matrix.os }}.<your-test-zone> || echo "ROUND-TRIP FAILED"
      - name: Teardown
        if: always()
        shell: bash
        run: |
          pkill cloudflared || true
          ./cloudflared tunnel delete -f "$TUNNEL_ID" || echo "manual cleanup needed: tunnel $TUNNEL_ID"
```
Record, per OS: pass/fail of the final `curl`, the "seconds to register" line, and the exact protocol used (grep `cloudflared.log` for `"protocol":"quic"` vs `"protocol":"http2"` per doc 10 §4.5's structured log fields). Delete the workflow file and revoke `SPIKE_CF_API_TOKEN` after.

### Sources

- https://github.com/github/roadmap/issues/821 — GitHub's own statement that hosted runners have unrestricted outbound access today
- https://github.com/orgs/community/discussions/146489 — outbound network control discussion for GitHub-hosted runners
- https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/configure-tunnels/tunnel-with-firewall/ — port 7844 TCP/UDP requirement, anycast IP ranges, SNI-firewall hostnames (fetched 2026-09-15)
- https://github.com/valeriangalliat/action-sshd-cloudflared — `setup-ssh` script read directly (`gh api repos/valeriangalliat/action-sshd-cloudflared/contents/setup-ssh`), README, repo metadata (43 stars, pushed 2025-05-05, updated 2026-03-24)
- https://github.com/AnimMouse/setup-cloudflared — `tunnel/action.yaml`, `tunnel/scripts/autostart/{Unix-like.sh,Windows.ps1}`, `tunnel/scripts/sign-in/Unix-like.sh`, `.github/workflows/{test.yaml,test-tunnel.yaml}` all read directly via `gh api`; live test page `https://setup-cloudflared.44444444.xyz` checked (530, no active tunnel at fetch time — expected, not evidence either way); `gh api repos/AnimMouse/setup-cloudflared/actions/runs` returned 0 runs — **could not confirm an actual successful execution**, flagged UNVERIFIED
- https://www.kenmuse.com/blog/restricting-ip-access-on-github-hosted-runners/ — GitHub-hosted runners run on Azure `Standard_DS2_v2` VMs; Azure blocks ICMP by default (unrelated to UDP/7844, cited to rule it out)
- `docs/research/tunnel/10-cloudflared-cli-named-tunnel-lifecycle.md` §4.3, §4.5 (this repo) — `--protocol auto` QUIC-then-HTTP2-fallback behavior and structured log field names, cited rather than re-derived
- `docs/research/tunnel/11-cloudflare-oauth-and-api-token-options.md` §3 (this repo) — permission groups used for the spike token above
