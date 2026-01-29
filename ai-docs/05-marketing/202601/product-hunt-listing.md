# Product Hunt Listing - Bodhi Chat

## Launch Details

### Name (80 char limit)
**Bodhi Chat - Powered by Bodhi App**

### Tagline (60 char limit)
**PoC for Bodhi Platform, create webapps powered by local LLMs**

### Links
- Demo: https://bodhiapps.github.io/chat/
- GitHub: https://github.com/bodhiapps/chat
- Developer Console: https://developer.getbodhi.app
- Bodhi App: https://github.com/BodhiSearch/BodhiApp
- bodhi-js SDK: https://github.com/BodhiSearch/bodhi-js

### X account of the launch
@bodhiapphq

---

## What's New/Different (260 char limit)

**"First web app that securely connects to local LLMs via OAuth 2.1. Unlike Ollama/LM Studio, Bodhi App lets web apps access your local models with granular permissions. Devs can build similar apps with our SDK - no backend needed."** (241 chars)

---

## Description of the Launch (500 char limit)

**Bodhi Chat is a PoC showing the future of AI Apps with Local LLMs. Connects to your locally running Bodhi App securely via OAuth 2.1. Webapps hosted anywhere can access your local AI - enabling agentic chat + deep research entirely on your machine. Zero backend infrastructure needed, zero API costs, 100% privacy. Developers: build similar AI apps powered by Local LLMs? Sign up for waitlist at developer.getbodhi.app** (408 chars)

---

## Launch Tags (select up to 3)
1. **Privacy**
2. **Developer Tools**
3. **Artificial Intelligence**

---

## Quick Reference: Critical Timeline (IST)

**LAUNCH DAY: Monday**
- **1:31 PM IST** - Submit product to Product Hunt (12:01 AM PT)
- **1:31-5:31 PM IST** - CRITICAL 4-hour window (highest priority, respond to every comment)
- **8:31 PM IST** - Major push (US East Coast online, send Email Batch 2)

**TUESDAY**
- **3:31 AM IST** - Afternoon push (US West Coast peak, send Email Batch 3)
- **9:31 AM IST** - Final push (Europe evening, send Email Batch 4)

**3-DAY PREP COUNTDOWN:**
- **Friday:** Prepare all assets, reach out to 50-100 people personally
- **Saturday:** Twitter thread + LinkedIn post + community shares (Discord/Slack/Reddit)
- **Sunday 8 PM IST:** Send Email Batch 1 (Asia/Australia), finalize everything

**Email Schedule:**
- Batch 1 (Asia/Australia): Sunday 8 PM IST
- Batch 2 (US East Coast): Monday 12 PM IST
- Batch 3 (US West Coast): Monday 8 PM IST
- Batch 4 (Europe): Tuesday 12 AM IST

---

## First Comment

Hey Product Hunt! 👋

I built Bodhi Chat to prove something that doesn't exist yet: web apps securely accessing YOUR local AI models with OAuth 2.1.

**The Problem:**
Local LLM solutions (Ollama, LM Studio) let you run models privately, but you need to manually configure apps, download them, or run via Docker. You're left with generic ChatGPT-like interfaces. And they can't safely expose APIs to web apps - no granular permissions, no access control.

**What Bodhi Changes:**
• **Bodhi App** = first local LLM server with OAuth 2.1 security built-in
• Web apps request specific permissions (inference, embeddings, tools)
• You grant granular access based on scopes and roles
• Apps can only access what you authorize
• You can revoke access anytime

This opens variety of use-cases where web apps rely on AI-APIs provided by users. Apps like OpenWebUI/Open Notebook can be built with zero local install requirements, just like any other webapp.

**Bodhi Chat: The Proof of Concept**
Static site on GitHub Pages doing deep research + agentic chat with web search on YOUR local model. No backend infrastructure needed from developer's side.

**How It Works:**
1. Run Bodhi App locally
2. Load your preferred AI model
3. Web app requests specific permissions via OAuth 2.1
4. You review scopes (inference? embeddings? download-models? tools?) and grant access
5. Chat runs entirely on your machine with your model

**For Developers:**
Using bodhi-js SDK, you can build web apps that access users' local AI models with proper security. This pattern doesn't exist anywhere else in the local LLM ecosystem.

We're opening our developer platform for selected beta users.

🔗 Try demo: https://bodhiapps.github.io/chat/
🛠️ Build your own: https://developer.getbodhi.app

**I'd love your feedback:**
- Would you grant web apps access to your local LLMs with OAuth?
- Developers: what apps would you build on this platform?

Thanks for checking it out!

---

## 3-Day Pre-Launch Playbook (Friday-Sunday)

### Critical Context
**Reality:** Ideal PH prep = 2-6 weeks with 400+ supporters. With 3 days, focus on high-impact tactics with existing network.

**Key Change 2026:** Product Hunt discontinued teaser/coming soon pages in late 2025. Focus on organic community building and direct outreach.

---

### FRIDAY (72 Hours Out): Foundation Sprint

#### Hours 1-4: Launch Assets Preparation
- [ ] **Visual Content:**
  - 5-7 high-quality screenshots/GIFs showing key features
  - 1-minute demo video (crucial for developer tools)
  - Social media graphics (1200x630 for sharing)

- [ ] **Content Finalization:**
  - Product tagline (60 chars)
  - Description (under 500 chars)
  - First comment (final draft)
  - Social media copy variants

#### Hours 5-8: Network Mobilization
- [ ] **Email List Segmentation** (geographic distribution 3-5x more effective):
  - **Asia/Australia batch** - send Sunday 8 PM IST (12 hours before launch)
  - **US East Coast batch** - send Monday 12 PM IST (2 hours before launch)
  - **US West Coast batch** - send Monday 8 PM IST (7 hours after launch)
  - **Europe batch** - send Tuesday 12 AM IST (10 hours after launch)
  - **Critical:** Ask for "feedback" not "upvotes" (avoids spam filters)

- [ ] **Direct Outreach** (50-100 people):
  - Existing beta users who love product
  - GitHub stars/followers (developer audience)
  - Industry peers who've launched on PH
  - Past supporters of similar AI/developer products

**Template:**
```
Hey [Name], launching Bodhi App on Product Hunt Monday 1:31 PM IST -
local AI with OAuth 2.1 for web apps. Would love your feedback when live.
Been following your work on [specific thing]. Will send link Monday. Thanks!
```

#### Hours 9-12: Product Hunt Warm-Up
- [ ] Create/optimize PH maker profile with bio, avatar, social links
- [ ] Support 5-10 launches with meaningful comments (not just upvotes)
- [ ] Engage in PH discussions/forums related to AI/developer tools
- [ ] Build relationship with potential hunters (10k+ followers boost visibility)

---

### SATURDAY (48 Hours Out): Community Activation

#### Morning (9 AM - 12 PM IST): Twitter/X Blitz
540M monthly users, real-time engagement hub:

- [ ] **Tweet Thread**:
  - Tweet 1: "Launching on Product Hunt Monday: [problem you solve]"
  - Tweet 2: Demo GIF showing 10-second use case
  - Tweet 3: "Why we built this: [origin story, 2-3 sentences]"
  - Tweet 4: "Early supporter preview Monday 1:31 PM IST. Want in? Drop a 👋"

- [ ] **Engagement:**
  - Tag communities (#AITwitter, #DevTools, #LocalAI, #OpenSource)
  - Reply to EVERY comment personally
  - DM 20-30 engaged followers with personal ask
  - Share in developer-focused Twitter Spaces if available

#### Afternoon (2 PM - 5 PM IST): LinkedIn Strategy
B2B decision makers, 36% YoY video growth:

- [ ] **Post Format** (not article):
  - Short-form video (30-60 sec) showing Bodhi App value prop
  - Personal story: "After months building local AI inference tools..."
  - CTA: "Launching Monday on Product Hunt - feedback from [target persona]"

- [ ] **Outreach:**
  - Comment on posts from AI/developer tool founders
  - Message 10-15 connections with personal context
  - Share in relevant groups (AI, Developer Tools, Open Source)

#### Evening (6 PM - 10 PM IST): Community Platforms

- [ ] **Discord/Slack Communities:**
  - AI/ML focused servers (r/LocalLLaMA Discord, HuggingFace, etc.)
  - Developer tool communities
  - Indie hacker/maker groups (#BuildInPublic)
  - **Rule:** Contribute first, ask second - follow community norms
  - Share in #show-and-tell or #launches with context

- [ ] **Reddit** (High-Risk, High-Reward):
  - r/MachineLearning, r/LocalLLaMA, r/SideProject (check rules first)
  - Contribute to discussions, mention PH launch in comments
  - Avoid direct promotion unless explicitly allowed

- [ ] **HackerNews:**
  - If you have strong HN karma, consider Show HN post
  - Time it for Sunday evening PT (Monday morning IST) for overlap
  - Frame as technical deep-dive, not launch announcement

---

### SUNDAY (24 Hours Out): Final Mobilization

#### Morning (9 AM - 12 PM IST): Supporter Prep

- [ ] **Email Batch 1 - Asia/Australia** (send 8 PM IST):
  - Clear subject: "Bodhi App launches Product Hunt tomorrow - your feedback?"
  - Include launch time: Monday 1:31 PM IST
  - One-click link to PH profile (follow maker now)
  - Personal note: why their feedback matters

- [ ] **Confirm Hunter** (if using one):
  - Hunter with 10k+ followers boosts visibility
  - Brief on product, target audience, key talking points
  - Confirm they'll post first comment or you will

#### Afternoon (2 PM - 6 PM IST): Content Pre-Staging

- [ ] **Schedule Social Posts:**
  - **Twitter:** Monday 1:35 PM IST, 8:30 PM IST, Tuesday 3:30 AM IST (three major pushes)
  - **LinkedIn:** Monday 8:30 PM IST, Tuesday 12:30 AM IST
  - **Instagram/Facebook** if relevant audience

- [ ] **First Comment Final Review:**
  - Clear value prop in first sentence
  - Specific CTA: "Try the chat feature first"
  - Personal story/context (builds connection)
  - Question to spark discussion: "Would you grant web apps OAuth access to local LLMs?"

- [ ] **Visual Gallery Upload:**
  - Upload all screenshots/GIFs to PH (can prepare draft)
  - Test demo video plays correctly
  - Ensure all links work (demo, GitHub, developer console)

#### Evening (7 PM - 10 PM IST): Team Coordination

- [ ] **Launch Day Roles:**
  - Who monitors PH comments (respond within 5 min)
  - Who manages Twitter/LinkedIn
  - Who handles Discord/Slack/Reddit
  - Who tracks analytics and vote velocity
  - Backup person for each role

- [ ] **Tools Setup:**
  - PH mobile app notifications ON
  - Comment tracking spreadsheet
  - Social media scheduling confirmed
  - Analytics dashboard ready (track referrers, sign-ups)

- [ ] **Email Batch 2 - US East Coast** (send 12 PM IST Monday):
  - Prep email with direct PH link (will have URL by then)
  - Emphasize "just went live, would love your feedback"

- [ ] **Final Checks:**
  - All links tested (demo, GitHub, developer.getbodhi.app)
  - Product Hunt submission ready (can save draft)
  - First comment copy-pasted and ready
  - Screenshots/video uploaded and sequenced

---

### Launch Readiness Checklist

- [ ] 50-100 people personally reached out (not mass email)
- [ ] 4 email batches segmented by timezone
- [ ] Social media posts scheduled for 3 major pushes
- [ ] PH profile optimized with bio, links, avatar
- [ ] Visual gallery ready (5-7 screenshots + demo video)
- [ ] First comment finalized and tested for length
- [ ] Team roles assigned with backups
- [ ] Mobile notifications enabled
- [ ] Demo flow tested end-to-end
- [ ] Developer console sign-up funnel working

---

## Launch Day Strategy (2026 Best Practices)

### Critical Timing (Monday Launch)
**IST Timings** (Primary):
- **Launch: Monday 1:31 PM IST** (12:01 AM PT)
- **Critical window: Monday 1:31 PM - 5:31 PM IST** (first 4 hours determine ranking)
- **US East push: Monday 8:31 PM IST** (7 AM PT)
- **Afternoon push: Tuesday 3:31 AM IST** (2 PM PT Monday)
- **Final push: Tuesday 9:31 AM IST** (8 PM PT Monday)

**PT Timings** (Reference):
- Launch at 12:01 AM PT Monday
- First 4 hours: 12:01 AM - 4:00 AM PT
- Activate supporters immediately in waves

### Success Factors
- **Engagement quality > quantity** - 1 quality comment = ~3 upvotes algorithmically
- **Visual gallery** - show workflow without text, users scan images first (5-7 screenshots/GIFs)
- **Founder presence** - respond to every comment within 5 minutes throughout the day
- **Clear CTA** - drive traffic to developer.getbodhi.app for beta sign-ups
- **Ask for feedback, not upvotes** - builds genuine engagement, avoids spam filters
- **Stagger outreach** - waves not blasts (4 timezone-based email batches)
- **Steady velocity beats spikes** - algorithm favors consistent engagement over sudden bursts
- **Target metrics:** 40-60 votes/hour first 4 hours for Top 3, 20-30 for Top 10

### Identifying Early Supporters (Quality Over Quantity)

**High-Probability Targets:**
1. **Active PH Users** (accounts 3+ months old with regular engagement):
   - Search who commented on similar products (Ollama, llama.cpp, LocalAI)
   - Engage with their launches before asking for support

2. **Your Existing Network:**
   - GitHub stars/followers (developer tool = developer audience)
   - Email subscribers (even small list valuable)
   - Twitter followers who engage with your content
   - LinkedIn connections in target persona (AI/ML engineers, DevOps)

3. **AI/Developer Tool Enthusiasts:**
   - Find users who upvoted competitors on PH
   - Follow makers in AI/local LLM space
   - Discord/Slack community members who engage with your content

4. **Geographic Diversity** (3-5x better results):
   - Ensures 24-hour vote velocity
   - Asia/Australia: early momentum
   - Europe: mid-day boost
   - US East/West: peak hours coverage

**Red Flags to Avoid:**
- Brand-new PH accounts (votes removed by spam filter)
- Mass upvote requests without personal context
- Cold outreach to influencers on launch day (build relationship first)
- Vote-trading schemes (PH detects and penalizes)

### Messaging Priorities
1. **OAuth 2.1 web app ecosystem USP** - unique capability no other local LLM has (not just privacy)
2. **Granular permissions** - scopes, roles, toolset access control
3. **PoC nature** - manages expectations, builds curiosity
4. **Developer opportunity** - SDK, platform potential, no backend needed
5. **Accessible demo** - show don't tell
6. **Clear path to developer.getbodhi.app** for builders

**Key distinction:** Don't lead with privacy (Ollama/LM Studio already claim that). Lead with "web apps can securely connect to your local LLMs with OAuth 2.1" - this is what's unique.

### Reality Check & Expectations

**With 3 Days Preparation, You CAN:**
- Activate existing network effectively (50-100 engaged supporters)
- Achieve respectable Top 10-20 ranking
- Generate qualified feedback and early beta users
- Build brand visibility in AI/developer space
- Collect valuable user insights for product iteration

**You CANNOT:**
- Match teams with 6-week prep and 400+ supporter networks
- Guarantee Top 3 without pre-existing large community
- Fake organic engagement (PH algorithm detects and penalizes)
- Rely on viral luck - PH amplifies momentum, doesn't create it

**Realistic Goals:**
- **Stretch:** Top 5 product of the day (requires flawless execution + luck)
- **Realistic:** Top 10 product of the day (achievable with 50-100 quality supporters)
- **Minimum:** Top 20 + meaningful feedback (still valuable for product development)

**Success Beyond Rankings:**
- 20-50 developer sign-ups at developer.getbodhi.app
- Genuine feedback on OAuth 2.1 approach and use cases
- Connections with developers interested in building on Bodhi
- Visibility in AI/local LLM communities
- Foundation for future launches (many successful products launch 2-3 times)

**Focus:** Quality engagement, developer feedback, relationship building > vanity metrics

### Post-Launch Conversion & Follow-Up

**Day 1 (Launch Day):**
- Monitor developer.getbodhi.app sign-up funnel in real-time
- Track referrer sources (which messaging drives sign-ups)
- Respond to every comment with personalized reply (not generic "thanks")
- Note feature requests and pain points mentioned
- DM engaged commenters with deeper questions

**Day 2-3 (Tuesday-Wednesday):**
- Send thank-you email to all supporters with launch results
- Share "We hit #X on Product Hunt!" social media posts (builds credibility)
- Follow up with developers who signed up but didn't complete onboarding
- Engage with "upvoters" on Twitter/LinkedIn (many check notifications)
- Start conversations about app ideas mentioned in comments

**Week 1:**
- Publish "What we learned from PH launch" blog post
- Share feedback-driven roadmap updates
- Reach out to interested developers for 1:1 calls
- Build Discord/Slack community for Bodhi developers
- Plan next steps based on most requested features

**Metrics to Track:**
- PH ranking and final position
- Total upvotes and comments (engagement rate)
- developer.getbodhi.app sign-ups (conversion rate)
- Demo link clicks (from PH and social)
- GitHub stars increase (developer interest)
- Email list growth (future launch supporters)
- Quality of feedback (actionable insights vs. noise)

**Success Indicators Beyond Ranking:**
- 5+ developers actively building on Bodhi platform
- Clear use cases identified from community feedback
- Relationships with potential integration partners
- Content ideas for future marketing (user stories, case studies)

---

## Alternative Content Options

### Alternative Taglines (if needed)
- "Web apps meet local LLMs via OAuth 2.1 - PoC for developers" (60 chars)
- "First web app with OAuth 2.1 access to your local AI models" (60 chars)

### Alternative Names (if needed)
- "Bodhi Chat - Web Apps Accessing Local LLMs via OAuth 2.1" (57 chars)

---

## Key Resources

- Demo: https://bodhiapps.github.io/chat/
- Developer Console: https://developer.getbodhi.app
- GitHub (Chat): https://github.com/bodhiapps/chat
- GitHub (Bodhi App): https://github.com/BodhiSearch/BodhiApp
- GitHub (SDK): https://github.com/BodhiSearch/bodhi-js
- HN Discussion: https://news.ycombinator.com/item?id=46792987
- LinkedIn: https://www.linkedin.com/posts/anagri_we-just-shipped-bodhi-chat-a-demo-of-what-share-7422209769161121792-XXvc



<a href="https://www.producthunt.com/products/bodhi-chat-powered-by-bodhi-app/reviews/new?utm_source=badge-product_review&utm_medium=badge&utm_source=badge-bodhi&#0045;chat&#0045;powered&#0045;by&#0045;bodhi&#0045;app" target="_blank"><img src="https://api.producthunt.com/widgets/embed-image/v1/product_review.svg?product_id=1155365&theme=light" alt="Bodhi&#0032;Chat&#0032;&#0045;&#0032;Powered&#0032;by&#0032;Bodhi&#0032;App - PoC&#0032;for&#0032;Bodhi&#0032;Platform&#0044;&#0032;create&#0032;webapps&#0032;powered&#0032;by&#0032;Local&#0032;LLMs | Product Hunt" style="width: 250px; height: 54px;" width="250" height="54" /></a>