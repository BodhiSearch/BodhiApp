---
title: 'Getting Started'
description: 'End-to-end tutorial for building apps that connect to Bodhi — from OAuth registration through API calls and MCP tool execution'
order: 251
---

# Developer Getting Started

This guide walks through building an application that connects to a user's local Bodhi instance. By the end, you will have an app that authenticates via OAuth, calls OpenAI-compatible LLM APIs, and discovers and executes MCP tools.

## Prerequisites

- A React project (React 18.3+ or 19+) with Node.js 18+
- **Bodhi App** running locally (default: `http://localhost:1135`)
- An OAuth client ID from the developer portal

## 1. Register Your OAuth App

Register your application at [https://developer.getbodhi.app](https://developer.getbodhi.app) to obtain an `authClientId`. This is the OAuth client identifier your app uses to authenticate with Bodhi users.

## 2. Install the SDK

```bash
npm install @bodhiapp/bodhi-js-react
```

This single package includes React bindings, the web SDK, and core types.

## 3. Setup BodhiProvider

Wrap your application with `BodhiProvider`:

```typescript
// App.tsx
import { BodhiProvider } from '@bodhiapp/bodhi-js-react';

function App() {
  return (
    <BodhiProvider authClientId="your-client-id">
      <YourApp />
    </BodhiProvider>
  );
}
```

The provider manages connection state, OAuth flows, and exposes the API client. See the [Bodhi JS SDK reference](/docs/developer/bodhi-js-sdk/getting-started) for the full list of `BodhiProvider` props.

### Connection Modes

The SDK connects to the Bodhi server via **direct HTTP** (primary) or through the **Bodhi Browser extension** (alternative). On first load, the SDK auto-detects: it tries direct HTTP first (lower latency), then falls back to the extension if unavailable. The selected mode is persisted for subsequent page loads.

```typescript
const { isDirect, isExtension, clientState } = useBodhi();
// clientState.mode is 'direct' | 'extension' | null
```

## 4. Check Connection and Login

Use the `useBodhi()` hook to access connection state and trigger login:

```typescript
import { useBodhi } from '@bodhiapp/bodhi-js-react';

function Dashboard() {
  const { isOverallReady, isAuthenticated, canLogin, login, showSetup } = useBodhi();

  if (!isOverallReady) {
    return <button onClick={showSetup}>Connect to Bodhi</button>;
  }

  if (!isAuthenticated) {
    return <button onClick={login} disabled={!canLogin}>Login</button>;
  }

  return <ChatInterface />;
}
```

### Login with Resource Access Requests

When your app needs access to specific MCP servers, request access during login. The user reviews and approves the request before your app gains access:

```typescript
const handleLogin = async () => {
  await login({
    requested: {
      mcp_servers: [{ url: 'http://localhost:3000/mcp' }],
    },
    onProgress: (stage) => {
      // stage: 'requesting' -> 'reviewing' -> 'authenticating'
      console.log('Login stage:', stage);
    },
  });
};
```

This triggers the [app access request flow](/docs/developer/app-access-requests). The user sees a review page where they select which MCP instances to grant your app, and which role level to approve.

## 5. Call OpenAI-Compatible APIs

Once authenticated, use `client` from the hook to make API calls. Bodhi exposes OpenAI-compatible endpoints at `/v1/chat/completions`, `/v1/models`, and `/v1/embeddings`.

### List Models

```typescript
const { client } = useBodhi();

async function fetchModels() {
  const models: string[] = [];
  for await (const model of client.models.list()) {
    models.push(model.id);
  }
  return models;
}
```

### Chat Completions (Streaming)

```typescript
const { client } = useBodhi();

async function chatWithStreaming(model: string, userMessage: string) {
  const stream = client.chat.completions.create({
    model,
    messages: [{ role: 'user', content: userMessage }],
    stream: true,
  });

  let fullResponse = '';
  for await (const chunk of stream) {
    const content = chunk.choices?.[0]?.delta?.content || '';
    fullResponse += content;
  }
  return fullResponse;
}
```

### Chat Completions (Non-Streaming)

```typescript
const response = await client.chat.completions.create({
  model: 'your-model',
  messages: [{ role: 'user', content: 'Hello!' }],
});
const content = response.choices[0].message.content;
```

### Embeddings

```typescript
const response = await client.embeddings.create({
  model: 'your-embedding-model',
  input: 'Text to embed',
});
const embedding = response.data[0].embedding; // number[]
```

For the full list of endpoints, see the [OpenAPI Reference](/docs/developer/openapi-reference).

## 6. MCP Tool Discovery and Execution

After the user approves your app's access request with MCP servers, you can list, discover, and execute tools:

```typescript
const { client } = useBodhi();

// List MCP instances accessible to your app
const { mcps } = await client.mcps.list();

// List tools for a specific MCP instance
const { tools } = await client.mcps.listTools(mcp.id);

// Execute a tool
const result = await client.mcps.executeTool(mcp.id, 'tool_name', { param: 'value' });
```

Each MCP instance has a `slug` identifier and a `tools_cache` array. Tools have `name`, `description`, and `input_schema` fields.

### Agentic Tool Calling

For building agentic chat loops where the LLM discovers and calls MCP tools autonomously, convert MCP tools to the `ChatCompletionTools` format:

```typescript
import type { ChatCompletionTools } from '@bodhiapp/bodhi-js-react/api';

const { mcps } = await client.mcps.list();

const tools: ChatCompletionTools[] = [];
for (const mcp of mcps) {
  for (const tool of mcp.tools_cache ?? []) {
    tools.push({
      type: 'function',
      function: {
        name: `mcp__${mcp.slug}__${tool.name}`,
        description: tool.description ?? '',
        parameters: tool.input_schema as Record<string, unknown>,
      },
    });
  }
}

// Pass tools to chat completions
const stream = client.chat.completions.create({
  model: 'your-model',
  messages,
  stream: true,
  tools,
});
```

The tool naming convention is `mcp__<slug>__<tool-name>`. When the LLM returns a tool call, parse the name to find the MCP instance and execute it. See the [SDK Advanced Patterns](/docs/developer/bodhi-js-sdk/advanced) guide for the complete agentic loop implementation.

## REST API Endpoints

All Bodhi-specific endpoints use the `/bodhi/v1/` prefix. External app endpoints use `/bodhi/v1/apps/`:

| Endpoint                                             | Method | Description                        |
| ---------------------------------------------------- | ------ | ---------------------------------- |
| `/v1/chat/completions`                               | POST   | OpenAI-compatible chat completions |
| `/v1/models`                                         | GET    | List available models              |
| `/v1/embeddings`                                     | POST   | Generate embeddings                |
| `/bodhi/v1/apps/request-access`                      | POST   | Create access request              |
| `/bodhi/v1/apps/access-requests/{id}`                | GET    | Poll access request status         |
| `/bodhi/v1/apps/mcps`                                | GET    | List accessible MCP instances      |
| `/bodhi/v1/apps/mcps/{id}`                           | GET    | Get MCP instance details           |
| `/bodhi/v1/apps/mcps/{id}/tools/refresh`             | POST   | Refresh MCP tool list              |
| `/bodhi/v1/apps/mcps/{id}/tools/{tool_name}/execute` | POST   | Execute an MCP tool                |

For the complete API specification, visit `/swagger-ui` on your Bodhi instance or see the [OpenAPI Reference](/docs/developer/openapi-reference).

## Next Steps

- [Bodhi JS SDK Reference](/docs/developer/bodhi-js-sdk/getting-started) -- Provider props, hook API, connection modes, OAuth callback handling
- [Advanced SDK Patterns](/docs/developer/bodhi-js-sdk/advanced) -- Agentic tool loops, extension SDK, error handling, multi-tenant
- [App Access Requests](/docs/developer/app-access-requests) -- Resource consent model, API flow details, privilege escalation rules
- [OpenAPI Reference](/docs/developer/openapi-reference) -- Interactive API docs, curl examples, CORS policy
