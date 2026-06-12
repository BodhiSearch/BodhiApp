# Migrate `ai-docs/` → `docs/`: a clean, deduped, progressive-disclosure doc tree

## Context

`ai-docs/` has grown to **879 markdown files (15 MB)** across 16 top-level folders with severe rot:
- **Systemic staleness**: 13/25 files in `01-architecture/` reference Next.js (the app is now Vite + React + TanStack Router/Query v5), 10/25 reference the removed `objs` crate, and `ai-ide-memories.md` actively asserts "React Query v3.39.3 (not TanStack Query)" as current truth — directly misleading any coding assistant.
- **Triplicated crate docs**: per-crate docs exist in `03-crates/` (stale, Mar 2026), `context/` (a symlink index, 12 of 53 links broken), and the canonical `crates/<crate>/CLAUDE.md` in the source tree.
- **Mislabeled archives**: files in `active-stories/`/`planned/` are self-marked "Completed"; `02-features/implemented/` duplicates `completed-stories/`.
- The root `README.md` index is a Next.js-era artifact.

Rather than clean in place, we **build a fresh `docs/` tree** containing only relevant, audited, deduped docs, applying YAGNI. `ai-docs/` is **left physically untouched as a frozen dead snapshot** — nothing is deleted from it. Migration is **copy-only**. The end state: a token-dense, progressive-disclosure doc tree for Claude Code / AI coding assistants, with `crates/<crate>/CLAUDE.md` remaining the canonical source for crate-level docs (linked, never copied).

This plan is the output of a 6-agent parallel analysis of the full corpus (categorization recorded below) plus a Plan-agent design pass.

### Binding decisions (from user)
1. **Copy, don't move.** `ai-docs/` stays in place as trash; we never `git rm` it.
2. **Crate docs**: `crates/<crate>/CLAUDE.md` is canonical. `03-crates/` and `context/` are trash (not migrated).
3. **Marketing** (`05-marketing/`, `context/bodhi-blurb.md`, `claude-plans/cowork/`): left in `ai-docs/`, not migrated.
4. **Naming**: plain semantic folder names (`architecture/`, `guides/`).
5. **Archives**: `specs/` + `claude-plans/` → `docs/archive/` using the **`yyyymm/` subfolder convention**. `claude-plans/` already uses it (preserve); `specs/` flat dated dirs get regrouped under `yyyymm/`.
6. **Authoring repointed**: new plans/specs land in **`docs/claude-plans/202606/`** and **`docs/specs/`** so `ai-docs/` truly freezes.
7. **Stale configs**: fix `.claude/` + root md files; **delete `.kiro/`** (after salvaging anything useful); skip `.cursor/`.
8. **Security doc** → `docs/architecture/security.md`.
9. **Concept-rescue rule**: when a doc is trashed for staleness, do NOT blindly drop it — first check whether the *concept* it covers is important enough to deserve a **fresh doc written from current code as the source-of-truth snapshot**. If the concept is already canonically covered by a live doc (`crates/<crate>/CLAUDE.md`, `crates/bodhi/src/TESTING.md`, etc.), link to that instead of recreating. Only author a new doc when the concept matters AND no current-truth doc covers it. (See "Concept-rescue audit" below.)
10. **Testing patterns are first-class**: testing conventions have prevented many quality regressions and must be strongly documented and perpetuated. In particular, the **optimized E2E style** — one user journey per test built from many `test.step()` blocks, each a setup→action→assertion cycle, reusing accumulated server/session/DB state (server spin-up is expensive) — is a deliberate, load-bearing convention to document prominently and continue. A new `docs/conventions/testing.md` captures this.

---

## Target `docs/` structure (YAGNI — only folders with content get an index)

```
docs/
  CLAUDE.md                      # top-level progressive-disclosure hub (~60-100 lines)
  architecture/
    CLAUDE.md
    bodhi-platform.md            # cleanup: minor Next.js fix
    system-overview.md           # HEAVY rewrite: drop SQLx/objs, match crate chain
    architectural-decisions.md   # cleanup: objs refs
    authentication.md            # cleanup: drop stray Next.js, verify roles
    app-status.md                # cleanup: verify AppStatus enum vs code
    tauri-desktop.md             # cleanup: one Next.js mention
    security.md                  # MUST-KEEP (root CLAUDE.md mandatory pre-read)
  guides/                        # external API-consumer guides
    CLAUDE.md
    README.md overview.md getting-started.md authentication.md
    app-to-bodhi-oauth.md openai-api.md ollama-api.md bodhi-api.md
    model-management.md api-reference.md error-handling.md examples.md
    embedded-ui.md app-bindings-guide.md
  conventions/
    CLAUDE.md
    testing.md                         # NEW (code-sourced) — testing hub, batched-test.step E2E philosophy
    test-utils-packaging.md            unused-upgrade-dependencies.md
    cuda-dockerfile-optimizations.md   github-workflows-context.md
    llm-resource-server.md             model-parameters.md  setup-processes.md
  deployments/                   # no index (2 files, linked from root)
    railway.md railway-multi-tenant.md
  research/
    CLAUDE.md
    model-router/                # entire dir incl phasewise-impl/, as-is
  notes/                         # open/forward work (distinct from dead archive)
    CLAUDE.md
    20260219-password-input.md            20260529-anthropic-oauth-spoof-removal.md
    20260409-graceful-shutdown-fix.md     20260414-prompt-openai-codex-oauth.md
    refactor-msw-typesafe.md              roadmap.md
  claude-plans/202606/           # NEW active plan-authoring area (repointed)
  specs/                         # NEW active spec-authoring area (repointed)
  archive/                       # frozen snapshot — NO CLAUDE.md (don't invite loading a tomb)
    specs/<yyyymm>/<yyyymmdd-name>/...     # regrouped from flat dated dirs
    claude-plans/...                       # 202601..202606, archived/, stray plan
    features/...                            # 02-features/{completed,implemented,active,mcp} + planned(−roadmap) + features/20260207-cleanup
    knowledge-transfer/...                 # 06-kt dated writeups
    misc/...                               # root completed specs, 99-archive/, research-archival set
```

**Collapsed for YAGNI**: no `docs/security/` (single file → `architecture/security.md`); `deployments/` and `archive/**` get no `CLAUDE.md`.

---

## Per-file migration map (the contract)

### MIGRATE — live / convention / context / guides
| Source | Dest | Action |
|---|---|---|
| `guides/bodhi-app/*` (14 files) | `docs/guides/` | as-is, except: overview.md + embedded-ui.md fix "Next.js 14"→"Vite + TanStack Router"; api-reference.md + overview.md add `/anthropic/v1` + `/v1/responses` APIs |
| `guides/app-bindings-guide.md` | `docs/guides/app-bindings-guide.md` | cleanup: drop pinned `v0.0.13` |
| `func-specs/security/security.md` | `docs/architecture/security.md` | as-is |
| `deployments/{railway,railway-multi-tenant}.md` | `docs/deployments/` | as-is |
| `01-architecture/bodhi-platform.md` | `docs/architecture/` | light cleanup |
| `01-architecture/architectural-decisions.md` | `docs/architecture/` | light cleanup (objs) |
| `01-architecture/authentication.md` | `docs/architecture/` | light cleanup |
| `01-architecture/app-status.md` | `docs/architecture/` | verify enum |
| `01-architecture/tauri-desktop.md` | `docs/architecture/` | light cleanup |
| `01-architecture/system-overview.md` | `docs/architecture/` | HEAVY rewrite (SQLx/objs) |
| `context/test-utils-packaging.md` | `docs/conventions/` | as-is |
| `06-knowledge-transfer/unused-upgrade-dependencies.md` | `docs/conventions/` | cleanup |
| `context/cuda-Dockerfile-optimizations.md` | `docs/conventions/cuda-dockerfile-optimizations.md` | de-reference line numbers |
| `context/github-workflows-context.md` | `docs/conventions/` | freshness pass |
| `06-knowledge-transfer/llm-resource-server.md` | `docs/conventions/` | cleanup |
| `06-knowledge-transfer/model-parameters.md` | `docs/conventions/` | verify defaults |
| `06-knowledge-transfer/setup-processes.md` | `docs/conventions/` | drop non-authz path |
| `07-research/model-router/**` | `docs/research/model-router/` | as-is (active project) |
| `tech-debt/20260219-password-input.md` | `docs/notes/` | as-is |
| `tech-debt/20260529-anthropic-oauth-spoof-removal.md` | `docs/notes/` | as-is |
| `claude-plans/deferred/{20260409-graceful-shutdown-fix,20260414-prompt-openai-codex-oauth}.md` | `docs/notes/` | as-is |
| `specs/pending/refactor-msw-typesafe.md` | `docs/notes/` | as-is |
| `02-features/planned/roadmap.md` | `docs/notes/roadmap.md` | cleanup |
| — (NEW, authored from code) | `docs/conventions/testing.md` | write fresh; see "Testing doc" below |

`context/claude-package-generate.md` → **fold into `MDFILES.md`** (it overlaps); do not copy. If not folded, park in `docs/conventions/`.

### ARCHIVE (copy untouched into `docs/archive/`, no link rewriting)
- `specs/` (all dated dirs minus `pending/`) → `docs/archive/specs/<yyyymm>/<yyyymmdd-name>/` (regroup: 202509×20, 202510×9, 202511×1, 202601×4)
- `claude-plans/{202601..202606,archived}` + `smooth-weaving-blossom.md` → `docs/archive/claude-plans/`
- `02-features/{completed-stories,implemented,active-stories,mcp}` + `planned/`(−roadmap.md) + `features/20260207-cleanup/` → `docs/archive/features/`
- `06-knowledge-transfer/` dated writeups (`20250614-*`, `20250616-*`, `oauth-testing-fixes.md`) → `docs/archive/knowledge-transfer/`
- `99-archive/`, root `token-exchange-prompt.md`, `makefile-reorganization-requirements.md`, research-archival set (`07-research/{token-exchange,appreginfo-jwt-simplification-analysis,docker-build-optimization-plan,20250615-ffi-ui-testing-research,20260110-lna*}.md`) → `docs/archive/misc/`

### TRASH (do NOT migrate; left in `ai-docs/`)
- All `01-architecture/` frontend subgroup + indexes: `frontend-react.md`, `frontend-query.md`, `api-integration.md`, `frontend-testing.md`, `TESTING_GUIDE.md`, `ai-ide-memories.md`, `README.md`, `ARCHITECTURE_SUMMARY.md`, `development-conventions.md`, `testing-strategy.md`, `ui-design-system.md`
- All `01-architecture/` backend deep-dives (superseded by crate CLAUDE.md): `backend-error-l10n.md`, `backend-settings-service.md`, `backend-openapi-utoipa.md`, `backend-testing.md`, `backend-testing-utils.md`, `rust-backend.md`, `backend-development-conventions.md`
- All `03-crates/`, all `context/` symlinks + `README.md` + `oauth2-token-exchange-auth-service-context.md`
- `10-guide/` (both files), `02-features/{README,active-stories/llama-cpp-args/}`, `06-kt/README.md`, `02-features/README.md`
- `07-research/{web40,lna}.md`, `07-research/README.md`
- `05-marketing/`, `context/bodhi-blurb.md`, `claude-plans/cowork/` (marketing — stays in ai-docs)
- root `README.md` (replaced by `docs/CLAUDE.md`)

---

## Concept-rescue audit (every trash verdict, re-checked)

Before dropping a stale doc, we ask: *is the concept important, and is it covered by a current-truth doc?* Outcomes:

| Trashed doc(s) | Concept | Covered by current truth? | Action |
|---|---|---|---|
| `frontend-react.md`, `frontend-query.md`, `api-integration.md`, `ui-design-system.md` | Frontend architecture, TanStack Router/Query, design system, dumb-frontend | **Yes** — `crates/bodhi/src/CLAUDE.md` (live) | Trash; link from `docs/architecture/CLAUDE.md` to `crates/bodhi/src/CLAUDE.md`. No new doc. |
| `frontend-testing.md`, `TESTING_GUIDE.md`, `testing-strategy.md` | Frontend + overall test strategy | **Partly** — `crates/bodhi/src/TESTING.md` (components) + `tests-js/E2E.md` (E2E) exist, but **no single cross-layer testing hub** | **Author `docs/conventions/testing.md`** (new, code-sourced) as the hub; link out to the layer docs. |
| `rust-backend.md`, `backend-development-conventions.md` | Backend service/db patterns, crate org | **Yes** — `crates/CLAUDE.md` + `crates/services/CLAUDE.md` (live) | Trash; link. No new doc. |
| `backend-error-l10n.md` | Fluent + ErrorMeta + l10n | **Yes** — `crates/errmeta*/CLAUDE.md` + memory [[feedback_errmeta_gotchas]] | Trash; link. No new doc. |
| `backend-settings-service.md` | Cascaded settings architecture | **Yes** — `crates/services/CLAUDE.md` (SettingService) | Trash; link. No new doc. |
| `backend-openapi-utoipa.md` | utoipa/OpenAPI→TS pipeline | **Yes** — `crates/routes_app/CLAUDE.md` + `crates/CLAUDE.md` (OpenAPI→TypeScript section) | Trash; link. No new doc. |
| `backend-testing.md`, `backend-testing-utils.md` | Backend test patterns, dual-availability test-utils | **Yes** — `crates/routes_app/TESTING.md`, `crates/services/src/test_utils/CLAUDE.md`, `test-services`/`test-routes-app` skills; convention preserved in `docs/conventions/test-utils-packaging.md` | Trash; surface via `docs/conventions/testing.md` hub. No new doc. |
| `ai-ide-memories.md`, `ARCHITECTURE_SUMMARY.md`, `development-conventions.md` | AI-IDE memory, arch summary, coding standards | **Yes** — root `CLAUDE.md` + `crates/CLAUDE.md` + `~/.claude` memories | Trash. No new doc. |
| `10-guide/*`, `03-crates/*`, `context/` symlinks | Integration guide, crate docs | **Yes** — `docs/guides/` (richer) + `crates/<crate>/CLAUDE.md` | Trash; link. No new doc. |

**Net new doc authored from current code: exactly one — `docs/conventions/testing.md`** (the only important concept with no consolidated current-truth home). Everything else trashed is already canonically covered; we link rather than recreate. Any agent that, mid-migration, finds a trashed concept NOT covered by a live doc must flag it for a code-sourced rescue doc rather than silently dropping it.

---

## Testing doc — `docs/conventions/testing.md` (NEW, authored from current code)

Testing conventions are load-bearing (they have caught many regressions) and must be perpetuated. This doc is a **navigation hub + philosophy statement**, written against the *current* test suite as source of truth, that links to the canonical layer docs rather than duplicating them.

**Headline section (prominent): the optimized E2E philosophy.**
- **One user journey = one `test()` with many `test.step()` blocks**, each a setup→action→assertion cycle, building on accumulated server/session/DB state.
- **Why**: server spin-up (`bodhiserver_dev`) is expensive; a shared server + batched steps reuse one session+DB instead of paying setup per assertion.
- **Anti-patterns to avoid** (per `tests-js/E2E.md`): fragmented N-tiny-tests CRUD that should be 1–2 journeys; repeated login per test; missing `Phase N:` step markers.
- Concrete quoted example: `crates/lib_bodhiserver/tests-js/specs/mcps/mcps-crud.spec.mjs` (the corrected, batched form), with `file_path:line`.

**Remaining sections (terse, each linking out):**
- Stack overview: E2E (Playwright + bodhiserver_dev) · components (Vitest + MSW v2, `crates/bodhi/src/`) · backend (Rust `rstest`, sibling `test_*.rs`) · integration boundary (`routes_app` `tower::oneshot` vs `server_app` real HTTP).
- Server decision tree (shared port 51135/41135 vs dedicated `createServerManager` for custom config / non-ready state / multi-user contexts).
- Cross-layer rules — codify the existing memories so they live in-repo too: **black-box E2E** (no `page.evaluate`/context fetch for assertions; documented setup-helper exceptions) [[feedback_blackbox_e2e]]; **never `test.skip` for missing env — throw in `beforeAll`** [[feedback_no_skip_for_missing_env]]; **plans/tests must cover all layers** [[feedback_testing_depth]]; `data-testid`/`data-test-state` selectors; no inline timeouts (wait on state attributes); backend `FrozenTimeService` (never `Utc::now()` in tests); multi-tenant isolation (`TEST_TENANT_ID`/`TEST_TENANT_B_ID`).

**Canonical docs it links to (source of truth — never duplicated):**
`crates/lib_bodhiserver/tests-js/E2E.md`, `crates/lib_bodhiserver/tests-js/CLAUDE.md`, `crates/bodhi/src/TESTING.md`, `crates/routes_app/TESTING.md`, `crates/services/src/test_utils/CLAUDE.md`, plus the `test-services` and `test-routes-app` skills under `.claude/skills/`.

**Authoring constraint**: verify every quoted snippet and `file_path:line` against the live suite at authoring time (the exploration found the structure but line numbers drift). Keep it dense; push detail into the linked docs per `MDFILES.md`.

---

## Progressive-disclosure index design

Mirror the existing hub-and-spoke (`MDFILES.md` convention; root `CLAUDE.md` → `crates/CLAUDE.md` → per-crate). `docs/` is a peer subtree reached from root `CLAUDE.md`.

- **Root `CLAUDE.md`** (edit): add `docs/CLAUDE.md` to the companion-docs header; update security ref.
- **`docs/CLAUDE.md`**: docs hub. One-line description + link per sub-area. States explicitly: "crate-level docs are canonical at `crates/<crate>/CLAUDE.md` — not duplicated here" and "`archive/` is a frozen snapshot; do not load unless doing historical research."
- **Sub-indexes** (`architecture/`, `guides/`, `conventions/`, `research/`, `notes/` each get a `CLAUDE.md`): one-line-per-file, `file_path:line` refs over snippets, ≤200 lines. `architecture/CLAUDE.md` flags `security.md` as mandatory security pre-read. `conventions/CLAUDE.md` flags `testing.md` as the **first read before writing any test** and as the home of the batched-`test.step` E2E philosophy.
- **No index** in `deployments/`, `archive/**`, `claude-plans/`, `specs/`. `research/model-router/phasewise-impl/README.md` kept as-is.

---

## Execution sequencing (leaf-first, indexes last; each phase = one sub-agent, copy-only)

Per [[feedback_layered_refactors]]: phased sub-agents; this is a refactor-style migration so **defer commits to the end** (single commit, or per-user). Per [[feedback_subagent_overload_fallback]]: on 529 overload, finish the phase in main thread.

- **Phase 0 — Scaffold & manifest** (1 agent): create `docs/` subfolder tree; emit the source→dest manifest (the lookup table for link rewriting). No source edits.
- **Phase 1 — Archive bulk copy** (1 agent, mechanical): `cp -R` all ARCHIVE content. Regroup `specs/` by date-prefix into `yyyymm/`. **No link rewriting, no content edits** (frozen; `ai-docs/` survives so internal links stay valid).
- **Phase 2 — Leaf live docs** (2 agents parallel, independent): (2a) `deployments/` + `research/model-router/` + `conventions/` (with cleanups) **and author the new `docs/conventions/testing.md` from the live test suite** (verify every snippet/`file_path:line` against current code; emphasize the batched-`test.step` E2E philosophy); (2b) `guides/` (14 + app-bindings, with Next.js/API/version cleanups). During this phase, if either agent hits a trashed concept with no current-truth home, flag it (per the concept-rescue rule) instead of dropping it.
- **Phase 3 — Architecture** (1 agent): 7 arch files + `security.md`; heavy-rewrite `system-overview.md`; rewrite internal links to final Phase-2 dest paths and to canonical `crates/<crate>/CLAUDE.md`.
- **Phase 4 — Notes** (1 agent): copy 6 notes from scattered sources; add a status line to each; rewrite links.
- **Phase 5 — Indexes** (1 agent, LAST): author `docs/CLAUDE.md` + 5 sub-indexes; every link targets a file that already landed.
- **Phase 6 — External edits + verification** (1 agent): see below.

### Link-rewriting rules
1. **Doc-to-doc inside migrated LIVE files**: `ai-docs/<old>` → relative path to final `docs/<new>` via the Phase-0 manifest.
2. **Links to crate docs**: → `crates/<crate>/CLAUDE.md` (repo-root-relative), never a copy.
3. **Links inside ARCHIVED files**: leave untouched (valid because `ai-docs/` persists).
4. Unresolvable links (target was trashed): convert to plain text with `(archived)` note, or point at the archive copy.

---

## Cross-cutting external edits (Phase 6)

| File | Change |
|---|---|
| `CLAUDE.md:8` | `ai-docs/func-specs/security/security.md` → `docs/architecture/security.md`; add `docs/CLAUDE.md` to header |
| `TECHDEBT.md:4` | → `docs/archive/claude-plans/202603/20260303-multi-tenant/reviews/summary.md` |
| `devops/README.md:97` | → `docs/conventions/cuda-dockerfile-optimizations.md` |
| `.claude/settings.json:6` | `plansDirectory` → `docs/claude-plans/202606/` |
| `.claude/commands/{plan-md,task-md,next-iter-plan,review}.md` | repoint plan/spec authoring paths from `ai-docs/` → `docs/claude-plans/202606/` and `docs/specs/` |
| `.kiro/` | grep for unique salvageable content (steering is Next.js-era stale — likely nothing); migrate anything useful into `docs/`, then **`git rm -r .kiro/`** |
| `.cursor/plans/*` | skip (ephemeral) |

---

## Verification (Phase 6, read-only)

1. **Presence**: every manifest dest path exists; counts match (guides≈15, architecture=8, conventions≈8, notes=6, deployments=2, research model-router tree complete).
2. **Archive reconcile**: `find docs/archive/specs -name '*.md' | wc -l` vs source specs minus `pending/`; claude-plans minus `cowork/`+`deferred/`. Spot-check `docs/archive/specs/202509/20250902-ai-api-models/`.
3. **Internal link checker** (read-only script): for every `.md` under `docs/` except `docs/archive/`, resolve each markdown link + `docs/...`/`crates/...` path; fail on first unresolved.
4. **No stray ai-docs refs in live docs**: `grep -rn 'ai-docs/' docs/ --include='*.md' | grep -v '/archive/'` is empty.
5. **Security ref**: `docs/architecture/security.md` exists; `CLAUDE.md` no longer contains `ai-docs/func-specs`.
6. **External sweep**: re-run `grep -rln 'ai-docs/'` over live tooling (CLAUDE.md, TECHDEBT.md, devops/, .claude/); confirm zero dangling pointers (only `ai-docs/` content itself and `.cursor/` may remain).
7. **Hub walk**: load `docs/CLAUDE.md`, follow every link one hop, confirm each sub-index + its links resolve.
8. **`.kiro/` gone**: `test ! -d .kiro`.
9. **Testing doc integrity**: `docs/conventions/testing.md` exists; every `file_path:line` it quotes resolves and the quoted code still matches the live file (spot-check the `mcps-crud.spec.mjs` batched example); every linked canonical doc (`tests-js/E2E.md`, `crates/bodhi/src/TESTING.md`, `crates/routes_app/TESTING.md`, `crates/services/src/test_utils/CLAUDE.md`) exists.

Optionally run `make format` on touched md if a formatter applies; do not run backend/E2E gates (docs-only change) unless the user wants the full gate per [[feedback_run_all_gate_checks]].

---

## Critical files
- `CLAUDE.md`, `MDFILES.md`, `TECHDEBT.md` (root)
- `.claude/settings.json`, `.claude/commands/{plan-md,task-md,next-iter-plan,review}.md`
- `devops/README.md`
- `ai-docs/func-specs/security/security.md` (→ `docs/architecture/security.md`)
- `ai-docs/guides/bodhi-app/*`, `ai-docs/07-research/model-router/**` (largest live migrations)
- Testing sources of truth (for authoring `docs/conventions/testing.md`): `crates/lib_bodhiserver/tests-js/E2E.md`, `crates/lib_bodhiserver/tests-js/CLAUDE.md`, `crates/lib_bodhiserver/tests-js/specs/mcps/mcps-crud.spec.mjs` (batched-`test.step` exemplar), `crates/bodhi/src/TESTING.md`, `crates/routes_app/TESTING.md`, `crates/services/src/test_utils/CLAUDE.md`, `.claude/skills/{test-services,test-routes-app}/`
