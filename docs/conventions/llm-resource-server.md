# Bodhi App as an LLM Resource Server

Product and architecture vision. Bodhi App is a secure, OAuth2-based LLM resource server: a broker between client applications and local AI capabilities, giving users complete control over who can access their local models. This doc is the *vision*; implementation truth lives in the crate docs (`crates/services/CLAUDE.md`, `crates/routes_app/CLAUDE.md`).

## Core Vision

Bodhi App is the central node in a local AI ecosystem:

- **Local control** — on-premise deployment, complete data privacy, local resource management.
- **Secure access** — OAuth2 authentication + role-based authorization on every request.
- **Ecosystem integration** — standardized OpenAI-compatible APIs for client apps.
- **Future expansion** — a foundation for a broader GenAI resource server (multi-modal).

> Authentication is unconditional — every deployment registers as an OAuth2 resource server and enforces RBAC. (Earlier drafts described an optional "non-authenticated mode"; it has been removed.) See `docs/conventions/setup-processes.md`.

## Architecture Components

1. **LLM inference engine** — local llama.cpp servers; model load, inference, resource allocation. (`llama_server_proc`, `server_core`.)
2. **Auth & authorization** — OAuth2 token exchange, session auth (web cookies) and token auth (JWT) for clients, hierarchical RBAC. (`routes_app` middleware, `services` auth module.)
3. **API surface** — OpenAI-compatible REST endpoints, streaming, stateless design for horizontal scale. Also Anthropic/Gemini proxy formats.
4. **Model management** — HuggingFace download + local cache, alias system. (`services` models module.)
5. **Setup & configuration** — guided setup wizard, admin assignment, responsive UI.

## Authentication & Authorization

### RBAC

Role hierarchy (`ResourceRole`, see `crates/services/CLAUDE.md`): `Anonymous < Guest < User < PowerUser < Manager < Admin`. Key admin-assignable roles: resource admin (full administration), resource manager (model/user oversight), resource user (inference + own tokens).

### Tokens

- Stateless JWT tokens, independent of sessions, with configurable expiry/idle timeouts.
- API token format `sk-bodhiapp_<random><checksum>.<client_id>` (see `crates/services/CLAUDE.md`).
- Scope validation, token caching with invalidation, full revocation.

### OAuth2 Client Integration

```
Client App ── OAuth2 Authorization Request ─▶ Bodhi App
                                                  │
                                          User Consent UI
                                                  │
Client App ◀── Authorization Code ────────────────┘
   │
   ▼
Token Exchange ── Access Token ─▶ Resource Access
```

Clients register, request authorization, the user grants consent through the Bodhi App UI, and the client exchanges the code for an access token used on subsequent API calls. Client tokens are exchanged into local resource tokens with the appropriate privileges. OAuth-backed API formats use proper server-side PKCE/callback flows (never paste-token).

## Model Management & Aliases

- **HuggingFace integration** — idempotent, asynchronous downloads; local cache with metadata.
- **Alias system** — user-defined aliases separating request params (`OAIRequestParams`) from server context flags. Multiple alias kinds: local model, API model (OpenAI/Anthropic/Gemini/etc.), and model-router composites. See `crates/services/CLAUDE.md`.

## API Design Principles

- **Stateless** — each request self-contained; horizontal scaling without session affinity.
- **Security-integrated** — token validation + role checks on every request; clear authz errors; audit logging.
- **OpenAI-compatible** — `/v1/chat/completions`, `/v1/models`, plus Anthropic/Gemini proxy formats. Token management under `/bodhi/v1/tokens`.

## Client Integration Methods

- **Direct local access** — same-machine apps, minimal latency.
- **Remote / browser** — browser extensions and web apps via OAuth2 + proxy endpoints with secure token handling.
- **Cross-platform** — Mac, Linux, Windows desktop; Docker web server; PWA.

## Strategic Direction

The long-term goal is a comprehensive AI resource provider where users dictate which applications access their local AI, with granular, auditable, revocable permissions. Planned modality expansion: text-to-image, speech-to-text, text-to-audio, multimodal — under a unified API design.

### Differentiation

- **Security-first** — standard OAuth2 + RBAC + audit trails throughout.
- **Local control & privacy** — on-premise, data sovereignty, no external dependency for core inference.
- **Ecosystem** — OpenAI/Anthropic/Gemini-compatible APIs, SDKs, developer tooling.
- **Platform independence** — consistent behavior across all deployment targets.

## Status Snapshot

Implemented: OAuth2 auth + session management, RBAC, API token lifecycle, stateless JWT with caching, OpenAI-compatible APIs, HuggingFace model management, alias system, llama.cpp integration, setup wizard. In progress / planned: broader client SDKs, additional API compatibility layers, multi-modal capabilities, distributed multi-instance resource management.

For current, code-accurate detail always defer to the crate-level `CLAUDE.md` files.
