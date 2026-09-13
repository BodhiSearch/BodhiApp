---
title: 'Introduction'
description: 'Bodhi App — unified AI gateway for local models, cloud APIs, and MCP tools'
order: 0
---

# Welcome to Bodhi App

Bodhi App is a unified AI gateway that brings together local LLM inference, cloud API proxying, and MCP tool integration under a single platform — with built-in OAuth2 authentication, role-based access control, and drop-in compatibility for the most popular AI APIs.

Run a 7B model on your laptop in the morning, route to GPT-4 or Claude in the afternoon, and let your agents call MCP tools all day — through one set of endpoints, one auth token, one UI.

## Key Features

- **Multi-format API gateway**
  Bodhi exposes the same underlying inference layer through several wire formats simultaneously:
  - **OpenAI** — `/v1/chat/completions`, `/v1/models`, `/v1/embeddings`
  - **OpenAI Responses** — `/v1/responses` (async polling) for long-running and reasoning workloads
  - **Anthropic Messages** — `/anthropic/v1/messages` with first-class Anthropic OAuth (use a Claude.ai/Console account directly, no API key needed)
  - **Gemini** — `/v1beta/*` for any tool that speaks `x-goog-api-key`
  - **Ollama** — `/api/*` (deprecated, kept for legacy clients)

- **Local + remote inference, side by side**
  Run local GGUF models via llama.cpp and proxy to remote providers (OpenAI, Anthropic, Gemini, Groq, OpenRouter, HuggingFace, or any OpenAI-compatible API). Define **model aliases** that combine a model file with inference parameters and use them interchangeably with API models in chat or via API.

- **MCP tool integration**
  Connect to Model Context Protocol servers, discover their tools, and let your models invoke them mid-conversation. Three authentication methods are supported out of the box:
  - **Header-based** — static API key or bearer token sent on every request
  - **OAuth2 with preregistered client** — for MCP servers that publish a fixed client_id
  - **OAuth2 with Dynamic Client Registration (DCR)** — for servers that implement RFC 7591/8414, no manual setup
    Includes a built-in **MCP playground** to test tools, an admin **MCP store** to publish pre-registered servers across the workspace, and an **MCP proxy** so external apps can route their own MCP traffic through Bodhi's auth gateway.

- **Built-in chat UI with agentic tool execution**
  Streaming responses, markdown rendering, tool call/result inline display, parameter and system-prompt overrides, and one-click MCP tool selection per conversation.

- **Role-based auth and access control**
  OAuth2 PKCE login with a four-tier role hierarchy (User, PowerUser, Manager, Admin). User access requests, app access requests with resource consent, and scoped API tokens for programmatic access.

- **Multi-platform**
  Desktop apps for macOS (Intel + Apple Silicon), Windows, and Linux. Docker images for CPU (AMD64 + ARM64), CUDA, ROCm, Vulkan, MUSA, Intel, and CANN.

- **Bodhi JS SDK for developers**
  [`@bodhiapp/bodhi-js-react`](https://www.npmjs.com/package/@bodhiapp/bodhi-js-react) — register your app, request MCP access, and call OpenAI-compatible APIs from any web application.

## Quick Start

- **[Install Bodhi App](/docs/install)** — Desktop and Docker setup with guided wizard
- **[Concepts → Overview](/docs/concepts/overview)** — Understand the mental model before you dive in
- **[Features](/docs/features)** — Chat, Models, MCPs, Auth, Settings
- **[API Compatibility](/docs/api-compatibility/overview)** — OpenAI / Anthropic / Gemini / Ollama drop-in usage
- **[Developer Guide](/docs/developer/getting-started)** — Integrate your app with the Bodhi JS SDK
- **[Docker Deployment](/docs/deployment/docker)** — Production deployment with GPU acceleration
