# Settings Analysis for Multi-Tenancy

> **Purpose**: Complete categorization of every BODHI_* setting.
> Determines which are global infrastructure vs per-tenant vs dead code in multi-tenant mode.
>
> **Created**: 2026-03-03
> **Key finding**: ALL current settings are global/infrastructure. No per-tenant settings exist.

---

## Summary

- **Total settings**: ~30 distinct keys
- **Infrastructure (always global)**: 20+
- **LLM execution (dead in multi)**: 7
- **Editable via API**: 2 (BODHI_EXEC_VARIANT, BODHI_KEEP_ALIVE_SECS)
- **Decision**: Settings table stays global permanently. Separate `tenant_settings` table if needed.

---

## 1. System/Infrastructure Settings (Always Global)

These are immutable system properties set at startup.

| Key | Default | Source | Editable | Notes |
|-----|---------|--------|----------|-------|
| `BODHI_HOME` | `~/.cache/bodhi` | System (env-derived) | **NEVER** | Cannot be changed via API |
| `BODHI_LOGS` | `{BODHI_HOME}/logs` | Derived | No | Read-only |
| `BODHI_ENV_TYPE` | `development` | System (build) | No | Enum: Production, Development |
| `BODHI_APP_TYPE` | `native` or `container` | System (build) | No | Enum: Native, Container |
| `BODHI_VERSION` | (from git) | Build | No | System property |
| `BODHI_COMMIT_SHA` | (from git) | Build | No | System property |
| `BODHI_AUTH_URL` | (Keycloak URL) | Env/config | No | System property |
| `BODHI_AUTH_REALM` | `bodhi` | Env/config | No | System property |
| `BODHI_ENCRYPTION_KEY` | (generated) | Environment only | No | System secret |
| `BODHI_ON_RUNPOD` | `false` | Environment | No | Cloud deployment flag |
| `RUNPOD_POD_ID` | (from RunPod) | Environment | No | Cloud deployment ID |

## 2. HTTP Server Configuration (Global Infrastructure)

| Key | Default | Source | Editable | Notes |
|-----|---------|--------|----------|-------|
| `BODHI_SCHEME` | `http` | Hardcoded | No | |
| `BODHI_HOST` | `0.0.0.0` | Hardcoded | No | |
| `BODHI_PORT` | `1135` | Hardcoded | No | |
| `BODHI_PUBLIC_SCHEME` | (mirrors BODHI_SCHEME) | Derived | No | |
| `BODHI_PUBLIC_HOST` | (mirrors BODHI_HOST) | Derived/RunPod | No | |
| `BODHI_PUBLIC_PORT` | (mirrors BODHI_PORT) | Derived/RunPod | No | |
| `BODHI_CANONICAL_REDIRECT` | `true` | Hardcoded | No | |
| `BODHI_DEPLOYMENT` | `standalone` | Hardcoded | No | Values: `standalone`, `multi` |

## 3. LLM Execution Settings (Dead Code in Multi-Tenant)

These control llama.cpp process startup. All dead code when `BODHI_DEPLOYMENT=multi`.

| Key | Default | Source | Editable | Notes |
|-----|---------|--------|----------|-------|
| `BODHI_EXEC_LOOKUP_PATH` | (build-dependent) | Environment | No | Binary search path |
| `BODHI_EXEC_VARIANT` | `default` | Default | **YES** | GPU variant switching (CPU/CUDA) |
| `BODHI_EXEC_TARGET` | (build target triplet) | Hardcoded | No | |
| `BODHI_EXEC_NAME` | `llama-server` | Hardcoded | No | |
| `BODHI_EXEC_VARIANTS` | CSV: `default,cuda,...` | Hardcoded | No | Available variants |
| `BODHI_LLAMACPP_ARGS` | `--jinja --no-webui` | Hardcoded | No | Common args |
| `BODHI_LLAMACPP_ARGS_{VARIANT}` | (none) | Optional env/db | No | Per-variant overrides |

## 4. Database URLs (Global Infrastructure)

| Key | Default | Source | Editable | Notes |
|-----|---------|--------|----------|-------|
| `BODHI_SESSION_DB_URL` | `sqlite:{BODHI_HOME}/session.sqlite` | Default | No | Session store |
| `BODHI_APP_DB_URL` | `sqlite:{BODHI_HOME}/bodhi.sqlite` | Default | No | App data store |

## 5. Model/Data Management (Global)

| Key | Default | Source | Notes |
|-----|---------|--------|-------|
| `HF_HOME` | `~/.cache/huggingface` | System home | Model cache location |

## 6. Logging (Global Infrastructure)

| Key | Default | Editable | Notes |
|-----|---------|----------|-------|
| `BODHI_LOG_LEVEL` | `warn` | No | Enum: off/error/warn/info/debug/trace |
| `BODHI_LOG_STDOUT` | `false` | No | |

## 7. Server Maintenance (Global)

| Key | Default | Editable | Notes |
|-----|---------|----------|-------|
| `BODHI_KEEP_ALIVE_SECS` | `300` | **YES** | Inactivity timeout (300-86400) |

## 8. Development Only (Global)

| Key | Default | Notes |
|-----|---------|-------|
| `BODHI_DEV_PROXY_UI` | `http://localhost:3000` | Dev-only proxy |

## 9. New Settings for Multi-Tenancy

| Key | Default | Mode | Notes |
|-----|---------|------|-------|
| `BODHI_DEPLOYMENT` | `standalone` | All | Already exists as constant, needs operational implementation |
| `BODHI_MULTITENANT_CLIENT_ID` | (none) | Multi only | Platform client for initial auth. Error if standalone. Error if not set in multi. |

---

## Setting Storage & Priority

All settings follow this priority (highest → lowest):
1. System settings (hardcoded build-time)
2. Command-line arguments
3. Environment variables
4. Database (`settings` table)
5. settings.yaml file
6. Defaults (hardcoded in `build_all_defaults()`)

**Database persistence**: Only settings in `SETTING_VARS` constant or matching `BODHI_LLAMACPP_ARGS_*` pattern can be persisted to the `settings` table.

**Editable via API**: Only keys in `EDIT_SETTINGS_ALLOWED` allowlist in `routes_app/src/settings/routes_settings.rs`. Currently: `BODHI_EXEC_VARIANT`, `BODHI_KEEP_ALIVE_SECS`.

---

## Multi-Tenant Impact

In multi-tenant mode:
- LLM settings (section 3) become dead code
- BODHI_EXEC_VARIANT editing should be disabled
- All other settings remain global and applicable
- No per-tenant settings needed for initial launch
- If per-tenant settings needed later → create `tenant_settings` table (decision D9)
