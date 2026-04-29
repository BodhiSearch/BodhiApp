---
title: 'Error Codes'
description: 'Lookup index for the most common error codes returned across Bodhi-native, OpenAI, Anthropic, and Gemini envelopes'
order: 3
---

# Error Codes

When Bodhi rejects a request, the JSON body carries a stable `code` (or `type`) string you can match on. This page is a lookup index for the most-encountered codes. For the _shape_ of each error envelope and how to tell them apart, read [Error Format](/docs/api-compatibility/error-format) first.

## Four envelopes, one underlying error

Bodhi produces a single internal error and serializes it into one of four wire formats based on which compat layer received the request:

| Envelope             | Endpoints                                                                         | Carries `code` field                                                                            |
| -------------------- | --------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| **Bodhi-native**     | `/bodhi/v1/*` (UI APIs, MCP CRUD, settings, tokens, app access)                   | Yes — `error.code`                                                                              |
| **OpenAI-style**     | `/v1/chat/completions`, `/v1/embeddings`, `/v1/responses`, `/v1/models`, `/api/*` | Yes — `error.code`                                                                              |
| **Anthropic-native** | `/anthropic/v1/messages`, `/v1/messages`, `/anthropic/v1/models`                  | No — only `error.type`. 5xx messages are sanitized to `"internal server error"`.                |
| **Gemini-native**    | `/v1beta/*`                                                                       | No — only `error.code` (HTTP int) and `error.status` (gRPC string). 5xx messages are sanitized. |

The Bodhi-native and OpenAI envelopes preserve the full code string and message. The Anthropic and Gemini envelopes intentionally drop the granular code (their SDKs don't model it) and replace 5xx messages so internal details never leak. If you need the exact reason a 5xx happened, hit a Bodhi-native or OpenAI-style endpoint or check the server logs.

## Code naming convention

Codes follow `<domain>-<reason>`, where the domain is the originating service in snake_case and the reason is the variant name in snake_case. Examples:

- `data_service_error-alias_not_found`
- `mcp_error-mcp_not_found`
- `auth_context_error-anonymous_not_allowed`

Codes are stable across patch releases — match on them in your client. Messages are not stable; do not parse them.

## Top error codes by domain

The tables below list the codes you are most likely to encounter as a user or app developer. They are not exhaustive — new codes can ship in any release. If you see one not listed, the `code` string is still a reliable programmatic identifier.

### Auth and tokens

| Code                                       | HTTP | When you see it                                              | Likely cause                                                                        |
| ------------------------------------------ | ---- | ------------------------------------------------------------ | ----------------------------------------------------------------------------------- |
| `auth_context_error-anonymous_not_allowed` | 403  | An endpoint requires authentication; you sent no credentials | Missing `Authorization: Bearer` header or session cookie                            |
| `auth_context_error-missing_token`         | 401  | Endpoint requires a token specifically                       | You're authenticated by session but the endpoint is API-only                        |
| `token_error-invalid_token`                | 401  | Token cannot be parsed or hash check fails                   | Truncated, wrong format (`bodhiapp_<random>.<client_id>`), or revoked               |
| `token_error-token_expired`                | 401  | Token is no longer valid                                     | Refresh failed or token was deactivated                                             |
| `token_scope_error-invalid_token_scope`    | 400  | Scope string is not recognized                               | Use `scope_token_user` or `scope_token_power_user`                                  |
| `auth_service_error-token_exchange_error`  | 400  | OAuth code → token exchange failed                           | Bad redirect URI, expired code, clock skew                                          |
| `role_error-invalid_role`                  | 400  | Role string is not assignable                                | Use `resource_user`, `resource_power_user`, `resource_manager`, or `resource_admin` |

### Models and aliases

| Code                                 | HTTP | When you see it                             | Likely cause                                                       |
| ------------------------------------ | ---- | ------------------------------------------- | ------------------------------------------------------------------ |
| `data_service_error-alias_not_found` | 404  | Requested model alias doesn't exist         | Typo, alias deleted, or you used the file ID instead of alias name |
| `data_service_error-alias_exists`    | 400  | Cannot create — alias name already taken    | Use a different name or edit the existing alias                    |
| `data_service_error-unsupported`     | 400  | Operation not supported for this alias type | E.g. trying to edit a system-managed alias                         |
| `download_service_error-not_found`   | 404  | GGUF file not found on HuggingFace          | Wrong repo or filename                                             |
| `download_service_error-auth`        | 401  | HuggingFace rejected the download           | Set `HF_TOKEN` for gated repos                                     |
| `api_model_service_error-validation` | 400  | API model config rejected                   | Missing key, invalid base URL, unsupported provider                |
| `api_model_service_error-not_found`  | 404  | API model alias doesn't exist               | —                                                                  |
| `api_model_service_error-auth`       | 401  | Upstream provider rejected the API key      | Key is wrong, revoked, or for the wrong account                    |
| `model_validation_error-*`           | 400  | GGUF file failed validation                 | Wrong format, partial download, unsupported architecture           |

### MCPs

| Code                                                    | HTTP | When you see it                                  | Likely cause                                                                                  |
| ------------------------------------------------------- | ---- | ------------------------------------------------ | --------------------------------------------------------------------------------------------- |
| `mcp_error-mcp_not_found`                               | 404  | MCP instance ID doesn't exist                    | Deleted, or you're hitting the wrong tenant                                                   |
| `mcp_error-mcp_server_not_found`                        | 404  | Pre-registered MCP server slug not found         | Server removed from the catalog by an admin                                                   |
| `mcp_error-mcp_disabled`                                | 400  | Instance is disabled                             | Re-enable in `/ui/mcps/`                                                                      |
| `mcp_error-slug_exists`                                 | 409  | Slug already in use                              | Pick a different slug                                                                         |
| `mcp_error-name_required`, `-url_required`              | 400  | Required field missing on MCP instance           | Fill in the field                                                                             |
| `mcp_error-url_invalid`, `-url_too_long`                | 400  | URL failed validation                            | Must be `http`/`https`, ≤ 2048 chars                                                          |
| `mcp_error-oauth_token_not_found`                       | 404  | No OAuth token stored for this MCP               | Re-run the connect flow                                                                       |
| `mcp_error-oauth_token_expired`                         | 400  | Stored OAuth token expired and could not refresh | Disconnect and re-authorize                                                                   |
| `mcp_error-oauth_refresh_failed`                        | 500  | Refresh token round-trip failed                  | Upstream MCP rejected the refresh — re-authorize                                              |
| `mcp_error-oauth_discovery_failed`                      | 500  | RFC 8414 discovery failed during DCR             | Server doesn't expose `.well-known/oauth-authorization-server`; switch to OAuth preregistered |
| `mcp_error-forbidden`                                   | 403  | Caller lacks consent for this MCP                | Resource not granted in the access request                                                    |
| `mcp_server_error-name_required`, `-url_required`, etc. | 400  | Pre-registered MCP server validation failed      | Fill in the field                                                                             |

### App access

| Code                                          | HTTP | When you see it                                     | Likely cause                                 |
| --------------------------------------------- | ---- | --------------------------------------------------- | -------------------------------------------- |
| `access_request_error-not_found`              | 404  | App access request ID doesn't exist                 | Already approved/rejected or expired         |
| `access_request_error-expired`                | 409  | Request older than 10 minutes                       | App must submit a new request                |
| `access_request_error-already_processed`      | 409  | Request already approved/rejected                   | No-op                                        |
| `access_request_error-missing_redirect_uri`   | 400  | Required redirect URI missing                       | Include `redirect_uri` in the create request |
| `access_request_error-version_mismatch`       | 400  | Submitted version doesn't match latest              | Stale draft — refresh and resubmit           |
| `access_request_error-kc_registration_failed` | 500  | Upstream OAuth provider could not create the client | Retry; if persistent, contact support        |

### Settings

| Code                                         | HTTP | When you see it                            | Likely cause                                                                                                            |
| -------------------------------------------- | ---- | ------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------- |
| `setting_service_error-invalid_setting_key`  | 400  | Tried to PUT/DELETE a non-editable setting | Only `BODHI_EXEC_VARIANT` and `BODHI_KEEP_ALIVE_SECS` are editable. See [Settings precedence](/docs/reference/settings) |
| `settings_metadata_error-null_value`         | 400  | Setting value cannot be null               | —                                                                                                                       |
| `settings_metadata_error-invalid_value`      | 400  | Value not in allowed set / out of range    | Check allowed options in the UI                                                                                         |
| `settings_metadata_error-invalid_value_type` | 400  | Wrong JSON type for the setting            | Number expected, got string, etc.                                                                                       |

### Inference

| Code                | HTTP | When you see it                              | Likely cause                                                                                                          |
| ------------------- | ---- | -------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `inference_error-*` | 500  | llama.cpp process failed or returned non-2xx | Variant mismatch (CUDA on a non-GPU box), GGUF too large for VRAM, model crashed mid-stream. Check `$BODHI_HOME/logs` |

### Generic

| Code                                     | HTTP | When you see it                | Likely cause                                                              |
| ---------------------------------------- | ---- | ------------------------------ | ------------------------------------------------------------------------- |
| `obj_validation_error-validation_failed` | 422  | Body validation failed         | Missing required field, wrong type — check `error.params` for which field |
| `serde_json_error-*`                     | 400  | Request body is not valid JSON | Trailing comma, unquoted key                                              |
| `db_error-*`                             | 500  | DB query failed                | Disk full, locked SQLite, PostgreSQL unreachable                          |
| `reqwest_error-*`                        | 500  | Outbound HTTP failed           | Upstream provider down, DNS, certificate                                  |
| `keyring_error-*`                        | 500  | OS keychain access failed      | Desktop only — see [Troubleshooting](/docs/support/troubleshooting)       |

## Mapping to envelopes

For Bodhi-native and OpenAI envelopes, the `code` shown above appears verbatim in `error.code`. For Anthropic, the closest equivalent is `error.type` (one of `invalid_request_error`, `authentication_error`, `permission_error`, `not_found_error`, `api_error`, `overloaded_error`). For Gemini, `error.status` is the gRPC string (`INVALID_ARGUMENT`, `UNAUTHENTICATED`, `PERMISSION_DENIED`, `NOT_FOUND`, `INTERNAL`, `UNAVAILABLE`).

| HTTP | Bodhi/OpenAI `type`          | Anthropic `error.type`         | Gemini `error.status`     |
| ---- | ---------------------------- | ------------------------------ | ------------------------- |
| 400  | `invalid_request_error`      | `invalid_request_error`        | `INVALID_ARGUMENT`        |
| 401  | `authentication_error`       | `authentication_error`         | `UNAUTHENTICATED`         |
| 403  | `forbidden_error`            | `permission_error`             | `PERMISSION_DENIED`       |
| 404  | `not_found_error`            | `not_found_error`              | `NOT_FOUND`               |
| 409  | `conflict_error`             | `invalid_request_error`        | `INVALID_ARGUMENT`        |
| 422  | `unprocessable_entity_error` | `invalid_request_error`        | `INVALID_ARGUMENT`        |
| 500  | `internal_server_error`      | `api_error` (sanitized)        | `INTERNAL` (sanitized)    |
| 503  | `service_unavailable`        | `overloaded_error` (sanitized) | `UNAVAILABLE` (sanitized) |

## Tips for handling errors programmatically

- Match on `code` when available (Bodhi-native and OpenAI envelopes). It is the only stable identifier across releases.
- For Anthropic and Gemini, branch on the HTTP status and, secondarily, on `error.type` / `error.status`.
- Never parse human messages — they are translated/expanded over time.
- For 5xx, retry with backoff. Bodhi never includes 5xx detail in the Anthropic and Gemini envelopes; check Bodhi-native logs if you need to know why.

## Related

- [Error Format](/docs/api-compatibility/error-format) — exact JSON shape per envelope.
- [Roles and scopes](/docs/reference/roles-and-scopes) — what causes 401 vs 403.
- [Troubleshooting](/docs/support/troubleshooting) — symptom → cause → fix flow.
- Swagger UI at `/swagger-ui` on your running instance — every endpoint's exact error response schemas.
