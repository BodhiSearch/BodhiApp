Here's your reusable brief:

---

## Bodhi App — Product & Design Brief

**What it is:** Bodhi App is an open-source, unified AI gateway that lets users run local LLMs (via llama.cpp/GGUF), connect to cloud AI APIs (OpenAI-compatible), and integrate MCP (Model Context Protocol) tool servers — all through a single, privacy-first interface.

**Core USP:** Secure OAuth2+JWT system that allows third-party apps to connect and access the user's LLM and MCP resources with role-based permissions and explicit user-controlled consent. This is what differentiates Bodhi from simple local model runners.

**Brand Identity:**
- Name: Bodhi (बोधि) = enlightenment/wisdom in Sanskrit
- Logo: Lotus flower — symbolizes knowledge rising from complexity (Indian cultural/mythological symbolism). Current logo is a placeholder emoji; final will carry the same backstory.
- Brand adjectives: wise, generous, trustworthy, helpful, mentor-like
- Tone: warm and approachable, NOT cold/corporate, NOT flashy

**Audience:** Both solo developers/tinkerers and enterprise teams. UI philosophy: consumer-app approachability on the surface, with power-user advanced options revealed progressively on demand.

**Key Features:**
- Built-in Chat UI with markdown, streaming (SSE), thinking model support
- Model management: local GGUF aliases + cloud API models in one unified list
- MCP tool integration with agentic mid-conversation tool calling
- Role-based access control: 4 levels (User, PowerUser, Manager, Admin)
- API token management (scope-based, SHA-256, DB-backed)
- App access request workflow: third-party apps request consent, user/admin approves
- 12+ inference parameters: temperature, top-p, seed, stop words, frequency penalty, etc.
- Docker variants: CPU, CUDA, ROCm, Vulkan, MUSA, Intel, CANN

**Tech Stack:**
- Frontend: React + Vite, Tailwind CSS
- Backend: Rust
- Desktop: Tauri (macOS Apple Silicon & Intel, Windows, Linux)
- Cloud/Docker: available at cloud.getbodhi.app
- GitHub: github.com/BodhiSearch/BodhiApp (public, open source)

**Current UI State (as of April 2026):**
- Placeholder 2-color design: flat purple accent (#6D28D9 range) + gray backgrounds
- Layout: left sidebar (chat history/nav), center (main content), right collapsible settings panel
- Key screens: Chat, Models, API Tokens, App Settings, User Management, MCP Setup, Model Downloads

**Design System Goals:**
- Fresh color palette inspired by the lotus (warm saffron/amber or lotus-pink primary + deep indigo/midnight teal for trust/depth). Avoid generic blue SaaS look.
- Light mode: clean, open, "morning light on water" feel
- Dark mode: deep, focused, "meditation space" feel — not harsh
- Progressive disclosure: simple clean default surfaces that expand to reveal advanced controls
- Design inspiration: polish of Linear/Raycast for power users + approachability of a consumer app
- Component needs: toggles, sliders, sortable tables, badges/pills, code blocks, expandable cards, breadcrumbs, modals, info banners, pagination, inputs with help tooltips, sidebar nav
- Typography: humanist sans-serif for UI + monospace for code/tokens/API values

---

Save this and paste it at the start of any future Claude conversation to instantly restore full context. You can also add it as a Project Document in a claude.ai Project so it's always included automatically.