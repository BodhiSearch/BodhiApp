# Plan: `getbodhi.app/tools/free-billion-tokens-bodhi-app/` → design-partner interest form

## Context

We're launching a tool built on BodhiApp: a **model router preconfigured with the free-tier LLM APIs** that providers have recently opened up, so users can reach those free APIs through Bodhi App. We're recruiting **design partners**, and the interest list is a Tally form at `https://tally.so/r/lbkbov`.

We want to hand out a branded, on-domain URL rather than a raw `tally.so` link:

```
https://getbodhi.app/tools/free-billion-tokens-bodhi-app/  →  https://tally.so/r/lbkbov?utm_…
```

The redirect is **temporary** — a real marketing page will eventually live at this path — so nothing may be permanently cached or indexed against it.

### Why this isn't an HTTP 302

The original ask was a 302. That is **not achievable without an infrastructure change**, and we've decided not to make one now.

`getbodhi.app` is a **Next.js 14 App Router site built with `output: 'export'`** (`getbodhi.app/next.config.mjs`), deployed as a static artifact to **GitHub Pages** (`.github/workflows/deploy-website.yml`; custom domain via `getbodhi.app/public/CNAME`). GitHub Pages is a plain file host — it serves files with 200 or 404 and honours no redirect-rule file (`_redirects`, `_headers`, `.htaccess` are all ignored). Next's own `redirects()` is **silently dropped** under `output: 'export'`.

A true 302 was reachable only via Cloudflare: the domain's nameservers are already Cloudflare (`fay`/`sterling.ns.cloudflare.com`) but the proxy is **off** — apex A records point straight at GitHub Pages IPs (`185.199.108–111.153`) and responses carry `server: GitHub.com` with no `cf-ray`. Turning the proxy on would have unlocked a Redirect Rule. **We've explicitly deferred that** — it changes the traffic path for the entire site (and carries a real site-wide redirect-loop hazard if the SSL mode isn't Full/strict), which is disproportionate for one campaign URL.

### Decisions taken during planning

- **Static stub page returning HTTP 200**, redirecting client-side. Visually equivalent for users; not a 302.
- **Show a visible "Redirecting…" body** so the user knows what's happening — not an invisible instant bounce.
- **One slug only:** `free-billion-tokens-bodhi-app` (the longer `…-model-router-…` variant from the initial brief is dropped).
- **No wildcard matching.** Static export cannot express one; dynamic routes require `generateStaticParams()` to enumerate paths at build time.
- **OpenGraph tags included** so shared links unfurl as a Bodhi card.
- **No analytics SDK.** The site has none today (no PostHog, GA, or Plausible anywhere in `getbodhi.app/`). Attribution comes from UTM params captured by Tally.
- **Redirect only** — no scaffolding for the future real page.

## Implementation

### The one file to add

**`getbodhi.app/public/tools/free-billion-tokens-bodhi-app/index.html`**

A hand-written, fully self-contained HTML file. This follows the existing precedent of **`getbodhi.app/public/privacy-extension.html`** — a raw HTML file dropped in `public/` and served verbatim, bypassing the App Router entirely.

Why `public/` rather than a Next route (`src/app/tools/…/page.tsx`):

- Everything in `public/` is copied verbatim into `out/`, preserving directory structure, so `public/tools/<slug>/index.html` → `out/tools/<slug>/index.html` → served at `/tools/<slug>/`. This matches the site's `trailingSlash: true` convention, and GitHub Pages 301s the no-slash form to the slash form automatically.
- Next 14's Metadata API has **no `http-equiv` support**, so a meta-refresh fallback can't be expressed through it cleanly.
- `redirect()` from `next/navigation` throws under static export.
- The file needs no Tailwind, no build step, and no framework involvement — and is trivially deleted later.

### File contents

Structure and the reasoning behind each part:

**`<head>`**

- `<meta name="robots" content="noindex, follow">` — **important.** Google treats a `0`-second meta refresh as equivalent to a *permanent* redirect for indexing purposes, which is the opposite of the temporary semantics we want. `noindex` keeps the stub out of the index entirely, so nothing is cached against the URL. Removed when the real page ships.
- **OpenGraph + Twitter card tags** — `og:type`, `og:url` (the canonical getbodhi.app URL), `og:title`, `og:description`, `og:image` → `https://getbodhi.app/bodhi-logo-1024.png`, plus `twitter:card`.
  - Using the **square 1024×1024 logo with `twitter:card=summary`** — it's on-brand and won't crop awkwardly. If you'd rather have a big product screenshot, `public/chat-ui.jpeg` is 1440×900 and works with `summary_large_image`; that's a two-line swap.
  - This is a genuine advantage of the 200-page approach: unfurlers read HTML without executing JS, so Twitter/LinkedIn/Slack show **our** card. A real 302 would have followed through and shown Tally's generic form card instead.
- `<meta http-equiv="refresh" content="3; url=…">` — the **no-JS fallback**, set slightly longer than the JS timer so JS wins on normal browsers.
- Inline `<style>` — self-contained, no external requests. Small centred card, system font stack, `prefers-color-scheme` for dark/light. The file sits outside the Next build so it gets no Tailwind.

**`<body>`**

A small branded panel: the Bodhi logo (`/bodhi-logo/bodhi-logo-240.svg`), a heading such as *"Taking you to the interest form…"*, one line of context naming the design-partner programme and that the form is hosted on Tally, and an always-visible **"Continue now →"** link carrying the same UTM-tagged URL. The manual link is what makes the page correct even if JS is disabled and the meta refresh is blocked.

**`<script>` at end of body**

```js
setTimeout(function () {
  location.replace("https://tally.so/r/lbkbov?utm_source=getbodhi.app&utm_medium=website&utm_campaign=free-billion-tokens-bodhi-app");
}, 1500);
```

Two deliberate choices:

- **`location.replace()`, not `location.href`.** `replace()` does not push a history entry, so pressing Back from the Tally form returns the user to wherever they came from. With `href`, Back would land them on the stub, which would immediately redirect them forward again — a trap.
- **1500 ms delay**, not `0`. You asked for the user to be aware they're being redirected; ~1.5 s is long enough to read the line and notice the branding, short enough not to feel broken. It's a one-number change if you want it snappier or slower.

### Target URL

```
https://tally.so/r/lbkbov?utm_source=getbodhi.app&utm_medium=website&utm_campaign=free-billion-tokens-bodhi-app
```

The identical string appears in three places in the file — the meta refresh, the `href` of the Continue link, and the `location.replace()` call. They must stay in sync.

### Tally-side setup (required for the UTM params to do anything)

Query params are inert unless the form captures them. In the **Tally form builder** for `lbkbov`, add three **Hidden fields** named exactly:

```
utm_source
utm_medium
utm_campaign
```

Tally populates hidden fields from matching URL query params and stores them with each submission, making responses filterable by campaign. Tally also reports form views and completion rate natively, which covers the funnel without any SDK on our side. **Without these fields the params are silently ignored** — the redirect still works, but you get no attribution.

This is a dashboard step I can't perform; it's on you (or grant access).

## Critical files

| Path | Change |
|---|---|
| `getbodhi.app/public/tools/free-billion-tokens-bodhi-app/index.html` | **New** — the entire implementation |

Nothing else changes. No edits to `next.config.mjs`, `src/app/**`, the header/footer nav (this is an unlisted campaign URL, not a site section), or any workflow.

## Deployment

The site deploys **only** on a git tag matching `getbodhi.app/v*` — pushing to `main` deploys nothing. Ship it with:

```bash
make website.release     # or: make -C getbodhi.app release
```

which computes the next version from remote tags, tags, and pushes, triggering `deploy-website.yml`.

**Be aware this is a full website release, not a targeted deploy.** The same workflow run also fires the `update-homebrew` job (a repository-dispatch to `BodhiSearch/homebrew-apps` driven by `public/releases.json`) and the `bump-dev-version` job, which auto-commits a `-dev` version bump and a regenerated `public/version.json` back to `main`. That's the normal release path, but worth knowing that a one-file campaign stub carries those side effects — so it's best folded in alongside the next website release rather than tagged in isolation, unless you need the URL live immediately.

## Verification

**Local, before tagging** — this is the important gate, since CI will not catch problems here (`next.config.mjs` sets both `typescript.ignoreBuildErrors: true` and `eslint.ignoreDuringBuilds: true`, and the website has no test suite at all):

```bash
cd getbodhi.app
npm run build

# The file must land in the export at the right path
ls -la out/tools/free-billion-tokens-bodhi-app/index.html

# All three occurrences of the UTM-tagged target must be present and identical
grep -o 'https://tally.so/r/lbkbov?[^"]*' out/tools/free-billion-tokens-bodhi-app/index.html

# noindex must be present
grep -o '<meta name="robots"[^>]*>' out/tools/free-billion-tokens-bodhi-app/index.html
```

Then serve the export and exercise it in a real browser (I'll drive this with the Chrome MCP tools):

```bash
npx serve out -p 3005
```

- Visit `http://localhost:3005/tools/free-billion-tokens-bodhi-app/` — confirm the branded panel renders, is readable, and looks right in **both light and dark** colour schemes.
- Confirm it lands on the Tally form after ~1.5 s.
- **Press Back** — must return to the previous page, *not* bounce forward into the redirect again. This verifies `location.replace()`.
- Click **"Continue now →"** directly — must go straight to the form.
- Disable JavaScript and reload — the meta refresh must still deliver you to the form after ~3 s.

**After deploy** — confirm the live URL:

```bash
# Expect: HTTP/2 200
curl -sSI https://getbodhi.app/tools/free-billion-tokens-bodhi-app/ | head -1

# No-slash form: expect a 301 from GitHub Pages to the trailing-slash URL
curl -sSI https://getbodhi.app/tools/free-billion-tokens-bodhi-app | grep -iE '^(HTTP|location)'

# OG tags present for unfurlers
curl -sS https://getbodhi.app/tools/free-billion-tokens-bodhi-app/ | grep -iE 'og:|twitter:|robots'

# Negative check: the rest of the site is untouched
curl -sSI https://getbodhi.app/      | head -1
curl -sSI https://getbodhi.app/docs/ | head -1
```

**End-to-end:** open the live URL in a fresh browser tab, submit a test response on the Tally form, and confirm in Tally that the submission carries `utm_source=getbodhi.app`, `utm_medium=website`, `utm_campaign=free-billion-tokens-bodhi-app`. Delete the test response afterwards. Also paste the URL into Slack (or a Twitter/LinkedIn draft) to confirm the card unfurls with the Bodhi logo and copy.

## Retirement

When the real page ships at this path:

**Delete `getbodhi.app/public/tools/free-billion-tokens-bodhi-app/` before adding `src/app/tools/free-billion-tokens-bodhi-app/page.tsx`.** A `public/` file and a Next route at the same path collide — leaving the stub in place would shadow the real page. Because the stub is `noindex` and was never a permanent redirect, nothing is cached or indexed against the URL and the swap is clean.

## Known trade-offs

1. **It is a 200, not a 302.** Accepted deliberately. Users can't tell the difference; automated clients and crawlers can.
2. **No wildcard.** Only the exact slug (and its no-slash 301 twin) works. A typo'd variant hits the site's normal 404. Mitigated by keeping the slug short.
3. **UTM params are inert until the Tally hidden fields exist.** The redirect works either way, so this fails silently — do it before publishing the link.
4. **A brief visible interstitial**, by request. Users on a slow connection see the panel a little longer.

## Pre-existing issue noticed (out of scope, flagging only)

`getbodhi.app/public/robots.txt` advertises `Sitemap: https://getbodhi.app/sitemap.xml`, but **no sitemap exists** anywhere — no `src/app/sitemap.ts`, no `public/sitemap.xml`, nothing in `out/`. That URL 404s today. Unrelated to this change; worth a separate fix.
