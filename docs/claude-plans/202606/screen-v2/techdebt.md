# UI V2 Migration — Tech Debt / Failing Tests

Tracked failing tests + deferred fixes surfaced during the migration. Update as items are resolved
(remove the entry) or as new ones appear in later batches.

## Failing / Deferred

### Chat `chat.spec.mjs:72` "multi-chat management and error handling" — Batch-0 shell regression
Confirmed (not env/streaming flakiness): the chat-history **delete buttons are unclickable** because
the new AppShell's left **resize handle** floats on top of them.

Root cause, measured live (1280×720, standalone):
- `.shell-resize.left` is a 16px-wide, full-height, **`z-index:40`** strip centered on the shell
  sidebar's 240px boundary → occupies `x≈232–248`, full height.
- The chat page is an unmigrated screen that renders its **own** `fixed`, **`z-10`** history sidebar
  (width 260px, starting at `x=0`) inside the shell. Its delete buttons sit at `x≈227–247`, directly
  under the higher-z resize handle. `document.elementFromPoint(deleteCenter)` returns
  `DIV.shell-resize left`, so Playwright's click never lands → 30s actionability timeout.
- `z-index` on the delete button does not help: the chat sidebar (`z-10`) and the handle (`z-40`)
  live in different stacking contexts, so the button can't rise above the handle from inside its own.

**Decision: defer.** Fix when chat migrates to screen v2 (the unmigrated chat sidebar overlapping the
shell sidebar is the real source of the collision; it goes away once chat adopts the shell chrome).
Candidate fixes when we get there: suppress the shell's left/right resize handles on `flush`
coexistence screens that render their own sidebar, or narrow the handle's pointer-capture area.
Fails identically in `standalone` + `multi_tenant`.

## Other deferred items
- Repo-wide ESLint: ~289 pre-existing errors in files untouched by the migration (import/order in
  `stores/initStores.ts`, `types/chat.ts`; a missing `react/display-name` rule in
  `tests/mocks/framer-motion.tsx`; etc.). Batch-0 files are lint-clean. Not addressed here.
- `make test.backend` requires Docker (PostgreSQL); pg-variant DB tests panic at fixture setup if
  Docker is down. Start Docker before the backend gate.
