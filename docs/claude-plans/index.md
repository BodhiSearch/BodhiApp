# docs/claude-plans — Index

Claude Code plans, kickoff prompts, and retrospectives, organized into monthly folders. Folder entries below only note what the folder holds — open the folder's own `index.md` for its file listing (progressive disclosure). See [CLAUDE.md](CLAUDE.md) for the maintenance rules.

## Month folders

- 2026-06-12 — [202606/](202606/index.md) — Plan files for June 2026: Screen V2 UI migration batches, Models/Explore pages, MCP screens/playground, API tokens. See [202606/index.md](202606/index.md) for the file listing.
- 2026-07-01 — [202607/](202607/index.md) — Plan files for July 2026 and later one-offs: OAuth single-step flow and assorted fix/implementation briefs. See [202607/index.md](202607/index.md) for the file listing.

## Files

- 2026-06-30 — [techdebt.md](techdebt.md) — Running tech-debt backlog of deferred, intentionally-scoped-out follow-ups (mostly from the tokens/grants screen-v2 review and its remediation), covering things like conditional model/MCP fetching, an error-variant rename, access-request route relocation, a middleware double-parse inefficiency, missing grant-enforcement tests, and small E2E fixture/dead-code cleanups.
- 2026-07-05 — [nicetohave.md](nicetohave.md) — Backlog of deferred, non-blocking hardening/UX items captured during recent feature work: visual granted-vs-new pills and role cards for the exchange-review UI, and invalidating/marking as Exchanged the superseded token in the access-request exchange (upgrade) flow.
