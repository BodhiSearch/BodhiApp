# Show HN: Bodhi App – Bring your own AI model to any web app

We built Bodhi Chat (https://chat.getbodhi.app) — a static site on GitHub Pages that talks to your local LLMs through OAuth 2.1. No API keys leave your machine.

Bodhi Chat is a demo chat app that connects to Bodhi App running locally (llama.cpp backend). Auth happens via OAuth 2.1 PKCE against Bodhi's hosted identity. There's an MVP of Exa web search built in — the LLM can search the web during conversations, with API keys managed server-side instead of exposed in the browser.

The developer angle: we built `bodhi-js-react` (npm package) to make this easy. Wrap your React app in `<BodhiProvider>`, and you get an OpenAI-compatible client, OAuth flow, and a setup modal that walks new users through installing Bodhi App locally. TypeScript interfaces catch integration issues at compile time, so you know if something breaks before your users do.

Links:
- Demo: https://chat.getbodhi.app
- Bodhi Browser repo: https://github.com/bodhiapps/chat
- bodhi-js-react: https://www.npmjs.com/package/@bodhiapp/bodhi-js-react
- Developer waitlist: https://developer.getbodhi.app

If you want to build your own Bodhi-powered app, join the developer waitlist. We're working with early developers to refine the SDK.

---

**Word count: 198 words**
