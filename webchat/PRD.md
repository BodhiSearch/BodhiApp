# Bodhi Web Chat – Product Requirements Document (PRD)

## 1. Overview
Bodhi Web Chat is a minimal, responsive web application that enables users to chat with an LLM (OpenAI-compatible) by providing their own API key. The app is built with Vite, React, TailwindCSS, and Shadcn UI, with all LLM interactions happening client-side.

---

## 2. Core Features (MVP)

### 2.1. Chat Interface
- Simple chat window for user prompts and LLM responses.
- Supports streaming responses from the LLM.
- In-memory conversation history (cleared on reload).

### 2.2. API Key Handling
- Modal dialog prompts user to enter their OpenAI-compatible API key.
- API key is stored in-memory only (not persisted).

### 2.3. Model & Prompt Configuration
- User can select from available models (if supported by their key).
- User can configure system prompt.
- User can adjust temperature setting.

### 2.4. Basic Chat Settings
- Access to chat settings for API key and temperature adjustment.

### 2.5. Copy Response
- User can copy LLM response to clipboard.

### 2.6. Theming & Responsiveness
- Light/dark mode toggle (via TailwindCSS/Shadcn).
- Fully responsive for mobile and desktop.

### 2.7. Error Handling
- Basic error alerts for failed API calls, invalid keys, and network issues.

---

## 3. Nice-to-Have / Low Priority Features

- Edit chat (low priority).
- Prompt templates (low priority).
- Chat export (not in MVP).
- Multi-chat/folders/renaming (not in MVP).
- PWA support (not in MVP).
- Advanced analytics, accessibility, or compliance (not in MVP).

---

## 4. Non-Features (Explicitly Out of Scope for MVP)
- Backend server or chat persistence.
- Any authentication or user accounts.
- Storing API keys or chat data.
- Advanced user settings beyond API key and temperature.
- Chat folders, renaming, or multi-chat.
- PWA or installable app features.

---

## 5. Technical Stack
- Vite + React (no Next.js)
- TailwindCSS & Shadcn UI
- No backend (frontend-only, BYOK model)
