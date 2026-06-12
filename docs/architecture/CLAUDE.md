# docs/architecture/ — CLAUDE.md

System design and architectural context. For code-level crate detail, follow links to `crates/<crate>/CLAUDE.md`.

| Doc | Covers |
|---|---|
| `system-overview.md` | High-level architecture: crate dependency chain, tech stack (Rust+Axum+SeaORM, Vite+React+TanStack), API surfaces (OpenAI/Ollama/Anthropic/Responses), key features |
| `bodhi-platform.md` | Multi-component platform: Bodhi App, browser extension, bodhi.js, auth server, and how they integrate |
| `architectural-decisions.md` | ADR-style rationale: dumb-frontend, Rust-first, SeaORM, TanStack Query, OAuth, crate organization |
| `authentication.md` | OAuth2/OIDC/Keycloak, JWT, sessions vs API tokens, RBAC (`ResourceRole` hierarchy) |
| `app-status.md` | `AppStatus` state machine (`setup → resource_admin → ready`) and the `GET /bodhi/v1/info` shape |
| `tauri-desktop.md` | Desktop app: native feature flag, embedded-UI bundle (`include_dir!` over Vite `out/`), dual-mode |
| `security.md` | **Mandatory pre-read for any security work** — known accepted risks, by-design decisions, remediated vulns |
