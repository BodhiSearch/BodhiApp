# Fix path references after `getbodhi.app` → `getbodhi-app` folder rename

## Context

The website folder `getbodhi.app` was renamed on disk to `getbodhi-app` to avoid macOS Finder treating any `*.app`-suffixed directory as an Application bundle. The rename itself is done (confirmed: `getbodhi.app` no longer exists, `getbodhi-app` does). This plan covers everything else that needs to change so CI, local dev commands, and git tracking keep working — nothing more.

Three parallel Explore passes covered `.github/` (workflows + actions), all Makefiles, and the rest of the repo (root configs, scripts, docs, and the renamed folder's own self-references). The scope below is only the items confirmed to actually reference the folder as a **path**. Everything else — the live domain `getbodhi.app` (used in `id.getbodhi.app`, `api.getbodhi.app`, CNAME files, docs prose, etc.), the git release-tag prefix `getbodhi.app/v*`, and the website's own `package.json` name / README self-references — is confirmed unrelated or explicitly out of scope per your decisions below, and must NOT be touched.

**Decisions already made with you:**
- Keep the release-tag prefix as `getbodhi.app/v*` (23 tags already pushed under this prefix; `getbodhi-app/Makefile`'s version-bump logic finds "current version" by filtering tags on this exact string — renaming it would silently reset version continuity for zero benefit, since tags aren't folders and don't cause the Finder problem).
- Leave `getbodhi-app/package.json`'s `"name": "getbodhi.app"` field and `getbodhi-app/README.md`'s internal path prose as-is (cosmetic, nothing parses/depends on either).

## Critical finding: silent data-loss risk in `.gitignore`

`.gitignore:188` has:
```
.env
.env.*
...
!getbodhi.app/.env.release_urls
```
The blanket `.env.*` rule ignores everything, with a negation carve-out for the old path only. Since the negation doesn't match the new path, `getbodhi-app/.env.release_urls` is currently **silently gitignored** — `git add -A` will skip it entirely (git refuses to add ignored files without `-f`), so this file would quietly stop being tracked even though it sits on disk. This must be fixed before anything is committed, ideally in the same commit that stages the renamed folder, so `git add -A` picks it up correctly.

## Files to change

### 1. `.gitignore` (line 188)
```diff
- !getbodhi.app/.env.release_urls
+ !getbodhi-app/.env.release_urls
```

### 2. `Makefile` (line 105, inside the `format:` target)
```diff
- $(MAKE) -C getbodhi.app format
+ $(MAKE) -C getbodhi-app format
```

### 3. `Makefile.website.mk` (lines 18, 21, 24)
```diff
- $(MAKE) -C getbodhi.app update_releases
+ $(MAKE) -C getbodhi-app update_releases
- $(MAKE) -C getbodhi.app update_releases.check
+ $(MAKE) -C getbodhi-app update_releases.check
- $(MAKE) -C getbodhi.app release
+ $(MAKE) -C getbodhi-app release
```

### 4. `.github/workflows/deploy-website.yml` — path references only (11 lines, keep tag-related lines untouched)

Change these (folder path / cache key / artifact path / jq input / commit file list):
- Line 28: `working-directory: getbodhi.app` → `working-directory: getbodhi-app`
- Line 56: `cache-dependency-path: getbodhi.app/package-lock.json` → `getbodhi-app/package-lock.json`
- Line 67: `getbodhi.app/.next/cache` → `getbodhi-app/.next/cache`
- Line 68: both `hashFiles('getbodhi.app/package-lock.json')` and `hashFiles('getbodhi.app/**.[jt]s', 'getbodhi.app/**.[jt]sx')` → `getbodhi-app/...`
- Line 70: `hashFiles('getbodhi.app/package-lock.json')` → `getbodhi-app/package-lock.json`
- Line 110: `path: ./getbodhi.app/out` → `./getbodhi-app/out`
- Lines 139, 140, 143, 144: `jq -r '...' getbodhi.app/public/releases.json` → `getbodhi-app/public/releases.json` (all four)
- Line 185: `working-directory: getbodhi.app` → `working-directory: getbodhi-app`
- Line 227: `files: "getbodhi.app/package.json getbodhi.app/package-lock.json getbodhi.app/public/version.json"` → the three `getbodhi-app/...` equivalents

**Do NOT change** (release-tag convention, kept per your decision): line 6 (`tags: - 'getbodhi.app/v*'`), line 10 (dispatch input description), line 48-49 (validate-release-tag pattern/description), line 79 and line 200 (`VERSION="${...#getbodhi.app/v}"`), line 224 (commit message `chore(getbodhi.app): ...`). These are git-tag strings and a commit-message scope label, not filesystem paths — they stay as `getbodhi.app` by design.

Note line 201, `node ../scripts/increment_version.js`, is a relative path from the job's working directory — it needs no change since it resolves correctly regardless of the folder's name.

## Nothing else needs to change

Confirmed by exhaustive grep across `.github/`, all `Makefile*`, and the rest of the repo tree:
- `getbodhi-app/Makefile` — uses only relative paths (`../Makefile.release.mk`, `../scripts/increment_version.js`) and the (kept) tag convention; no internal fix needed.
- `.github/actions/*` (13 composite actions) — all generic, receive paths as inputs from the caller workflow; none hardcode the folder name.
- No `on.push.paths` / `on.pull_request.paths` filters anywhere reference this folder (the only trigger involving it is the tag-based trigger on line 6, unaffected by the rename).
- `scripts/trigger_workflow.js` — tag-prefix string only, kept per your decision.
- No root `package.json`, `pnpm-workspace.yaml`, `turbo.json`, `netlify.toml`, `vercel.json`, or `wrangler.toml` exist in this repo to update.
- `docs/CLAUDE.md`, `crates/CLAUDE.md` — no references.
- `.gitmodules` — `getbodhi.app` was never a submodule.
- `docs/archive/**` and `docs/claude-plans/**` contain old path references (e.g. `docs/archive/specs/202510/20251009-getbodhi-app/*`) but are established as frozen historical record elsewhere in this repo's plan conventions — left untouched.
- Every other hit for the string `getbodhi.app` across the repo (auth realms `id.getbodhi.app`/`main-id.getbodhi.app`, `api.getbodhi.app`, CNAME files, OpenAPI contact URLs, docs prose, `getbodhi-app/scripts/get-version.js`'s live URL fetch) is the actual DNS domain name, not a folder path, and must stay as `getbodhi.app`.

## Verification

1. **Gitignore fix, concretely proven**: before editing, run `git check-ignore -v getbodhi-app/.env.release_urls` — expect it to print a match against `.env.*` (proving the bug). After editing `.gitignore`, run it again — expect no output (file no longer ignored).
2. **Path existence sanity check**: `ls getbodhi-app/package-lock.json getbodhi-app/public/releases.json getbodhi-app/out 2>&1` (the last may not exist until a build runs — that's fine, it's a build output) to confirm the renamed paths used in the workflow actually exist.
3. **Makefile dry-run** (non-destructive): `make -n format` and `make -n website.update-releases` — confirm the printed (not executed) commands show `-C getbodhi-app ...` and don't error on a missing directory.
4. **YAML sanity**: reload `.github/workflows/deploy-website.yml` and diff against the list above to confirm only the 11 intended lines changed and all tag-convention lines (6, 10, 48, 49, 79, 200, 224) are untouched.
5. **Full-repo re-grep**: `grep -rn "getbodhi\.app" --include="*.yml" --include="Makefile*" .github Makefile Makefile.website.mk .gitignore` — after the fix, the only remaining hits should be the tag-convention lines in `deploy-website.yml` (and the CLAUDE.md / publish-docker*.yml domain references, which are out of scope and correctly unchanged).
6. **Commit as one unit**: stage `.gitignore` together with the `getbodhi-app/` folder and the Makefile/workflow edits in a single commit so `git add -A` correctly tracks `getbodhi-app/.env.release_urls` from the start (avoids a window where the fix and the file addition are split across commits).
7. Full CI validation of `deploy-website.yml` can only happen by actually running it (tag push or `workflow_dispatch`), which is outside the scope of local verification — call this out to the user as a follow-up: after merging, do a `workflow_dispatch` dry run (or wait for the next real release tag) to confirm the Pages deploy job succeeds end-to-end.
