# Bodhi App — AI Festival Demo Plan

**Event:** AI Festival | **Date:** March 16, 2026, 10:00 AM | **Slot:** 7 minutes
**Presenter:** Amir, Founder of Bodhi App
**Audience:** Startup founders, entrepreneurs, senior technical developers, VCs

---

## Core Narrative

**One-liner:** "Bodhi App is the Operating System for AI Apps."

**Analogy (the hook):** Android gives mobile apps system services — GPS, camera, contacts, notifications, storage. The app doesn't build those; it requests permission, and the OS provides. **Bodhi App does the same for AI.** Any website can request access to LLM inference, web search, social media intelligence, knowledge management — all authenticated, user-consented, and ready to use. Zero AI infrastructure required.

**Credibility bridge:** "I'm Amir. I spent 20+ years building software. At Gojek, I architected the super-app of Southeast Asia — one app connecting millions of users to hundreds of services. Now I'm building the same thing for AI. Bodhi App is the platform layer that lets any website become AI-powered."

---

## Demo App: Bodhi Bot

**What it is:** A standalone React app (React + Vite + Tailwind + shadcn) that functions as an AI-powered Research Assistant. It has zero AI infrastructure of its own. All intelligence comes from Bodhi App's "system services" via `@bodhiapp/bodhi-js-react`.

**Research topic for live demo:** "Geopolitical impact of closure of Hormuz Strait on South Asia"

**Why this topic:** Complex, current, multi-dimensional. Not a toy example. The kind of analysis VCs pay consultants $50K for. When the agent produces it in 2 minutes using three different data sources, the audience will feel the value viscerally.

### MCP Stack (System Services)

| MCP | Role in Demo | Analogy to Android |
|-----|-------------|-------------------|
| **Exa AI** | Semantic web search — finds academic papers, news analysis, geopolitical reports | Like Android's "Internet" permission |
| **Apify** | Real-time social intelligence — scrapes Twitter/LinkedIn for live sentiment and expert takes | Like Android's "Contacts + Social" — access to people's signals |
| **Notion** | Knowledge repository — exports the final research report as a structured Notion page | Like Android's "Storage" — persistent, organized data |

**Fallback plan (if Apify fails testing tonight):**
Replace Apify with **Brave Search** or **Tavily**. Reframe the third MCP as "broad web search" alongside Exa's "deep semantic search." The narrative becomes: "Two search engines triangulating sources, then exporting to your knowledge base." Still compelling, still shows 3 system services.

---

## Presentation Script — Minute by Minute

### SLIDES: Introduction (0:00 – 1:30)

**Slide 1 — Title (10 seconds)**
- Bodhi App logo (centered, large)
- Tagline: "The Operating System for AI Apps"
- Your name, small: Amir · Founder

**Slide 2 — The Problem (30 seconds)**
> *"Every website that wants AI has to build the same thing from scratch. They integrate an LLM. They build auth. They add tool connections. They manage API keys. They handle user consent. It's like every mobile app building its own GPS chip."*

Visual: Left side shows a mess of logos (OpenAI, Anthropic, various MCP servers, auth systems) with arrows everywhere. Right side shows a clean phone icon with "Android" underneath — the message is clear without saying it.

**Slide 3 — The Solution (30 seconds)**
> *"Bodhi App is the OS layer. You install it — locally on your machine, as a Docker container, or use our cloud service. It provides AI system services: LLM inference, web search, social intelligence, knowledge management, file access — all behind authentication, user consent, and fine-grained permissions. Any website connects through our SDK with 3 lines of code."*

Visual: Clean architecture diagram. Bodhi App in the center as a purple circle. Above it: multiple website icons connecting down. Below it: MCP servers as "system services" (labeled: Search, Social, Knowledge, Code, Tasks). Arrows show the websites accessing services through Bodhi.

**Slide 4 — Transition to Demo (20 seconds)**
> *"Let me show you. I built a simple React app called Bodhi Bot. It's a research assistant — you give it a brief, and it uses Bodhi App's system services to research the web, gather social intelligence, and write a full report to your Notion. It has zero AI backend. All the intelligence comes from Bodhi App. Let me walk you through it."*

Visual: Screenshot of Bodhi Bot UI (polished, ready state). Simple, clean.

---

### LIVE DEMO: Access Request Flow (1:30 – 2:30)

**Setup on screen:** Browser with two tabs ready — (1) Bodhi Bot at its URL, (2) Bodhi App dashboard at cloud.getbodhi.app

> *"First, Bodhi Bot needs to connect to my Bodhi App. Just like a mobile app asks for permissions when you first open it, Bodhi Bot requests access to the services it needs."*

**Action:** Click "Connect to Bodhi App" button in Bodhi Bot.

> *"It's requesting access to three system services: Exa for web search, Apify for social media intelligence, and Notion for knowledge storage. This request goes to my Bodhi App."*

**Action:** Switch to Bodhi App dashboard tab. Show the access request review page.

> *"As the user, I can see exactly what this app is requesting. I can approve all, deny some, or grant partial access. Full user control — just like Android permissions. I'll approve all three."*

**Action:** Click approve. Switch back to Bodhi Bot tab. Show the connection established state (green indicators for 3 MCPs).

> *"Connected. Bodhi Bot now has access to three AI system services, with a scoped JWT token. No API keys were shared. No backend was built. The app just asked the OS, and the OS provided."*

---

### LIVE DEMO: Research in Action (2:30 – 5:30)

**Beat 1 — The Brief (2:30 – 3:00)**

> *"Now let's use it. I'll give Bodhi Bot a real research brief — not a toy example."*

**Action:** Type into Bodhi Bot chat:
```
Research the geopolitical impact of a potential closure of the Strait of Hormuz on South Asian economies. Find recent analysis, expert opinions from social media, and create a comprehensive briefing.
```

> *"This is the kind of analysis a VC or founder might need before making an investment decision in the region. Let's see what Bodhi Bot does with its system services."*

**Beat 2 — Agent Working (3:00 – 4:00)**

The agent will begin its agentic loop. Narrate what's happening as tool calls appear:

> *"Watch what's happening. The agent is calling Exa — that's deep semantic search across academic papers, news analysis, and reports. It found 8 relevant sources."*

(pause as results stream)

> *"Now it's calling Apify — scraping recent Twitter threads and LinkedIn posts from geopolitical analysts and defense experts. Real-time social intelligence."*

(pause as results stream)

> *"And now it's synthesizing. The LLM is combining structured research with real-time social signals to build a comprehensive picture. This is the agentic loop — the model decides which tools to call, in what order, and how to combine the results."*

**Beat 3 — The Report (4:00 – 5:00)**

> *"Here's the synthesized report — sources cited, multiple perspectives, real-time social sentiment. And now watch this..."*

**Action:** The agent calls the Notion MCP to export the report.

> *"It's writing the full report to Notion. Let me show you."*

**Action:** Switch to Notion tab. Show the page appearing/appeared.

> *"A real Notion page. Structured, sourced, ready to share with your team. Created by a React app with zero AI infrastructure — powered entirely by Bodhi App's system services."*

**Beat 4 — The Zoom Out (5:00 – 5:30)**

> *"What you just saw was three system services — search, social intelligence, and knowledge storage — composed by an AI agent, accessed through user consent, delivered to a simple web app. The app didn't build any of this. Bodhi App provided it."*

---

### CLOSING: Vision + CTA (5:30 – 7:00)

> *"This is just one app. Imagine thousands. A sales tool that accesses your CRM, email, and market data through Bodhi. A customer support agent that reads your docs, checks your ticket system, and responds — all through the same OS layer. A personal finance app that aggregates your accounts and provides AI-powered advice — with the user controlling exactly what it can access."*

> *"Bodhi App is open source, privacy-first, and runs anywhere — your laptop, your server, or our cloud. The SDK is on npm right now. You can build your first AI-powered app in an afternoon."*

> *"We're live at cloud.getbodhi.app — sign up, get a free account, connect your first MCP server, and start building. Star us on GitHub. Join our Discord. And if you want to talk about what we're building — find me after this."*

> *"Bodhi App. The Operating System for AI Apps. Thank you."*

---

## Tonight's Preparation Checklist (2-3 hours)

### Hour 1: MCP Testing (Critical Path)

1. **Test Exa AI MCP** (15 min)
   - Ensure MCP server is registered on cloud.getbodhi.app
   - Create MCP instance, fetch tools, execute a search query
   - Verify: response quality, speed, reliability

2. **Test Apify MCP** (20 min)
   - Register Apify MCP server
   - Configure Twitter/LinkedIn scraping actors
   - Test end-to-end: tool discovery → execution → results
   - **If it fails:** Immediately switch to Brave Search or Tavily as fallback
   - Update demo script accordingly (change "social intelligence" to "broad web search")

3. **Test Notion MCP** (15 min)
   - Register Notion MCP server
   - Create instance, verify write access
   - Test: create a page with formatted content
   - Verify the page appears correctly in Notion workspace

4. **Verify MCP chaining** (10 min)
   - From Bodhi App's chat, test a prompt that triggers all 3 MCPs in sequence
   - Time the full execution — needs to complete in under 90 seconds for demo pacing

### Hour 2: Bodhi Bot App

1. **Scaffold React + Vite + Tailwind + shadcn** (10 min)
   - `npm create vite@latest bodhi-bot -- --template react-ts`
   - Install dependencies: `@bodhiapp/bodhi-js-react`, tailwindcss, shadcn

2. **Build the UI** (30 min)
   - Polished chat interface with message history
   - Top bar showing connected MCPs (green dots when connected)
   - "Connect to Bodhi App" button for initial connection
   - Tool call visualization (show which MCP is being called, with small icons)
   - Streaming response display

3. **Integration** (20 min)
   - Configure BodhiProvider with cloud.getbodhi.app as basePath
   - Implement login flow with MCP access request
   - Implement agentic chat loop (tool discovery, execution, re-prompting)
   - Test full flow: connect → approve → research → export

### Hour 3: Polish + Rehearse

1. **Visual polish** (15 min)
   - Increase font sizes for projector visibility (16px minimum body, 20px+ chat)
   - Set browser zoom to 125-150% for demo
   - Test dark mode vs light mode (light mode is usually better for projectors)
   - Ensure Bodhi App logo is visible in the app

2. **Rehearse the full 7 minutes** (30 min)
   - Run through the entire script 2-3 times
   - Time each section — adjust if over/under
   - Practice the transitions between slides and demo
   - Test the exact prompt — refine wording if agent doesn't behave as expected
   - Pre-type the research prompt in a text file for quick paste on stage

3. **Prepare fallbacks** (15 min)
   - Screenshot every step of a successful run
   - If live demo fails, you can narrate over screenshots
   - Have the Notion page already created from a test run as backup
   - Pre-load all tabs, pre-login to everything

---

## Technical Setup for Stage

### Browser Setup
- **Tab 1:** Bodhi Bot (the demo app)
- **Tab 2:** Bodhi App dashboard (cloud.getbodhi.app) — logged in, ready
- **Tab 3:** Notion workspace — ready to show the exported page
- **Tab 4:** (hidden) Backup screenshots of successful run

### Display Settings
- Browser zoom: 133% minimum
- Hide bookmark bar
- Full screen browser (F11)
- Light mode for projector visibility
- Close all other apps, disable notifications

### Pre-Demo Checklist (5 minutes before slot)
- [ ] All 3 tabs loaded and logged in
- [ ] Bodhi Bot at initial state (not connected yet)
- [ ] Research prompt ready in clipboard
- [ ] Internet connection verified (speed test)
- [ ] Phone on silent
- [ ] Slide deck ready (separate window or first tab)

---

## Risk Matrix

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| WiFi drops during demo | Medium | Critical | Use phone hotspot as backup. Pre-load all pages. |
| LLM response is slow (>30s) | Low | High | Using GPT-4o on cloud — fast. If slow, narrate: "While it processes..." |
| Apify MCP fails | Medium | Medium | Fallback to Brave/Tavily. Script barely changes. |
| Agent makes wrong tool calls | Low | Medium | Rehearse exact prompt. Have backup prompt ready. |
| Notion export fails | Low | High | Have pre-created Notion page. Switch to that tab. |
| Access approval flow glitches | Very Low | High | Pre-approve as backup. Show screenshots if needed. |
| Audience Q&A eats into time | N/A | N/A | No Q&A in 7-min slot. If asked: "Find me after!" |

---

## Key Messaging Notes

### Do Say
- "Operating System for AI Apps"
- "System services" (not "APIs" or "integrations")
- "User consent and permissions" (not "auth flow")
- "Any website" (emphasize universality)
- "Zero AI infrastructure" (emphasize the value prop)
- "3 lines of code" (SDK simplicity)
- "Open source, privacy-first, runs anywhere"

### Don't Say
- Don't say "MCP" without explaining it — say "tool connections" or "system services"
- Don't dive into technical implementation details
- Don't compare to specific competitors by name
- Don't mention limitations or roadmap items
- Don't say "we're early" or "it's alpha/beta" — project confidence

### Audience-Specific Hooks
- **VCs:** "Platform play. Network effects. Every app built on Bodhi makes the ecosystem more valuable."
- **Founders:** "Ship AI features in your product this week. No ML team required."
- **Developers:** "npm install, 3 lines of code, OpenAI-compatible API. You know the drill."

---

## Slide Deck Specs

**Format:** 4 slides, 16:9, dark background (Bodhi purple #a855f7 accents on dark navy #0f172a)
**Font:** Clean sans-serif (Inter or system), large (40pt+ titles, 24pt+ body)
**Logo:** Bodhi App logo (SVG from /public/bodhi-logo/)

| Slide | Content | Duration |
|-------|---------|----------|
| 1 | Logo + "The Operating System for AI Apps" + name | 10s |
| 2 | The Problem: visual chaos of building AI from scratch | 30s |
| 3 | The Solution: clean Bodhi architecture diagram | 30s |
| 4 | "Let me show you" + Bodhi Bot screenshot | 20s |

---

## Post-Demo Assets Needed

- cloud.getbodhi.app account creation flow should be smooth
- GitHub repo should be starred-ready and have a clean README
- Discord invite link should be active
- Consider: QR code on the final slide linking to cloud.getbodhi.app
