# Slice 1 — Cloudflare named tunnel handover

Status: implementation and focused automated checks complete; live demo blocked on owner/machine prerequisites.

## What now works

- A user can open Settings → Tunnels, enter a public hostname, enable/disable a named Cloudflare tunnel, see connection status and public URL, and copy the exact Keycloak callback URI. The connector is not coupled to the native/Tauri package; its environment setting defaults from `is_native()` and can explicitly enable CLI mode.
- The app creates or reuses a hostname-derived tunnel, routes DNS, sends traffic to loopback, retains the hostname for a later re-enable, and leaves the connector disabled after restart.
- A tunnel-host login request now generates `https://<tunnel-host>/ui/auth/callback`; local/LAN behavior remains request-host based and `BODHI_PUBLIC_HOST` is never changed.

## Demo results

1. Not attempted — owner/machine prerequisite missing (`cloudflared` is not on PATH).
2. Not attempted — same blocker; a hostname from the owner's Cloudflare zone is required.
3. Not attempted — requires a real Cloudflare zone and connector.
4. Passed by code inspection and UI build; not exercised in the desktop app.
5. Not attempted — owner-only Keycloak admin action.
6. Not attempted — no real public URL, owner Keycloak action, or second network/device. This remains the decisive unproven risk.
7. Not attempted live. Unix builds supervise the connector through a liveness pipe and terminate it on normal shutdown or parent death; this needs real-process verification.
8. Not attempted live. The hostname persists in app settings; the deterministic tunnel name plus Cloudflare lookup should reuse it.

## Design and verification

- `crates/services/src/tunnels/` holds the small `TunnelService` seam, CLI lifecycle, `/ready` polling, persistence, and Unix child supervision. Credential files live under `$BODHI_HOME/tunnels`; `cert.pem` stays in `~/.cloudflared` (or explicit `TUNNEL_ORIGIN_CERT`).
- `crates/routes_app/src/tunnels/` exposes admin-session `GET/PUT/DELETE /bodhi/v1/tunnel`; OpenAPI and the generated TS client are refreshed.
- `crates/bodhi/src/routes/tunnels/` and `hooks/tunnels/` provide the Settings page and poll every second while connecting, otherwise every ten seconds.
- `routes_auth.rs` accepts the forwarded HTTPS origin only when `Host` equals the persisted tunnel hostname. This preserves multiple simultaneous origins and avoids setting canonical-host fields.
- The run invocation uses a credentials file, loopback origin, `--no-autoupdate`, loopback metrics, `--loglevel info`, and `--protocol auto`; no Cloudflare secret is passed as an argument. No Keycloak API call was added.
- Focused checks passed: `cargo test -p services tunnels`, the tunneled OAuth redirect test, `cargo test -p server_app --lib`, `cargo fmt --check`, UI typecheck, targeted UI ESLint, and UI production build. Chose the cheap redirect test over a fake-cloudflared harness; the live lifecycle remains untested.

## Incomplete / next steps

- Install `cloudflared` for the desktop account and run `cloudflared tunnel login` so `~/.cloudflared/cert.pem` exists; this slice intentionally has no installer or login flow.
- Start the CLI app with `BODHI_TUNNEL=true` (the native default already follows `is_native()`), enable the chosen hostname, and wait for Connected.
- Owner adds `https://<tunnel-host>/ui/auth/callback` to the Keycloak client's valid redirect URIs, then tests login and a streaming chat from a second network.
- Verify disable and abrupt-app-exit leave no connector, then restart and re-enable to prove tunnel/DNS reuse. Run the full required suites, refresh the code graph, and commit only after that demo passes.

## Working tree

- Tunnel implementation, generated specs/client, UI route, and package guidance are committed locally. Existing unrelated user changes and untracked plan documents were preserved.
