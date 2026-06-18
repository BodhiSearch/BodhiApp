# UI V2 Migration — Architecture Orientation

Where things live and the direction of the work. **Pointers, not a file:line map** — the code
changes as the migration progresses, so explore the current tree rather than trusting a snapshot.
Follow the repo conventions in `crates/bodhi/src/CLAUDE.md`, `crates/CLAUDE.md`, and root `CLAUDE.md`.

## Production frontend — `crates/bodhi`

- **Stack:** Vite, TanStack Router (file-based routing, basepath `/ui`, trailing-slash always),
  TanStack Query v5, shadcn/ui, TailwindCSS, `lucide-react`, a `ThemeProvider` (light/dark/system).
- **Routes:** `src/routes/` (file-based). Each of the 5 nav sections maps to one or more route
  folders; the new design also introduces at least one **new page route** (a create flow that was a
  dialog). Explore `src/routes/` to find the current screen↔route mapping; cross-check against the
  prototype screens in `design/`.
- **Root layout:** `src/routes/__root.tsx` composes the providers + the current top-header chrome.
  The migration makes **AppShell the root layout**; the old top-header nav
  (`src/components/navigation/`) is superseded and deleted at the very end.
- **Data layer (reused as-is):** domain hooks under `src/hooks/<domain>/` (constants/endpoints +
  query-key factories + `useX` hooks + barrels), generic wrappers in `src/hooks/useQuery.ts`, axios
  client in `src/lib/apiClient.ts`. Every `/bodhi/v1/*` endpoint these screens need already has a
  hook — confirm by reading the relevant `src/hooks/<domain>/`.
- **Shared UI:** shadcn primitives in `src/components/ui/`; app-level components in
  `src/components/`; feature components colocated under `src/routes/<domain>/-components/`.
- **Theming/tokens:** `src/styles/globals.css` (shadcn HSL CSS variables, light + `.dark`) and
  `tailwind.config.ts`. The design tokens (`design/colors_and_type.css`) are a **refresh** of these
  same variables (notably the brand `--primary`), merged in at the foundation batch.
- **Tests:** Vitest + React Testing Library + MSW v2 (OpenAPI-typed handlers in
  `src/test-utils/msw-v2/`) + fixtures in `src/test-fixtures/`; route tests colocated as
  `index.test.tsx`. See [process.md](@process.md) for e2e + the testid discipline.
- **Types:** `@bodhiapp/ts-client`, generated from the backend OpenAPI spec.

## Backend — Rust crates

- **Layering & conventions:** `crates/CLAUDE.md` (crate chain, work upstream→downstream).
- **Config:** a `SettingService` is the configuration mechanism (env > db > file > default), in
  `crates/services/src/settings/`. New configurable values (e.g. the reference-API endpoint) are
  added there as settings.
- **Auth/session & user identity:** OAuth login + session live in `crates/services/src/auth/` and
  the auth routes in `crates/routes_app/src/auth/`; session keys in
  `crates/services/src/session_keys.rs`; the current-user response in
  `crates/routes_app/src/users/`. The app-info response is in `crates/routes_app/src/setup/`.
- **The Batch-0 backend deltas** (surface the OAuth id_token + a reference-API endpoint config) are
  described directionally in [reference-api.md](@reference-api.md). Explore the current signatures
  before implementing — they may have changed.
- **OpenAPI → ts-client sync:** after any backend API change, regenerate and rebuild the TS client
  (`cargo run --package xtask openapi` then `make build.ts-client`) so the frontend types + MSW
  schema types track. Commit the regenerated client in the same batch.

## Directional notes for the AppShell port

- Port `design/bodhi-app-shell.jsx` into production TSX (a `components/shell/` area), translating
  prototype idioms to ours: global `lucide` → `lucide-react`; `window.*`/`ReactDOM.createRoot`
  removed; `data-theme` setattr removed (theme is owned by `ThemeProvider`); nav uses TanStack
  `<Link>` with real trailing-slash route hrefs; nav highlight is driven by each route declaring its
  `section`/`subPage`. The exact component split and flag mechanism are for the batch plan to
  propose after exploring the current code — keep it idiomatic to whatever the tree looks like then.
