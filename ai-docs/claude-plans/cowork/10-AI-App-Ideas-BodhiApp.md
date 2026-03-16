# 10 AI App Ideas
## Powered by Bodhi App Platform

**Implementation Blueprints with MCP Wiring & Platform APIs**

Each idea is implementable today using:

- **Chat Completions API** (OpenAI-compatible `/v1/chat/completions`)
- **MCP System Services** (Exa, Brave, Apify, GitHub, Notion, Todoist + more)
- **bodhi-js-sdk** (`npm install @bodhiapp/bodhi-js-react`)
- **App Access Request Flow** (user-consented, scoped JWT authentication)

*March 2026 | Bodhi App | getbodhi.app*

---

## Quick Reference

All 10 ideas ranked by implementation speed and market opportunity. Every idea uses Bodhi App's Chat Completions API + MCP system services. No backend required.

| # | App Name | Category | MCPs Used | Build Time |
|---|----------|----------|-----------|------------|
| 1 | Scout AI | Competitive Intel | Exa, Apify, Brave, Notion | 1 weekend |
| 2 | ShipLog | DevTools | GitHub, Notion, Exa | 2–3 days |
| 3 | BriefMe | Productivity | Todoist, GitHub, Exa, Notion | 1 weekend |
| 4 | DealScope | FinTech / VC | Exa, Apify, Brave, Notion | 1 week |
| 5 | ContentEngine | Content Marketing | Exa, Apify, Notion | 2–3 days |
| 6 | CodeReview.ai | DevTools | GitHub, Exa, Brave | 1 week |
| 7 | LegalLens | LegalTech | Exa, Brave, Notion | 1 week |
| 8 | TrendRadar | Market Intel | Exa, Apify, Brave, Notion | 1 week |
| 9 | DocuMentor | Knowledge Mgmt | GitHub, Notion, Exa | 1 weekend |
| 10 | PitchCraft | Startup Tools | Exa, Brave, Apify, Notion | 2–3 days |

---

## The 10 Ideas

---

### 1. Scout AI — Competitive Intelligence Agent

*Your AI analyst that monitors competitors 24/7 and delivers actionable briefings.*

**CI/Market Intel | B2B SaaS | $20K–$40K ACV market | a16z Top 100 adjacent**

**The Problem:** Startups and VCs spend $20K–$40K/year on competitive intelligence tools like Crayon, Klue, and Unkover. These are bloated dashboards that require manual setup, and the insights are often stale by the time you read them. Meanwhile, the real signals — social media chatter, new GitHub repos, job postings, pricing changes — live across dozens of fragmented sources.

**The Solution:** Scout AI is a lightweight web app powered by Bodhi App that takes a competitor watchlist and delivers real-time intelligence briefings. It chains web search (for news and announcements), social media scraping (for sentiment and executive signals), and a knowledge base (for persistent tracking) — all through Bodhi's OS-level system services. No backend, no API keys to manage, no data infrastructure.

#### MCP Wiring (System Services)

| MCP Server | Role | Key Tools Used |
|------------|------|----------------|
| Exa AI | Deep semantic search | search for funding rounds, product launches, press releases |
| Apify | Social intelligence | scrape Twitter/LinkedIn for executive posts, hiring signals |
| Brave Search | Broad web coverage | catch news articles, blog posts, pricing page changes |
| Notion | Knowledge repository | persist competitor profiles, track changes over time |

#### Bodhi Platform APIs Used

- `/v1/chat/completions` — GPT-4o/4.1 for analysis, synthesis, and report generation
- `/v1/apps/mcps` — access approved MCP servers for search + social + storage
- `bodhi-js-sdk login()` with MCP access request — user consents to 4 system services
- Agentic tool loop — model autonomously decides which sources to query per competitor

#### How It Works (User Flow)

1. User enters 3–5 competitor names/URLs on Scout AI website
2. Scout AI requests access to Bodhi App system services (search, social, knowledge base)
3. User approves in Bodhi App dashboard — scoped JWT token issued
4. Agent runs multi-source research: Exa for deep analysis, Brave for news, Apify for social signals
5. Agent synthesizes a structured competitive briefing with threat assessments
6. Report is exported to user's Notion workspace as a persistent, updatable page

**Monetization:** Freemium: 3 competitors free, unlimited at $29/mo. Enterprise: custom watchlists at $199/mo.

**Product Hunt Comparables:** Crayon ($20K+/yr), Klue (250K+ users), Unkover (unified market intelligence), HeadsUp (AI CI agent)

---

### 2. ShipLog — AI Changelog & Release Notes Generator

*Turns your GitHub commits and PRs into beautiful, audience-ready release notes automatically.*

**DevTools | Content Ops | PLG | Developer favorite on PH**

**The Problem:** Engineering teams ship code constantly but hate writing release notes. The disconnect between what's merged and what users see is massive. Product marketers spend hours translating technical PRs into customer-facing changelogs. Tools like GitHub's auto-generated notes are too technical, while manual writing is too slow.

**The Solution:** ShipLog connects to your GitHub via Bodhi App, reads recent PRs and commits, understands the changes using an LLM, and generates audience-appropriate release notes — technical for developers, simplified for end users, marketing-ready for announcements. It then publishes to your Notion changelog page or creates a formatted document.

#### MCP Wiring (System Services)

| MCP Server | Role | Key Tools Used |
|------------|------|----------------|
| GitHub | Code intelligence | read PRs, commits, diff summaries, labels, and milestones |
| Notion | Publishing destination | create/update changelog pages with formatted release notes |
| Exa AI | Context enrichment | research libraries/APIs mentioned in PRs for richer descriptions |

#### Bodhi Platform APIs Used

- `/v1/chat/completions` — LLM generates multiple audience versions of the same changes
- `/v1/apps/mcps` — access GitHub (read) and Notion (write) through approved connections
- Agentic loop — model reads PRs, categorizes changes, researches context, writes notes

#### How It Works (User Flow)

1. Developer connects ShipLog to Bodhi App, granting access to GitHub and Notion MCPs
2. ShipLog fetches recent merged PRs and commits from specified repositories
3. Agent categorizes changes (features, fixes, breaking changes, internal)
4. Agent generates three versions: technical changelog, user-facing notes, marketing announcement
5. User reviews and edits in ShipLog's clean UI
6. One-click publish to Notion changelog page with proper formatting

**Monetization:** Free for open source. $19/mo for private repos. $49/mo for team features (review workflow, scheduling).

**Product Hunt Comparables:** Chronicle (Cursor for Slides, PH Best 2025), Mintlify (developer docs), Loki.Build (AI landing pages)

---

### 3. BriefMe — AI Daily Executive Briefing

*Wake up to a personalized intelligence briefing synthesized from your tools, news, and market signals.*

**Productivity | Executive Tools | $7.3B meeting assistant market | Google CC competitor**

**The Problem:** Executives and founders start their day checking 5–10 different apps: email, Slack, project management, news, calendar. Google's CC agent showed the demand for 'Your Day Ahead' briefings, but it's locked to Google Workspace. The meeting assistant market alone is $3.24B and growing to $7.33B by 2035. People want a single, synthesized view of what matters today.

**The Solution:** BriefMe is a personal intelligence dashboard that connects to your productivity tools through Bodhi App and delivers a morning briefing: what's on your calendar, what happened in your project management tool overnight, what's trending in your industry, and what needs your attention. All synthesized by an AI agent into a 2-minute read.

#### MCP Wiring (System Services)

| MCP Server | Role | Key Tools Used |
|------------|------|----------------|
| Todoist | Task intelligence | fetch today's tasks, overdue items, priority changes |
| GitHub | Dev activity | overnight PRs, review requests, CI failures, mentions |
| Exa AI | Industry news | semantic search for news relevant to user's industry/interests |
| Notion | Briefing archive | save daily briefings as a running log for reference |

#### Bodhi Platform APIs Used

- `/v1/chat/completions` — LLM synthesizes multi-source data into a coherent narrative briefing
- `/v1/apps/mcps` — fan-out reads across 3–4 MCPs, then write to archive
- `bodhi-js-sdk` — handles auth, token management, MCP access for the web app

#### How It Works (User Flow)

1. User configures BriefMe: connects Bodhi App, selects which system services to include
2. Each morning, BriefMe triggers an agentic research cycle across all connected services
3. Agent reads tasks (Todoist), dev activity (GitHub), and industry news (Exa)
4. Agent synthesizes a prioritized briefing: 'Top 3 things that need your attention today'
5. Briefing displayed in clean web UI and archived to Notion
6. User can ask follow-up questions in chat: 'Tell me more about that PR'

**Monetization:** Free for 1 source. $12/mo for unlimited sources. $29/mo adds scheduled delivery and team briefings.

**Product Hunt Comparables:** Dimension (proactive AI assistant, PH 2025 favorite), Google CC (Workspace AI), Fireflies/Otter (notetakers)

---

### 4. DealScope — AI Due Diligence Research Agent

*Give it a company name. Get back a structured investment memo with risk flags in minutes, not weeks.*

**FinTech | VC/PE Tools | $2B+ market | DiligenceSquared raised $5M seed**

**The Problem:** Due diligence is a $2B+ market. VCs and PE firms spend 5–10 days per deal on first-pass research: financials, market position, team background, competitive landscape, risk factors. Tools like Keye, ToltIQ, and DiligenceSquared are raising millions to solve this. But they're all monolithic platforms with enterprise pricing. No lightweight tool exists for angel investors, scouts, or early-stage VCs who evaluate 50+ deals per month.

**The Solution:** DealScope is a focused research agent that takes a company name or URL and produces a structured investment memo. It uses web search for company data, social media intelligence for team/culture signals, and exports a formatted memo to your knowledge base. Powered entirely by Bodhi App — the user's own AI infrastructure, keeping sensitive deal data under their control.

#### MCP Wiring (System Services)

| MCP Server | Role | Key Tools Used |
|------------|------|----------------|
| Exa AI | Deep company research | find funding history, press coverage, product reviews, team bios |
| Apify | Social/team intelligence | scrape LinkedIn for team backgrounds, Twitter for founder activity |
| Brave Search | Broad coverage | catch Crunchbase data, news articles, regulatory filings |
| Notion | Memo repository | export structured investment memo as a Notion page |

#### Bodhi Platform APIs Used

- `/v1/chat/completions` — GPT-4o for structured analysis, risk assessment, and memo generation
- `/v1/apps/mcps` — multi-source research through 4 approved system services
- Agentic loop with tool chaining — research → analyze → structure → export

#### How It Works (User Flow)

1. Investor enters company name/URL in DealScope
2. DealScope requests Bodhi App access to research + social + knowledge MCPs
3. Agent runs parallel research: Exa for deep company intel, Brave for news, Apify for team signals
4. Agent structures findings into standard memo format: overview, market, team, financials, risks, recommendation
5. User reviews memo in DealScope UI, can ask follow-up questions
6. One-click export to Notion deal pipeline with structured fields

**Monetization:** Free for 3 memos/mo. $49/mo for unlimited. $199/mo adds batch analysis and portfolio monitoring.

**Product Hunt Comparables:** DiligenceSquared ($5M seed, PE research), Keye (AI due diligence), ToltIQ (PE platform), V7 Go (VDR agent)

---

### 5. ContentEngine — AI Content Repurposing Pipeline

*One blog post in. Ten platform-optimized social posts, a newsletter draft, and a thread — out.*

**Content Marketing | Creator Tools | High PH engagement category | Saves 5+ hours/week**

**The Problem:** Content repurposing is the #1 efficiency hack for marketing teams, but doing it well is incredibly time-consuming. You write a blog post, then manually adapt it for Twitter/X threads, LinkedIn posts, newsletter format, and short-form video scripts. Tools like Narrato, Repurpose.io, and MeetEdgar each handle one piece. Nothing does the full pipeline from one source asset to all formats, with brand voice consistency.

**The Solution:** ContentEngine takes a single content asset (blog URL, document, or raw text), analyzes it through web search to add current context and data points, then uses an LLM to generate 10+ platform-optimized outputs. All generated content is saved to your Notion content calendar. Zero AI infrastructure — Bodhi App provides the LLM + MCP services.

#### MCP Wiring (System Services)

| MCP Server | Role | Key Tools Used |
|------------|------|----------------|
| Exa AI | Content enrichment | find current stats, quotes, and examples to enhance content |
| Apify | Source scraping | extract full content from blog URLs, analyze competitor posts for style |
| Notion | Content calendar | save all generated content as organized Notion database entries |

#### Bodhi Platform APIs Used

- `/v1/chat/completions` — LLM generates platform-specific adaptations with brand voice
- `/v1/apps/mcps` — scrape source content, research enrichment data, save outputs
- Agentic loop — model analyzes source, researches context, generates multiple formats

#### How It Works (User Flow)

1. User pastes a blog URL or text into ContentEngine
2. Agent scrapes full content via Apify, analyzes structure and key messages
3. Agent uses Exa to find current stats, quotes, and trending angles to enrich the content
4. Agent generates: 5 Twitter/X posts, 3 LinkedIn posts, 1 newsletter intro, 1 thread outline
5. All outputs displayed in ContentEngine UI with platform-specific previews
6. One-click save to Notion content calendar with publish dates and platform tags

**Monetization:** Free for 5 repurposes/mo. $19/mo for unlimited. $49/mo adds brand voice training and team access.

**Product Hunt Comparables:** Narrato (content repurposing), MeetEdgar (social scheduling), Wispr Flow (PH 2025 favorite), Buffer (AI social)

---

### 6. CodeReview.ai — AI Pull Request Reviewer

*Automated, context-aware code reviews that understand your codebase, not just syntax.*

**DevTools | AI Coding | Cursor/Claude Code adjacent | Top PH category 2026**

**The Problem:** AI coding agents (Cursor, Claude Code, Copilot) help write code, but code review is still a bottleneck. Senior engineers spend 5–10 hours/week reviewing PRs. Existing AI review tools do surface-level linting. They don't understand your architecture, your conventions, or the broader context of why a change was made. The result: noisy, unhelpful comments that reviewers ignore.

**The Solution:** CodeReview.ai connects to your GitHub through Bodhi App, reads the PR diff, then researches the broader context — related files, recent changes, library documentation — before generating a thoughtful review. It's like having a senior engineer who actually reads the docs and understands the codebase before commenting.

#### MCP Wiring (System Services)

| MCP Server | Role | Key Tools Used |
|------------|------|----------------|
| GitHub | PR access | read PR diffs, file history, related issues, CI status, previous reviews |
| Exa AI | Documentation research | find docs for libraries/APIs used in the PR, best practices |
| Brave Search | Broader context | research error patterns, security advisories, deprecation notices |

#### Bodhi Platform APIs Used

- `/v1/chat/completions` — LLM with tool use for contextual, multi-file code analysis
- `/v1/apps/mcps` — GitHub (read PRs), Exa + Brave (research context)
- Agentic loop — model reads diff, identifies concerns, researches context, writes review

#### How It Works (User Flow)

1. Developer installs CodeReview.ai and connects to Bodhi App with GitHub MCP access
2. On new PR, agent reads the full diff and changed file context
3. Agent identifies libraries, patterns, and APIs used in the changes
4. Agent researches documentation and best practices via web search MCPs
5. Agent generates a structured review: summary, concerns, suggestions, with citations
6. Review posted as GitHub PR comment or displayed in CodeReview.ai dashboard

**Monetization:** Free for public repos. $29/mo per developer for private repos. $99/mo for team with custom rules.

**Product Hunt Comparables:** Cursor (PH top AI coding), Claude Code (terminal-first reviews), Amp (free AI coding, PH 2025 favorite)

---

### 7. LegalLens — AI Contract Analysis Agent

*Upload a contract. Get a plain-English risk summary, clause-by-clause analysis, and negotiation suggestions.*

**LegalTech | B2B SaaS | Regulated industry | $1.8B MCP market in regulated fields**

**The Problem:** Startups sign dozens of contracts: vendor agreements, partnership deals, NDAs, SOWs. Most founders don't have legal counsel on retainer and can't afford $500/hour lawyers for every contract review. AI legal tools exist (LegalFly, Harvey) but they're enterprise-priced and require uploading sensitive documents to third-party clouds. Privacy-conscious founders want their legal AI to run on their own infrastructure.

**The Solution:** LegalLens analyzes contracts using Bodhi App's LLM and enriches the analysis with legal research via web search MCPs. The key differentiator: Bodhi App can run locally, so sensitive contracts never leave the user's machine. The agent reads the contract, identifies risky clauses, searches for relevant legal precedents and standard terms, and delivers a structured risk report.

#### MCP Wiring (System Services)

| MCP Server | Role | Key Tools Used |
|------------|------|----------------|
| Exa AI | Legal research | find relevant case law, standard clause language, regulatory guidance |
| Brave Search | Broad legal context | search for jurisdiction-specific regulations, recent rulings |
| Notion | Report repository | export analysis as a structured Notion page for legal team review |

#### Bodhi Platform APIs Used

- `/v1/chat/completions` — LLM for clause extraction, risk scoring, and plain-English summaries
- `/v1/apps/mcps` — web search for legal research, Notion for report export
- `bodhi-js-sdk` — local-first connection option for maximum contract privacy

#### How It Works (User Flow)

1. User uploads contract PDF/DOCX to LegalLens web app
2. Agent extracts and parses all clauses from the document
3. Agent identifies potentially risky clauses: liability caps, termination terms, IP assignment, non-competes
4. Agent researches each flagged clause via Exa (legal precedents) and Brave (regulatory context)
5. Agent generates: executive summary, clause-by-clause analysis, risk score, and negotiation suggestions
6. Report exported to Notion with per-clause bookmarks for team discussion

**Monetization:** Free for NDA/simple contracts. $39/mo for all contract types. $149/mo adds redline generation and batch analysis.

**Product Hunt Comparables:** Harvey (AI lawyer), LegalFly (M&A due diligence), open-legal-compliance-mcp (open source legal MCP)

---

### 8. TrendRadar — AI Market Signal Aggregator

*Monitors the pulse of any market and alerts you when something important shifts.*

**Market Intelligence | VC/Founder tool | Real-time signals | Competitive moat via data aggregation**

**The Problem:** Founders and investors need to stay on top of market signals: funding rounds, product launches, hiring spikes, regulatory changes, social sentiment shifts. Currently this requires manually checking Crunchbase, Twitter/X, TechCrunch, regulatory databases, and competitor blogs daily. By the time you notice a signal, your competitors already have. Tools like CB Insights and PitchBook cost $30K+/year and are designed for large enterprises.

**The Solution:** TrendRadar is a lightweight, always-on market monitoring agent. Users define their market (e.g., 'AI developer tools' or 'Southeast Asian fintech'), and the agent continuously scans web sources, social media, and tech publications. When it detects a significant signal (new funding round, product pivot, executive departure, regulatory change), it sends a structured alert and updates a persistent market map in Notion.

#### MCP Wiring (System Services)

| MCP Server | Role | Key Tools Used |
|------------|------|----------------|
| Exa AI | Semantic market scanning | deep search for funding, launches, patents, regulatory filings |
| Apify | Social signal detection | monitor Twitter/LinkedIn for executive posts, hiring announcements |
| Brave Search | News monitoring | broad web scan for press releases, blog posts, conference talks |
| Notion | Market map | persistent database tracking all signals, trends, and entities over time |

#### Bodhi Platform APIs Used

- `/v1/chat/completions` — LLM classifies signals, assesses significance, generates alerts
- `/v1/apps/mcps` — 4-MCP fan-out for comprehensive market coverage
- Agentic loop — scan → classify → assess significance → alert → archive

#### How It Works (User Flow)

1. User defines their market and key entities to watch (competitors, adjacent companies, technologies)
2. TrendRadar connects to Bodhi App with 4 MCP services approved
3. Agent runs periodic scans across all sources for new signals
4. Agent classifies signals (funding, product, team, regulatory, sentiment) and scores significance
5. High-priority signals generate real-time alerts in the TrendRadar dashboard
6. All signals archived to Notion market map with entity relationships and timeline

**Monetization:** Free for 1 market. $29/mo for 3 markets. $99/mo for unlimited markets + team alerts + API access.

**Product Hunt Comparables:** Unkover (unified market intelligence), CB Insights (predictions), Crayon (competitive monitoring)

---

### 9. DocuMentor — AI Knowledge Base Builder

*Point it at your scattered docs, wikis, and repos. Get a structured, searchable knowledge base in minutes.*

**Knowledge Management | Enterprise | KNOA-adjacent | Tackles $71B AI SaaS market**

**The Problem:** Every growing team has the same problem: knowledge is scattered across Notion pages, Google Docs, GitHub READMEs, Slack threads, and people's heads. New hires spend weeks figuring out where things are. Knowledge base tools (Confluence, Notion, Mintlify) require manual curation. The average engineer spends 30 minutes/day searching for internal information. That's $15K/year per engineer in lost productivity.

**The Solution:** DocuMentor is an AI agent that reads your existing documentation sources through Bodhi App's MCP services, identifies gaps, resolves contradictions, and generates a unified, structured knowledge base. It doesn't replace your tools — it reads from GitHub, Notion, and web sources, then writes a clean, organized knowledge map back to Notion. Think of it as an AI technical writer that never sleeps.

#### MCP Wiring (System Services)

| MCP Server | Role | Key Tools Used |
|------------|------|----------------|
| GitHub | Code documentation | read READMEs, code comments, PR descriptions, wiki pages |
| Notion | Knowledge read + write | read existing Notion docs, write organized knowledge base |
| Exa AI | External reference | find official docs for libraries/tools referenced in internal docs |

#### Bodhi Platform APIs Used

- `/v1/chat/completions` — LLM for content analysis, gap detection, and knowledge synthesis
- `/v1/apps/mcps` — GitHub (read), Notion (read + write), Exa (reference enrichment)
- Agentic loop — crawl sources → analyze → identify gaps → generate → organize

#### How It Works (User Flow)

1. Team lead connects DocuMentor to Bodhi App with GitHub + Notion MCP access
2. Agent crawls all connected sources: GitHub repos, Notion workspace, specified URLs
3. Agent builds a topic map: what's documented, what's outdated, what's missing
4. Agent generates structured knowledge base pages with proper cross-references
5. Agent enriches with external documentation links for referenced tools/libraries
6. Knowledge base published to a dedicated Notion workspace with search-optimized structure

**Monetization:** Free for small teams (1 repo + Notion). $49/mo for unlimited sources. $199/mo adds freshness monitoring.

**Product Hunt Comparables:** KNOA (AI knowledge capture, PH featured), Mintlify (developer docs), Notion AI (workspace intelligence)

---

### 10. PitchCraft — AI Pitch Deck Research & Narrative Builder

*Tell it your startup idea. Get back a research-backed narrative with market data, comps, and talking points.*

**Startup Tools | VC-adjacent | Chronicle/Gamma competitor | PH Best 2025 category**

**The Problem:** Every founder building a pitch deck faces the same grind: researching TAM/SAM/SOM, finding comparable companies, sourcing market data, and crafting a compelling narrative. Tools like Chronicle and Gamma make beautiful slides, but they don't do the research. You still need to manually find and verify every data point. McKinsey charges $100K+ for the research that goes into a single pitch deck's market slide.

**The Solution:** PitchCraft takes your startup description and does the research work that goes behind a great pitch deck: market sizing with cited sources, competitive landscape analysis, comparable company identification with funding data, and industry trend analysis. It synthesizes everything into a structured narrative document that you can then design into slides using any tool. The insight, not the design, is the hard part.

#### MCP Wiring (System Services)

| MCP Server | Role | Key Tools Used |
|------------|------|----------------|
| Exa AI | Deep market research | find TAM/SAM data, industry reports, analyst predictions |
| Brave Search | Competitor data | Crunchbase funding data, competitor websites, market reports |
| Apify | Social proof | scrape ProductHunt upvotes, Twitter buzz, LinkedIn job postings for market signals |
| Notion | Narrative export | structured pitch narrative with all sources and data points |

#### Bodhi Platform APIs Used

- `/v1/chat/completions` — GPT-4o for market analysis, narrative structuring, data synthesis
- `/v1/apps/mcps` — 3-source research (Exa + Brave + Apify) and Notion export
- Agentic loop — research market → find comps → analyze trends → craft narrative

#### How It Works (User Flow)

1. Founder describes their startup in 2–3 sentences on PitchCraft
2. PitchCraft requests Bodhi App access to research + social + knowledge MCPs
3. Agent researches: market size (Exa), competitors and funding (Brave), social signals (Apify)
4. Agent structures findings into pitch deck sections: Problem, Market, Solution, Traction, Comps, Ask
5. Each section includes cited data points, quotable stats, and source URLs
6. Full narrative exported to Notion as a structured document ready for slide design

**Monetization:** Free for 1 pitch/mo. $29/mo for unlimited. $79/mo adds investor-specific customization and updates.

**Product Hunt Comparables:** Chronicle (Cursor for Slides, PH Best 2025), Gamma (AI presentations), Lovable (AI MVP builder, PH 2025)

---

## Platform Architecture Summary

Every app in this document follows the same architectural pattern, proving that Bodhi App functions as a true Operating System for AI Apps:

### The Universal Pattern

1. Third-party web app built with React + bodhi-js-sdk (zero AI backend)
2. App requests access to Bodhi App system services (MCP servers)
3. User reviews and approves in Bodhi App dashboard (Android-like permissions)
4. App receives scoped JWT token — no API keys shared, no backend needed
5. App calls `/v1/chat/completions` with tool use — LLM + MCP agentic loop
6. Agent reads from input MCPs, thinks, acts, and writes to output MCPs

### Available MCP Servers (Verified)

- **Exa AI** — Semantic web search, academic papers, funding data, deep research
- **Brave Search** — Broad web search, news, privacy-focused, real-time results
- **Apify** — Web scraping, Twitter/LinkedIn intelligence, data extraction from any website
- **GitHub** — Repository management, PR access, code search, issue tracking
- **Notion** — Knowledge base read/write, database management, page creation
- **Todoist** — Task management, project tracking, natural language task creation
- **Slack** — Channel messaging, thread access, workspace history
- **Linear** — Issue tracking, project management, sprint planning

---

Start building: **[cloud.getbodhi.app](https://cloud.getbodhi.app)**

```
npm install @bodhiapp/bodhi-js-react
```
