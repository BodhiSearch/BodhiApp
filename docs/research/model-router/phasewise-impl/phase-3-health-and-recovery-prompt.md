# Kickoff — Phase 3: Health-Aware Skipping & Automatic Recovery

You are picking up the **model-router** feature in BodhiApp. Phases 1 and 2 are already
implemented and merged. Your job this session is **Phase 3**. Do not write production code
yet — first understand, then propose a plan.

## The problem to solve

Today the router re-evaluates its whole target chain from the top on every single request.
If a provider is down or has burned through its free quota, every request still pays the cost
of trying it first and failing before falling through. That's wasteful and slow, and it never
"settles" onto a healthy provider.

Phase 3 should make the router **remember** that a target recently failed and **stop sending it
traffic for a while**, so a known-bad or quota-exhausted provider gets skipped on later requests.
Then, once enough time has passed, the router should **quietly try it again** on a real request,
and if it works, **go back to using it** — favouring the user's preferred (primary) provider as
soon as it recovers. All of this should happen on its own, driven by ordinary chat traffic, with
**no background pingers or health-check loops**.

A few behaviours matter a lot and are easy to get subtly wrong — explore how each should feel:
- A provider that's used by **more than one** router. If one router discovers it's down, should
  the others keep hammering it, or benefit from what's already known?
- What happens when **everything** the user enabled is currently in the doghouse — does the user
  get an instant error, or should the router still try its best?
- A provider the user **explicitly turned off** vs. one that's merely cooling down — these are
  different intents. Make sure the difference survives.
- When a provider asks you to back off (it tells you *how long* to wait), should the router listen?

There are also a few **knobs** that already exist in the data model but aren't surfaced anywhere
the user can reach. Part of this phase is letting the user actually see and set them, with sensible
defaults, in the router's create/edit screen.

## How to approach this (this is the important part)

1. **Read the phase brief and the handoff notes, then go read the code.**
   - The functional spec for this phase: `phase-3-health-and-recovery.md` (in this folder). It tells
     you *what the feature must do and how it will be tested* — treat the acceptance gates as the
     definition of done.
   - The Phase-2 handoff: `phase-2-in-request-fallback-notes.md` (same folder). It tells you what the
     previous phase actually shipped and where the seams are. **Trust the code over the notes** if they
     ever disagree, and call out the disagreement.
   - Background design thinking (read as *guidance*, not gospel): the implementation proposal and the
     consolidated research in the parent `model-router/` folder. They describe an intended design. **Your
     job is to validate that intended design against the current codebase and flag anything that has
     drifted, is already different, or no longer makes sense.** Where the docs and the code disagree, the
     code wins — but tell us about it.

2. **Map the current behaviour before changing it.** Trace a chat request to a model-router from the
   HTTP entry point through to where a target is chosen and forwarded. Find where a failure is detected
   and where the fall-through decision is made. Understand what state (if any) survives between requests
   today, and what does not. Look for existing patterns in this codebase for *in-memory, process-local,
   shared-across-requests* state and for *injected, controllable time* — you should reuse the house style,
   not invent a new one.

3. **Pin down the open questions and ask them.** Some decisions genuinely change the design — surface
   them to the user with the `AskUserQuestion` tool rather than guessing. Likely candidates: how a target's
   "identity" should be defined for the purpose of remembering its health (so that sharing across routers
   works the way the user wants, while keeping tenants isolated); whether a *structural* misconfiguration
   should be treated like a transient outage or not; what defaults the new UI knobs should carry; and how
   the user should be able to *see* which provider actually served a response (the plumbing for this may
   already exist — check).

4. **Insist on a controllable clock.** Anything time-based (cooldowns expiring, recovery) has to be
   testable without real waiting. Find how the rest of the codebase makes time deterministic in tests and
   plan to use exactly that. Tests that `sleep` are not acceptable.

5. **Plan test-first, at every layer this touches.** This phase reaches from the routing core up through
   configuration persistence and into the UI. The brief's acceptance gates span service/unit (with a fake
   clock), integration, a frontend component, and a black-box Playwright E2E. Your plan must say, concretely,
   what each test asserts and where it lives. Black-box E2E means UI interactions only — no reaching into the
   server's internals from the test.

6. **Respect what Phases 1–2 already guarantee.** Failure classification, the verbatim-exhaustion behaviour,
   "disabled targets are never tried", "failover happens before the first byte", and the observability headers
   are all settled. Phase 3 should change *which targets are eligible* and *what is remembered between
   requests* — not how a single forward is classified or returned. If you find yourself wanting to change
   Phase-2 behaviour, that's a flag to raise, not a thing to silently do.

## What to produce

A written implementation plan for Phase 3 that:
- States the current behaviour you found (with the file paths that prove it), and any drift between the
  design docs and the code.
- Lists the decisions you need from the user (via `AskUserQuestion`) before coding, with your recommended
  default for each and why.
- Describes the change in terms of behaviour and the seams it touches — upstream-to-downstream, following
  the project's layered methodology — naming the real functions/types/files involved.
- Enumerates the gating tests per layer, mapped to the brief's acceptance gates, and confirms a controllable
  clock is used wherever time matters.
- Calls out anything you're deferring to Phase 4 (the router *test/probe* capability is explicitly a later
  phase — don't build it here) and anything you think the future-phase specs should be updated to reflect.

Then stop and present the plan for approval. Don't start editing code until the plan and the open questions
are settled with the user.
