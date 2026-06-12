# Plan: Complete ts-client Re-Export Surface in bodhi-js-sdk

## Context

The downstream app `youtube-ai-chat` (uses `@bodhiapp/bodhi-js-react@0.0.33`) had to roll its own `ApiFormat` type and duplicate several ts-client response types because the SDK did not expose them via a discoverable subpath. Root cause: the SDK re-export chain only provides `/api` (full ts-client main barrel) and `/api/openai` subpaths — **no `/api/anthropic` or `/api/gemini` subpaths** — combined with insufficient docs, so consumers either added `@bodhiapp/ts-client` as a direct dependency or redefined types locally.

There is a second, related inconsistency: `UserScope`, `FlowType`, `RequestedResourcesV1` are currently re-exported from the main barrel of `core` / `react-core` / `react` / `react-ext`. These are ts-client management types — they should come from `/api` like every other management type. Since there is no public release yet, this is the right moment to clean up rather than grandfather the exceptions.

Convention confirmed: consumers import management types via the `/api` subpath. `ApiFormat` is already reachable today via `@bodhiapp/bodhi-js-react/api` (which re-exports `export * from '@bodhiapp/ts-client'`). The fix is to (a) make `/api/anthropic` and `/api/gemini` work, (b) remove the grandfathered management-type exports from the main barrel, and (c) document the convention in the public-facing CLAUDE.md files so the mistake is not repeated.

Intended outcome: downstream apps that depend on `@bodhiapp/bodhi-js-react` or `@bodhiapp/bodhi-js-react-ext` never need `@bodhiapp/ts-client` as a direct dependency, and there is exactly one canonical import path per type.

## Convention

| Import | Surface |
| --- | --- |
| `@bodhiapp/bodhi-js-react` | App-facing types & classes only: `AuthState`, `ClientState`, `UIClient`, `LoginOptions`, `BodhiError`, `BodhiApiError`, `unwrapResponse`, `BodhiProvider`, `useBodhi`, error type guards, etc. **No ts-client management types.** |
| `@bodhiapp/bodhi-js-react/api` | Full ts-client management types: `ApiFormat`, `UserScope`, `FlowType`, `RequestedResourcesV1`, `PaginatedAliasResponse`, `AliasResponse`, `ApiModel`, etc. |
| `@bodhiapp/bodhi-js-react/api/openai` | OpenAI-compat types — already works |
| `@bodhiapp/bodhi-js-react/api/anthropic` | Anthropic types — **new** |
| `@bodhiapp/bodhi-js-react/api/gemini` | Gemini types — **new** |

Same layout for `@bodhiapp/bodhi-js-react-ext`. Lower-level packages (`core`, `react-core`) expose the same `/api/*` subpaths.

**Rule for future SDK changes**: do not add ts-client management types to any main barrel. They live under `/api/*` only.

## Scope

**In scope**

1. Add `/api/anthropic` and `/api/gemini` subpaths in all 4 packages (`core`, `react-core`, `react`, `react-ext`).
2. Remove `UserScope`, `FlowType`, `RequestedResourcesV1` from the main barrels (they remain reachable via `/api`).
3. Migrate in-repo consumers that import these three types from the main barrel.
4. Update CLAUDE.md in both public-facing packages (`react`, `react-ext`) with the convention and a "do not add management types to main barrel" rule.
5. Update `bodhi-js-sdk/CLAUDE.md` and `bodhi-js-sdk/skills/bodhi-sdk/SKILL.md` with the subpath table + worked example.

**Out of scope**

- Typed SDK helper methods on the client (e.g., `client.listAliases()`) — separate design.
- Refactoring `youtube-ai-chat` — covered only as a verification smoke test.

## Existing Pattern to Reuse

The `/api/openai` subpath is wired end-to-end — copy it verbatim for anthropic/gemini.

- `bodhi-js-sdk/core/src/api/openai.ts` → `export * from '@bodhiapp/ts-client/openai';`
- `bodhi-js-sdk/react-core/src/api/openai.ts` → `export * from '@bodhiapp/bodhi-js-core/api/openai';`
- `bodhi-js-sdk/react/src/api/openai.ts`, `react-ext/src/api/openai.ts` — same pattern
- `bodhi-js-sdk/core/package.json` → `exports["./api/openai"]`
- `bodhi-js-sdk/core/vite.config.ts` → `lib.entry["api/openai"]`
- `bodhi-js-sdk/react/vite.config.ts` externals include `@bodhiapp/bodhi-js-core/api/openai` and `@bodhiapp/bodhi-js-react-core/api/openai`
- `bodhi-js-sdk/scripts/fix-api-subpath-imports.js` — post-build `.d.ts` rewriter invoked from the monorepo top-level Makefile. Confirm it globs `dist/api/*.d.ts`; if it hard-codes `openai`, extend it for `anthropic` and `gemini`.

## Implementation Steps

### 1. Add `/api/anthropic` and `/api/gemini` (4 packages)

**core**

- NEW `bodhi-js-sdk/core/src/api/anthropic.ts` → `export * from '@bodhiapp/ts-client/anthropic';`
- NEW `bodhi-js-sdk/core/src/api/gemini.ts` → `export * from '@bodhiapp/ts-client/gemini';`
- `bodhi-js-sdk/core/package.json` → add `"./api/anthropic"` and `"./api/gemini"` entries to `exports`
- `bodhi-js-sdk/core/vite.config.ts` → add `'api/anthropic'` and `'api/gemini'` entries to `lib.entry`
- Externals already cover `/^@bodhiapp\/ts-client(\/.*)?$/` — no change needed

**react-core**

- NEW `bodhi-js-sdk/react-core/src/api/anthropic.ts` → `export * from '@bodhiapp/bodhi-js-core/api/anthropic';`
- NEW `bodhi-js-sdk/react-core/src/api/gemini.ts` → `export * from '@bodhiapp/bodhi-js-core/api/gemini';`
- `bodhi-js-sdk/react-core/package.json` → add exports entries
- `bodhi-js-sdk/react-core/vite.config.ts` → add `lib.entry` entries + add `@bodhiapp/bodhi-js-core/api/anthropic` and `@bodhiapp/bodhi-js-core/api/gemini` to externals

**react** (public-facing)

- NEW `bodhi-js-sdk/react/src/api/anthropic.ts` → `export * from '@bodhiapp/bodhi-js-core/api/anthropic';`
- NEW `bodhi-js-sdk/react/src/api/gemini.ts` → `export * from '@bodhiapp/bodhi-js-core/api/gemini';`
- `bodhi-js-sdk/react/package.json` → add exports entries
- `bodhi-js-sdk/react/vite.config.ts` → add entries + externals for both core and react-core subpaths

**react-ext** (public-facing)

- NEW `bodhi-js-sdk/react-ext/src/api/anthropic.ts` → same pattern as react
- NEW `bodhi-js-sdk/react-ext/src/api/gemini.ts` → same
- `bodhi-js-sdk/react-ext/package.json` → add exports entries
- `bodhi-js-sdk/react-ext/vite.config.ts` → add entries + externals

### 2. Remove management types from main barrels

- `bodhi-js-sdk/core/src/types/index.ts:75` — delete the line `export type { FlowType, RequestedResourcesV1, UserScope } from '@bodhiapp/ts-client';`. Leave the `import type { ... }` at line 84 intact (used internally for the `LoginOptions` interface).
- `bodhi-js-sdk/react-core/src/index.ts:65–67` — remove `UserScope`, `FlowType`, `RequestedResourcesV1` from the named re-export from `@bodhiapp/bodhi-js-core`.
- `bodhi-js-sdk/react/src/index.ts:36–38` — remove the three type names from the named re-export from `@bodhiapp/bodhi-js-react-core`.
- `bodhi-js-sdk/react-ext/src/index.ts:36–38` — same as react.

### 3. Migrate in-repo consumers

- `sdk-test-app/web/src/sdk.ts:24–26` — change the re-export of `UserScope`, `FlowType`, `RequestedResourcesV1` to pull from `@bodhiapp/bodhi-js-react/api` instead of `@bodhiapp/bodhi-js-react`.
- `sdk-test-app/ext/src/sdk.ts:24–26` — same, pulling from `@bodhiapp/bodhi-js-react-ext/api`.
- `sdk-test-app/{web,ext}/src/app/components/setup/AuthStatusSection.tsx` — no change (they import from `@/sdk` which is the local barrel re-exporting these; the local barrel still exposes them).
- `bodhi-js-sdk/cli/src/types.ts:2` — already imports `RequestedResourcesV1` directly from `@bodhiapp/ts-client`; line 1 imports `UserScope` from `@bodhiapp/bodhi-js-core`. Change both to `@bodhiapp/bodhi-js-core/api` for consistency with the new convention.

### 4. Documentation — public-facing packages

- `bodhi-js-sdk/react/CLAUDE.md` — add a **"Type import paths"** section containing the convention table from this plan, plus an explicit rule: _"Do not add types from `@bodhiapp/ts-client` to the main barrel. Management types belong under `/api/*`. The main barrel is reserved for app-facing types (AuthState, ClientState, UIClient, error classes, React primitives)."_
- `bodhi-js-sdk/react-ext/CLAUDE.md` — add the same section.
- `bodhi-js-sdk/CLAUDE.md` — add the subpath table and a short worked example:

  ```ts
  import { BodhiProvider, useBodhi, BodhiError } from '@bodhiapp/bodhi-js-react';
  import type { ApiFormat, PaginatedAliasResponse } from '@bodhiapp/bodhi-js-react/api';
  import type { CreateMessageRequest } from '@bodhiapp/bodhi-js-react/api/anthropic';
  import type { GenerateContentRequest } from '@bodhiapp/bodhi-js-react/api/gemini';
  ```

  Note explicitly that downstream apps should **not** depend directly on `@bodhiapp/ts-client`.
- `bodhi-js-sdk/skills/bodhi-sdk/SKILL.md` — same convention table and example. Also add an entry to the "common mistakes" / troubleshooting section if one exists, stating that adding `@bodhiapp/ts-client` as a direct dep is a smell — use the `/api/*` subpaths instead.

## Critical Files

| File | Change |
| --- | --- |
| `bodhi-js-sdk/core/src/api/anthropic.ts` | NEW |
| `bodhi-js-sdk/core/src/api/gemini.ts` | NEW |
| `bodhi-js-sdk/core/package.json` | add `./api/anthropic`, `./api/gemini` exports |
| `bodhi-js-sdk/core/vite.config.ts` | add two `lib.entry` lines |
| `bodhi-js-sdk/core/src/types/index.ts` | delete line 75 |
| `bodhi-js-sdk/react-core/src/api/anthropic.ts` | NEW |
| `bodhi-js-sdk/react-core/src/api/gemini.ts` | NEW |
| `bodhi-js-sdk/react-core/src/index.ts` | remove 3 type names (lines 65–67) |
| `bodhi-js-sdk/react-core/package.json` | add exports |
| `bodhi-js-sdk/react-core/vite.config.ts` | entries + externals |
| `bodhi-js-sdk/react/src/api/anthropic.ts` | NEW |
| `bodhi-js-sdk/react/src/api/gemini.ts` | NEW |
| `bodhi-js-sdk/react/src/index.ts` | remove 3 type names (lines 36–38) |
| `bodhi-js-sdk/react/package.json` | add exports |
| `bodhi-js-sdk/react/vite.config.ts` | entries + externals |
| `bodhi-js-sdk/react/CLAUDE.md` | convention section + "no management types in main barrel" rule |
| `bodhi-js-sdk/react-ext/src/api/anthropic.ts` | NEW |
| `bodhi-js-sdk/react-ext/src/api/gemini.ts` | NEW |
| `bodhi-js-sdk/react-ext/src/index.ts` | remove 3 type names (lines 36–38) |
| `bodhi-js-sdk/react-ext/package.json` | add exports |
| `bodhi-js-sdk/react-ext/vite.config.ts` | entries + externals |
| `bodhi-js-sdk/react-ext/CLAUDE.md` | convention section + rule |
| `bodhi-js-sdk/scripts/fix-api-subpath-imports.js` | confirm it globs `dist/api/*.d.ts`; extend if hard-coded |
| `bodhi-js-sdk/CLAUDE.md` | convention table + example |
| `bodhi-js-sdk/skills/bodhi-sdk/SKILL.md` | convention + example + smell note |
| `sdk-test-app/web/src/sdk.ts` | switch 3 type re-exports to `/api` subpath |
| `sdk-test-app/ext/src/sdk.ts` | switch 3 type re-exports to `/api` subpath |
| `bodhi-js-sdk/cli/src/types.ts` | switch imports to `@bodhiapp/bodhi-js-core/api` |

## Verification

1. **Build**: `cd bodhi-js-sdk && make build` — all 4 packages produce `dist/api/anthropic.*` and `dist/api/gemini.*` (`.esm.js`, `.cjs.js`, `.d.ts`).
2. **Typecheck**: `cd bodhi-js-sdk && make typecheck` — passes for all packages.
3. **`.d.ts` post-process**: inspect `bodhi-js-sdk/{core,react-core,react,react-ext}/dist/api/anthropic.d.ts` and `gemini.d.ts`. Shapes should mirror the existing `openai.d.ts`.
4. **sdk-test-app typecheck**: `cd sdk-test-app && npm run typecheck` — confirms the 3 migrated re-exports in `web/src/sdk.ts` / `ext/src/sdk.ts` compile against the new `/api` subpath.
5. **sdk-test-app E2E regression**: `make dev.test.sdk-test-app` — smoke-test the builds.
6. **Scratch consumer check** — in `sdk-test-app/web/src/` or equivalent, add temporarily:

   ```ts
   import type {
     ApiFormat,
     PaginatedAliasResponse,
     AliasResponse,
   } from '@bodhiapp/bodhi-js-react/api';
   import type { CreateMessageRequest } from '@bodhiapp/bodhi-js-react/api/anthropic';
   import type { GenerateContentRequest } from '@bodhiapp/bodhi-js-react/api/gemini';
   const _fmt: ApiFormat = 'anthropic';
   ```

   Run typecheck. Remove the scratch block.
7. **Downstream smoke test**: point `youtube-ai-chat` at the locally-built SDK (`npm link` or `file:` reference), delete its local `ApiFormat` definition (`src/lib/agent-model.ts:5`) and `PaginatedAliasResponse` / `AliasEntry` / `ApiModel` interfaces (`src/lib/bodhi-models.ts:9–33`), re-import them from `@bodhiapp/bodhi-js-react/api`, then run `tsc --noEmit` in that repo. Should compile without adding `@bodhiapp/ts-client` as a direct dependency.
