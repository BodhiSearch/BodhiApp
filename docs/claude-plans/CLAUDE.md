# docs/claude-plans — CLAUDE.md

This folder holds Claude Code plans, kickoff prompts, retrospectives, and planning references, organized into monthly subfolders (`yyyymm/`, e.g. `202606/`).

## index.md convention (progressive disclosure)

Every level exposes an `index.md` describing only its immediate contents:

- `docs/claude-plans/index.md` — lists top-level files plus one entry per month folder. Month-folder entries only say the folder contains that month's plans and point to the folder's own `index.md`; they do NOT list the folder's files.
- `docs/claude-plans/yyyymm/index.md` — lists **all** files in that month folder, including files inside nested subfolders (e.g. `screen-v2/…`), as a flat list. Nested subfolders do NOT get their own `index.md`.

### Entry format

```markdown
- YYYY-MM-DD — [<relative-path>](<relative-path>) — 1-2 sentence summary.
```

- **Date** = the date the file was **added** (its git creation date; for a brand-new file, today's date). There is no modified-date tracking — never change the date when a file is edited. For a pre-existing file whose date is unknown, recover it with `git log --follow --diff-filter=A --format=%ad --date=short -- <file> | tail -1` (`--follow` matters: files have been moved/renamed).
- **Summary** = 1-2 sentences stating the actual feature/topic and the kind of doc (implementation plan, kickoff prompt, retrospective, reference, tech-debt list, …). Many filenames are auto-generated gibberish, so the summary must carry the meaning.
- Entries are sorted by date ascending (ties: path order).
- Only git-tracked files are indexed — skip gitignored files/folders and empty folders.

## Maintenance rules (MUST follow)

Whenever you **add** a file anywhere under `docs/claude-plans/`:

1. Add an entry (date = today, plus summary) to the `index.md` of the containing **month folder** — including files placed in nested subfolders like `202606/screen-v2/`.
2. If the file is at the top level of `docs/claude-plans/`, add the entry to `docs/claude-plans/index.md` instead.
3. If you create a **new month folder**, create its `index.md` (header + entries) and add a pointer entry for the folder in `docs/claude-plans/index.md`.

Whenever you **edit** a file in a way that changes what it covers (scope, status, conclusions):

- Update its summary in the corresponding `index.md`. Keep the original added date unchanged.

Whenever you **move or delete** a file, update the affected `index.md` entries accordingly (a moved file keeps its original added date).
