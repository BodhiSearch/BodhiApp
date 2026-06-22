# Explore · Local Models — Downloads Panel (Batch 3-6 Phase 6)

## Context

The V2 Explore · Local Models screen lets users discover HuggingFace GGUF repos and start
pulls, but there is no in-screen view of download progress. Today downloads live on a separate
legacy page (`/ui/models/files/pull/`), and a second legacy page (`/ui/models/files/`) browses
already-downloaded files — both predate the unified V2 shell.

This change adds a **Downloads panel** to the right rail of the Local Models screen (per the
provided design: DOWNLOADING / QUEUED / FAILED / COMPLETED sections with progress, retry, and
archive), wired to a "Downloads" toolbar button that opens the rail and cache-busts the list.
It also adds the two backend capabilities the design needs but the API lacks today — **archive**
(dismiss a terminal download so it leaves the list and the API response) and **retry** (re-run a
failed download, which resumes for free via hf-hub's `.sync.part`). Finally, it **deletes both
legacy pages**, whose features are already covered: downloads → this panel; local-file browsing →
the unified `/ui/models` page (`Local File` filter + `ModelPreviewModal`).

### Ground-truth findings (from exploration)

- **Status enum** (`crates/services/src/models/model_objs.rs`): only `Pending | Completed | Error`.
  No `Downloading`/`Queued`/`Archived`. UI sections are **derived**:
  - DOWNLOADING = `Pending` AND `started_at.is_some()` (set on first progress sync)
  - QUEUED = `Pending` AND `started_at.is_none()`
  - FAILED = `Error`; COMPLETED = `Completed`
- **Downloader** (`crates/services/src/models/hub_service.rs`): hf-hub 0.4.3
  `repo.download_with_progress(filename, progress)` — opaque call. It writes `<blob>.sync.part`
  and **resumes via HTTP Range** on re-invocation; the partial survives on error. So **retry =
  re-spawn the same pull = resume**, no intermediate-path storage needed. **No cancellation hook**
  → no pause, no active-download cancel (matches user decision).
- **Create handler** (`crates/routes_app/src/models/files/routes_files_pull.rs`): `POST` returns
  the existing row (200) without re-spawning when `find_by_repo_filename` hits — so a failed row
  cannot be retried by re-POSTing; we need a dedicated retry endpoint.
- **List** (`crates/services/src/models/download_repository.rs:118`): tenant-only filter,
  `ORDER BY updated_at DESC`. Needs an `archived_at IS NULL` filter.
- **Legacy-page deletion is safe**: unified `/ui/models` has `Local File` facet
  (`ModelSidebarFacets.tsx`), `ModelPreviewModal` (still used by `models/index.tsx`), and the same
  metadata-refresh hook. `useListModelFiles` **stays** (used by `AliasForm.tsx`).
- **E2E coverage**: `model-metadata.spec.mjs` Test 1 (line ~160, "from models page") already
  covers preview/metadata-refresh on the unified page; Test 2 (line ~213, "from modelfiles page")
  is the duplicate bound to the deleted route → remove Test 2 (coverage preserved by Test 1).

## Decisions (confirmed with user)

- **Archive**: backend `archived_at` column + endpoint + list filter (durable, excluded from API).
- **Retry**: dedicated `POST .../{id}/retry` endpoint (reset to `Pending`, re-spawn → resumes).
- **Cancel/pause active**: NOT supported by the client → not built. `×` only on terminal/queued.
- **Page removal**: delete BOTH `/models/files/pull/` and `/models/files/` (features migrated).

---

## Phase 1 — Backend: archive + retry (services, upstream-first)

**Goal:** add `archived_at`, the list filter, and `archive`/`retry` service methods; verify no
regression in existing pull tests. Defer route exposure to Phase 2.

- **Migration** — new `crates/services/src/db/sea_migrations/m20250101_0000NN_download_archived_at.rs`
  (next free index; register in the migrator alongside `m20250101_000001_download_requests.rs`).
  Add nullable `archived_at` timestamp to `download_requests`.
- **Entity** — `crates/services/src/models/download_request_entity.rs`: add
  `pub archived_at: Option<DateTime<Utc>>`. Update the SeaORM `ActiveModel` mapping in
  `download_repository.rs` create/update (set on both create=`NotSet`/`None` and update — see
  `[[feedback_activemodel_update_default_trap]]`: set the column explicitly in update, don't rely
  on `..Default::default()`).
- **Response DTO** — `download_service.rs` `DownloadRequest`: add `archived_at` (so it round-trips;
  the list never returns archived rows but `archive`/`get` responses may carry it).
- **Repository** (`download_repository.rs`):
  - `list_download_requests`: add `.filter(Column::ArchivedAt.is_null())` to BOTH the `count` and
    the page query.
  - add `archive_download_request(tenant_id, id, now)` (set `archived_at`) and reuse existing
    update path for retry's status reset.
- **Service** (`download_service.rs` + `auth_scoped_downloads.rs`): add to the `DownloadService`
  trait (and `#[automock]` mock + `AuthScopedDownloadService`):
  - `archive(tenant_id, id)` — loads row; if `status == Pending && started_at.is_some()`
    (actively downloading) return a typed `DownloadServiceError::CannotArchiveActive` (BadRequest);
    else set `archived_at`.
  - `reset_for_retry(tenant_id, id)` — require `status == Error` else
    `DownloadServiceError::NotRetryable` (BadRequest); set `status=Pending`, `error=None`,
    `started_at=None`, bump `updated_at`; return the updated entity (the route re-spawns).
  - New error variants in `DownloadServiceError` with `#[derive(ErrorMeta)]` codes
    `download_service_error-cannot_archive_active` / `-not_retryable` (see
    `[[feedback_errmeta_gotchas]]`).
- **Tests** (services, `test-services` skill): add cases for archive (terminal ok, active rejected),
  retry reset (error→pending ok, non-error rejected), and that `list` excludes archived rows.
  Run `#[values("sqlite","postgres")]`. Reuse `TestDbService`/`SeaTestContext`/`FrozenTimeService`.

**Gate:** `cargo test -p services --lib` green (capture once — `[[feedback_capture_long_commands]]`).

## Phase 2 — Backend: archive + retry routes (routes_app)

**Goal:** expose the two endpoints; regenerate OpenAPI + ts-client.

- **Handlers** in `crates/routes_app/src/models/files/routes_files_pull.rs`:
  - `POST {ENDPOINT_MODELS_FILES_PULL}/{id}/archive` → `models_pull_archive`: calls
    `auth_scope.downloads().archive(id)`, returns the updated `DownloadRequest` (200).
  - `POST {ENDPOINT_MODELS_FILES_PULL}/{id}/retry` → `models_pull_retry`: calls
    `reset_for_retry(id)`, then **re-spawns** the same `spawn(execute_pull_by_repo_file(...))`
    block already used by `models_pull_create` (extract that spawn into a small private helper
    `spawn_pull(app_service, repo, filename, request_id, tenant_id)` and call it from both create
    and retry). Returns the reset `DownloadRequest` (200).
  - Reuse `AuthScope`, `power_user` security, `BodhiErrorResponse` return type.
- **Register**: routes in `crates/routes_app/src/routes.rs` (next to the existing pull routes),
  and add `__path_models_pull_archive` / `__path_models_pull_retry` + any new schema to
  `crates/routes_app/src/shared/openapi.rs` (follow the OpenAPI checklist in routes_app/CLAUDE.md).
- **Tests** (routes_app, `test-routes-app` skill, `tower::oneshot`): archive terminal → 200 +
  excluded from subsequent list; archive active → 400 (`.code()` assert); retry error → 200 +
  status back to pending; retry non-error → 400. Mock `DownloadService` via `AppServiceStub`.
- **Regen**: `cargo run --package xtask openapi && make build.ts-client`. New `@bodhiapp/ts-client`
  types (`DownloadRequest.archived_at`, endpoints) become available.

**Gate:** `cargo test -p routes_app -- files_pull openapi` green; `make ci.ts-client-check`.

## Phase 3 — Frontend hooks (cache, archive, retry)

**Goal:** extend `crates/bodhi/src/hooks/models/useDownloads.ts` + `constants.ts`.

- `downloadKeys` (constants.ts): already has `all/lists/list`. Add endpoint constants
  `ENDPOINT_MODEL_FILES_PULL_ARCHIVE`/`_RETRY` (`.../pull/{id}/archive`, `.../pull/{id}/retry`).
- `useListDownloads`: add `staleTime: 60_000` (the 1-minute cache) alongside the existing
  `refetchInterval` (polling and staleness are independent — polling still drives live progress;
  staleTime governs background-focus refetch and the "fresh enough" window).
- Add `useArchiveDownload()` and `useRetryDownload()` mutations (mirror `usePullModel`: POST via
  `useMutationQuery`, `onSuccess` → `queryClient.invalidateQueries({ queryKey: downloadKeys.all })`).
- Add a `useDownloadsRefresh()` helper (or expose `invalidate`) the panel calls on Downloads-button
  click to **cache-bust**: `queryClient.invalidateQueries({ queryKey: downloadKeys.all })`.
- **Tests** (`useModels.test.ts` downloads section): archive/retry call the right endpoint +
  invalidate; staleTime present. Reuse MSW handlers (Phase 5).

## Phase 4 — Frontend: Downloads panel + rail coexistence (thin vertical slice)

**Goal:** render the panel in the rail behind the Downloads button; verify in Chrome.
Files under `crates/bodhi/src/routes/models/explore/local/-components/`.

- **Rail mode state** in `LocalDiscoveryScreen.tsx`: introduce
  `railMode: 'model' | 'downloads' | null`. Selecting a model sets `'model'`; the Downloads button
  sets `'downloads'` (and cache-busts). Publish `rail`/`railHeader` derived from `railMode`
  (downloads content takes precedence when `'downloads'`). Publishing non-null `rail` auto-opens
  the rail (AppShell). Closing the rail header resets `railMode=null`. Keep both memoized.
- **Downloads button**: add via `useShellChrome({ headerActions })` — a `ShellIcon name="download"`
  button with a **count badge** = number of active (DOWNLOADING+QUEUED) items from `useListDownloads`.
  `onClick`: `setRailMode('downloads')` + `useDownloadsRefresh()`.
- **New `DownloadsPanel.tsx`** (+ `DownloadsPanelHeader.tsx`): consumes `useListDownloads(1, 100,
  { enablePolling: hasActive })` where `hasActive` = any DOWNLOADING/QUEUED. Group rows by derived
  section (helper `sectionOf(d)` using the `started_at`/`status` derivation). Render per design:
  - DOWNLOADING: progress bar (`downloaded_bytes/total_bytes`), `compact()` GB/GB, MB/s + ETA
    (derive from successive polls or omit if not trivially available — show GB/GB + % at minimum).
    No `×`.
  - QUEUED: "Waiting…" + size; `×` (archive).
  - FAILED: error text + size; `↻` (retry → `useRetryDownload`) and `×` (archive).
  - COMPLETED: check + relative time (reuse existing date formatting util if present) + size; `×`.
  - `×` calls `useArchiveDownload`; row disappears on invalidation.
- **CSS**: `local-discovery.css`, `ld-dl-*` classes matching the `ld-*` convention.
- **Keyboard handoff**: the rail is outside `useListKeyNav`'s scope (it ignores `.shell-rail`), so
  arrows don't bubble. Add an `onKeyDown` on the panel root that, on **ArrowDown only**, moves
  focus to the main list (focus the `.l-listrow.active .l-rowlink` or first row) — ArrowUp does
  nothing (intentional asymmetry, per design).
- **Verify in Chrome** (`make app.run.live`, claude-in-chrome): start a pull, open Downloads, see
  it under DOWNLOADING; let it finish → COMPLETED; archive → disappears; force a failure → FAILED
  with retry; ArrowDown from panel jumps to the table.

## Phase 5 — Tests: vitest component + MSW + E2E

- **MSW** (`crates/bodhi/src/test-utils/msw-v2/handlers/modelfiles.ts`): add
  `mockModelPullArchive`/`mockModelPullRetry` (+ error variants); extend defaults to include a
  `started_at`-set pending (downloading), a `started_at`-null pending (queued), an error, and a
  completed row so all four sections render.
- **Component test** `explore/local/index.v2.test.tsx` (or a new `downloads.test.tsx`): Downloads
  button opens the panel + cache-busts (request count); four sections render from derived statuses;
  archive removes a row and calls the endpoint; retry calls the endpoint; badge counts active only.
- **E2E** — grow the Local Models spec (one spec, many `test.step`; black-box UI only — see
  `[[feedback_blackbox_e2e]]`, `[[feedback_e2e_reduced_motion_for_view_transitions]]`): open
  Downloads, assert sections, archive a completed item (gone after reload), retry a failed item.
  Throw in `beforeAll` if prerequisites missing — never `test.skip` (`[[feedback_no_skip_for_missing_env]]`).
  Note GATE-B: backend changed, so run against the **rebuilt** dev-server
  (`[[feedback_gateb_rebuild_binary_for_backend_batches]]`).

## Phase 6 — Delete legacy pages + cleanup

- **Delete** (entire dirs): `crates/bodhi/src/routes/models/files/pull/` (index, test, `-components/PullForm.tsx` + test)
  and `crates/bodhi/src/routes/models/files/` (index + test). `routeTree.gen.ts` regenerates on build.
- **Nav** (`crates/bodhi/src/hooks/navigation/useNavigation.tsx`): remove "Model Files" and
  "Model Downloads" items (lines ~72–82).
- **E2E** (`crates/lib_bodhiserver/tests-js/specs/models/model-metadata.spec.mjs`): remove Test 2
  ("from modelfiles page", ~213–271). Coverage preserved by Test 1 ("from models page").
- **Keep**: `useListModelFiles` + `ENDPOINT_MODEL_FILES` (used by `AliasForm.tsx`); `usePullModel`
  + `useListDownloads` (now used by the panel); the `download-models` setup step (independent).
- **Verify**: `cd crates/bodhi && npm run lint && npm test`; grep for dangling references to the
  deleted routes.

---

## Verification (end-to-end)

1. **Rust**: `cargo test -p services --lib` then `cargo test -p routes_app` (capture once each).
   Then `make test.backend`.
2. **Contract**: `cargo run --package xtask openapi && make build.ts-client && make ci.ts-client-check`.
3. **UI unit**: `cd crates/bodhi && npm test` (panel + hooks + MSW).
4. **Chrome smoke** (`make app.run.live`): pull → Downloads panel shows DOWNLOADING → COMPLETED;
   archive removes + survives reload (confirms API exclusion); fail → FAILED → retry resumes;
   badge counts active; ArrowDown handoff to table works; ArrowUp does not return.
5. **E2E** (rebuilt dev-server): `make build.dev-server` then `make test.e2e` from
   `crates/lib_bodhiserver/tests-js` — Local Models downloads steps + the trimmed metadata spec.
6. **Docs**: update `crates/services/PACKAGE.md` (archive/retry), `crates/routes_app/PACKAGE.md`
   (new routes), and memory `[[project_screen_v2_batch3_6_local_discovery]]`.

## Commit strategy

Feature rollout → commit per phase (`[[feedback_layered_refactors]]`), each after its gate checks
pass (`[[feedback_run_all_gate_checks]]`). Trunk-based: commit straight to `main`, rebase before push.
