# LinkedIn Post: Bodhi Chat Launch

We just shipped Bodhi Chat — a demo of what happens when you bring your own AI model to any web app.

https://bodhiapps.github.io/chat/

Here's what makes it interesting:

It's a static site on GitHub Pages that talks to your local LLMs through OAuth 2.1. No API keys leave your machine. The LLM runs on your hardware (llama.cpp backend), but the web app doesn't know or care where you're running it.

We built an MVP of Exa web search into it. The LLM can search the web during conversations, with API keys managed server-side instead of exposed in the browser.

The real work went into making this easy for developers. We built `bodhi-js-react` — wrap your React app in `<BodhiProvider>`, and you get:
- OpenAI-compatible client
- OAuth flow handled
- Setup modal that guides new users through installing Bodhi App
- TypeScript interfaces that catch integration issues at compile time

The pattern: decouple the AI from the app. Your users control the model, the compute, and the costs. You build the experience.

If you're building AI-powered web apps and want to try this approach, we're working with early developers:

👉 https://developer.getbodhi.app

Links:
- Demo: https://bodhiapps.github.io/chat/
- Source: https://github.com/bodhiapps/chat
- Bodhi App: https://github.com/BodhiSearch/BodhiApp 
- npm package: https://www.npmjs.com/package/@bodhiapp/bodhi-js-react

---

**Word count: ~220 words**
**Tone: Professional builder's log, focused on developer value prop**
