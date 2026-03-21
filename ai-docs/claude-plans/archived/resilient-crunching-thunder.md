# Plan: Show HN Post for Bodhi Chat Demo

## Output

A single Show HN post (title + body, ~200 words) to be delivered as text.

## Title Options (pick one)

- **Show HN: Bodhi App – Bring your own AI model to any web app**
- **Show HN: Bodhi Chat – A web app powered by your local LLMs with OAuth 2.1**

## Post Body Structure

1. **Hook** (1-2 sentences): Static site on GitHub Pages that talks to your local LLMs through OAuth 2.1. No API keys leave your machine.

2. **What it is** (2-3 sentences): Bodhi Chat is a demo chat app. It connects to Bodhi App running locally (llama.cpp backend). Auth via OAuth 2.1 PKCE against Bodhi's hosted identity. MVP of Exa web search built in — LLM can search the web during conversations, with API keys managed server-side.

3. **Developer angle** (2-3 sentences): Built with `bodhi-js-react` — wrap your React app in `<BodhiProvider>`, get OpenAI-compatible client, auth, and a setup modal that guides new users through installing Bodhi App. TypeScript interfaces catch integration issues at compile time.

4. **Links**: Demo, bodhi-browser repo, bodhi-js-react package, developer waitlist (developer.getbodhi.app)

5. **CTA**: Developer waitlist for building your own Bodhi-powered apps.

## Key Messaging Constraints

- Builder's log tone, no marketing speak
- Frame Exa/deep research as MVP/early capability
- Emphasize `bodhi-js-react` DX as the highlight
- Exa angle: API key security (keys stay on local server), not pluggability
- developer.getbodhi.app referenced as waitlist for early developers
- ~200 words, prose only, no diagrams

## Verification

- Read post aloud for HN tone check
- Confirm all links are correct before posting
- Word count ~200
