---
title: 'Introduction'
description: 'Bodhi App — unified AI gateway for local models, cloud APIs, and MCP tools'
order: 0
---

# Welcome to Bodhi App

Bodhi App is a unified AI gateway that brings together local LLM inference, cloud API proxying, and MCP tool integration under a single platform with built-in authentication and role-based access control.

## Key Features

- **Unified AI Gateway**
  Run local GGUF models via llama.cpp and proxy requests to cloud providers (OpenAI, OpenRouter, HuggingFace, and any OpenAI-compatible API) — all through one set of endpoints.

- **MCP Tool Integration**
  Connect to MCP servers with support for header-based and OAuth authentication (including Dynamic Client Registration). Discover tools, whitelist them per user, and execute them from the built-in playground or chat UI.

- **OpenAI + Ollama Compatible APIs**
  Drop-in replacement endpoints for `/v1/chat/completions`, `/v1/models`, and `/v1/embeddings` — compatible with any tool that supports OpenAI or Ollama API formats.

- **Built-in Chat UI with Agentic Tool Execution**
  Chat interface with streaming responses, markdown rendering, and agentic tool calling — models can invoke MCP tools mid-conversation with full tool call/result display.

- **Role-Based Auth & Access Control**
  OAuth2 PKCE authentication with four-tier role hierarchy (User, PowerUser, Manager, Admin). User access requests, app access requests with resource consent, and scoped API tokens.

- **Multi-Platform**
  Desktop apps for macOS (Intel + Apple Silicon), Windows, and Linux. Docker images for CPU (AMD64 + ARM64), CUDA, ROCm, Vulkan, MUSA, Intel, and CANN.

- **Bodhi JS SDK for Developers**
  [`@bodhiapp/bodhi-js-react`](https://www.npmjs.com/package/@bodhiapp/bodhi-js-react) — register your app, request MCP access, and call OpenAI-compatible APIs from any web application.

## Quick Start

- **[Install Bodhi App](/docs/install)** — Desktop and Docker setup with guided wizard
- **[Features](/docs/features)** — Chat, Models, MCPs, Auth, Settings
- **[Developer Guide](/docs/developer/getting-started)** — Integrate your app with the Bodhi JS SDK
- **[Docker Deployment](/docs/deployment/docker)** — Production deployment with GPU acceleration
