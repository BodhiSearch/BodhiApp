# routes_app/resources

Static OpenAPI specs that BodhiApp serves via swagger-ui for proxied AI providers.
These files are checked-in copies — they are NOT generated from BodhiApp's own
`utoipa` derives.

## openapi-anthropic.json

**Purpose**: Describes the Anthropic Messages API endpoints that BodhiApp's
`/anthropic/v1/*` routes proxy through to upstream Anthropic providers. Served
under `/api-docs/openapi-anthropic.json` and visible in the swagger-ui dropdown
at `/swagger-ui` alongside `openapi.json` (BodhiApp management) and
`openapi-oai.json` (OpenAI-compatible endpoints).

**Source**: Filtered from the upstream Anthropic OpenAPI spec (published by the
[anthropic-sdk-typescript](https://github.com/anthropics/anthropic-sdk-typescript)
repo) to only the endpoints BodhiApp proxies (`/v1/messages`, `/v1/models`,
`/v1/models/{model_id}`) plus their referenced component schemas. The filtering
uses `openapi-format` with `inverseOperationIds` + `unusedComponents` (see
`ts-client/anthropic-openapi-filter.yaml`).

**Refresh procedure**:

```bash
make openapi.anthropic
```

This command:
1. Fetches the latest `.stats.yml` from the Anthropic SDK repo to discover
   the current OpenAPI spec URL and hash.
2. Compares the hash with `ts-client/.anthropic-openapi-meta.json` — exits
   early if unchanged.
3. Downloads the full spec and filters it via `openapi-format`.
4. Writes `ts-client/openapi-anthropic.json` (the canonical filtered spec).
5. Copies it here to `crates/routes_app/resources/openapi-anthropic.json`.
6. Regenerates TypeScript types in `ts-client/src/types-anthropic/`.

After running, verify with:

```bash
cargo check -p routes_app
```

`routes_app` parses this file at compile time via `include_str!` and at boot
time via `serde_json::from_str`. A malformed file will fail loudly.

**Path rewriting**: The upstream file uses Anthropic's native paths
(`/v1/messages` etc.). BodhiApp proxies them under `/anthropic/v1/*`. Rather
than edit each path key, `routes.rs` injects a `servers: [{"url": "/anthropic"}]`
override at boot, so swagger-ui prepends the proxy prefix automatically.

**Do NOT hand-edit this file** — changes will be lost on the next refresh.
Update the filter config in `ts-client/anthropic-openapi-filter.yaml` instead.
