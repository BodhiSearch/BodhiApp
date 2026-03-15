---
title: 'FAQ'
description: 'Frequently asked questions about Bodhi App'
order: 500
---

# FAQ

## General Questions

### What is Bodhi App?

Bodhi App is a unified AI gateway that combines local LLM inference (via llama.cpp), cloud API proxying, and MCP tool integration under one platform with built-in authentication and role-based access control.

### What platforms does Bodhi App support?

- **Desktop**: macOS (Intel and Apple Silicon), Windows, and Linux
- **Docker**: CPU (AMD64 + ARM64), CUDA, ROCm, Vulkan, MUSA, Intel, CANN
- **Cloud**: Deployable on any platform supporting Docker (RunPod, Railway, etc.)

### How often is Bodhi App updated?

Bodhi App is regularly updated with new features and improvements. Check [GitHub releases](https://github.com/BodhiSearch/BodhiApp/releases) for the latest.

## Setup & Configuration

### How do I become an admin?

The first user to log in via OAuth automatically receives admin privileges. Subsequent users must request access and have an admin or manager approve them.

### How do additional users get access?

1. Log in with OAuth to authenticate
2. Submit an access request at `/ui/request-access/`
3. Wait for admin/manager approval
4. Approver selects the role during approval (Admin can assign any role; Manager can assign Manager, PowerUser, or User)
5. Re-login with assigned role

See [User Access Requests](/docs/features/auth/user-access-requests) for details.

### How do I access the app settings?

Visit `/ui/settings/` to view and update configuration settings. Settings are stored in the SQLite database and changes take effect immediately without restart.

## Models & Inference

### How does Model Alias work?

A model alias defines default inference and server parameters for a model. See [Model Aliases](/docs/features/models/model-alias) for details.

### Can I use both local and remote AI models?

Yes. Bodhi App supports hybrid AI usage:
- **Local models**: Download GGUF models from HuggingFace for complete privacy
- **API Models**: Configure providers like OpenAI, OpenRouter, HuggingFace, and any OpenAI-compatible API

Switch between local and remote models in the chat interface. See [API Models](/docs/features/models/api-models).

### How do I download a model?

Go to `/ui/models/files/pull/`, provide the HuggingFace repository name and filename, and submit your request. Downloads run in the background with real-time progress tracking.

### What should I do if a model download fails?

Verify your network connection and review error messages on the Downloads page. See [Troubleshooting](/docs/troubleshooting).

### What Docker variants are available?

Seven variants optimized for different hardware:
- **CPU**: AMD64 + ARM64
- **CUDA**: NVIDIA GPU acceleration
- **ROCm**: AMD GPU acceleration
- **Vulkan**: Cross-vendor GPU support
- **MUSA**: Moore Threads GPU acceleration
- **Intel**: Intel GPU acceleration
- **CANN**: Huawei Ascend NPU acceleration

See [Docker Deployment](/docs/deployment/docker).

## MCPs

### What are MCPs?

MCP (Model Context Protocol) servers provide tools that AI models can use during conversations. Bodhi App connects to MCP servers, discovers their tools, and lets you execute them from the playground or chat UI.

### How do I set up an MCP server?

Admins configure MCP servers at `/ui/mcp-servers/` with the server URL and authentication method (none, header-based, or OAuth). Users then create MCP instances at `/ui/mcps/` to connect and whitelist tools. See [MCP Setup](/docs/features/mcps/setup).

### How do I use MCP tools in chat?

Enable MCPs from the popover in the chat header, then send a message. Models with tool calling support can invoke MCP tools mid-conversation. See [MCP Usage](/docs/features/mcps/usage).

## Access Requests

### What's the difference between user and app access requests?

- **User access requests**: New human users requesting a role to use Bodhi App. Managed at `/ui/request-access/`.
- **App access requests**: Third-party applications requesting access to your MCPs and API. Reviewed at `/ui/apps/access-requests/review`.

These are completely separate features. See [User Access Requests](/docs/features/auth/user-access-requests) and [App Access Management](/docs/features/auth/app-access-management).

## API & Developer

### How do I create an API token?

Navigate to `/ui/tokens/`, click "New API Token", select scopes, and copy the generated token immediately (it's shown only once). See [API Tokens](/docs/features/auth/api-tokens).

### How can I access the API documentation?

Interactive Swagger UI is available at `/swagger-ui` on your Bodhi App instance. See [API Reference](/docs/developer/openapi-reference).

### How do I integrate my app with Bodhi?

Register at [developer.getbodhi.app](https://developer.getbodhi.app), install `@bodhiapp/bodhi-js-react`, and follow the [Developer Getting Started](/docs/developer/getting-started) guide.

### What's the difference between session-based and API token authentication?

- **Session-based**: Browser login with OAuth and secure cookies for interactive web UI access
- **API token**: Database-backed tokens with SHA-256 hashing for programmatic API access

API tokens support scope-based permissions and can be activated/deactivated as needed.

### What are the user roles?

Hierarchical role system:
- **User**: Chat and embeddings API access
- **PowerUser**: Plus model downloads, aliases, API model configuration, API tokens
- **Manager**: Plus user management and access request approval (cannot manage Admins)
- **Admin**: Full system access

### What does the browser extension do?

The Bodhi App browser extension exposes authenticated endpoints to any webpage, enabling AI capabilities on any website. Currently supports Chrome.

## Troubleshooting & Support

### What should I do if I encounter issues?

See the [Troubleshooting](/docs/troubleshooting) page. If problems persist, reach out via [Discord](https://discord.gg/bodhiapp) or [GitHub Issues](https://github.com/BodhiSearch/BodhiApp/issues).

### How do I update Bodhi App?

**Desktop**: Download the latest installer from [releases](https://github.com/BodhiSearch/BodhiApp/releases). Settings are preserved.

**Docker**: Pull the latest image:
```bash
docker pull ghcr.io/bodhisearch/bodhiapp:latest-cpu
```
