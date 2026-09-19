# services/src/tunnels — CLAUDE.md

Cloudflare named-tunnel support: provisioning a tunnel, supervising the `cloudflared` connector, and
advertising public reachability. See [PACKAGE.md](PACKAGE.md) for the file-by-file surface.

## The runtime seam

`DefaultTunnelService` never touches the outside world directly. Three traits in `runtime.rs` stand
between it and the OS:

| Trait | Covers |
|---|---|
| `CloudflaredCli` | one-shot `cloudflared` invocations |
| `ConnectorProcess` / `ConnectorHandle` | the supervised long-lived connector child |
| `TunnelIo` | filesystem, `PATH`, and HTTP (Cloudflare API and the loopback `/ready` probe) |

**The seam returns raw output and interprets nothing.** Exit status, bytes, and HTTP status plus body
come back untouched; every parse — the `--output json` payloads, the origin-certificate PEM, the
metrics-address log line, Cloudflare's envelopes — stays in `service.rs` where a unit test can reach
it. Adding a "convenience" that parses inside the seam puts that logic back out of reach.

`runtime_impl.rs` holds the only production implementations and is the one place platform `cfg`
branches are allowed. `service.rs` must stay platform-free.

## Why the connector is wrapped in a shell on Unix

The connector must die when BodhiApp does, including on `SIGKILL`, where no `Drop` or shutdown
handler of ours runs. `SystemConnectorProcess` therefore spawns `/bin/sh` holding one end of a
`UnixStream` pair; closing our end is what stops `cloudflared`. Process-group crates do not solve
this — they cover graceful shutdown only. Consequence to remember: on Unix the supervised PID is the
shell, not `cloudflared`.

`std::process` with reader threads is deliberate, matching `llama_server_proc`, whose async version
orphaned children. Only the short, awaited one-shot commands use `tokio::process`.

## Testing

`test_utils/tunnels.rs` provides `FakeCloudflared`, one in-memory object implementing all three
traits. Tests spawn no processes, write no files, open no sockets and do not sleep; the supervisor is
observed through the runtime's broadcast tap, and its cadence is injected with `with_intervals`.
