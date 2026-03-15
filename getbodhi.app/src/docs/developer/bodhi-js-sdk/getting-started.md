---
title: 'Getting Started'
description: 'SDK reference for integrating React applications with Bodhi using @bodhiapp/bodhi-js-react — provider setup, hooks, models, chat, embeddings, and MCP tools'
order: 253
---

# Getting Started with Bodhi JS SDK

Integrate your React application with a local LLM server using the Bodhi JS SDK.

## Prerequisites

- **Node.js 18+**
- A React project (React 18.3+ or 19+)
- **Bodhi App** running locally (default: `http://localhost:1135`)
- Register your application at [https://developer.getbodhi.app](https://developer.getbodhi.app) to obtain an `authClientId`

## Install

```bash
npm install @bodhiapp/bodhi-js-react
```

This single package includes everything you need: React bindings, the web SDK, and core types.

## Setup BodhiProvider

Wrap your application with `BodhiProvider` in your entry point:

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

### BodhiProvider Props

| Prop             | Type                | Default                 | Description                                                                             |
| ---------------- | ------------------- | ----------------------- | --------------------------------------------------------------------------------------- |
| `authClientId`   | `string`            | --                      | Required (unless `client` is provided). Your OAuth client ID from the developer portal. |
| `basePath`       | `string`            | `'/'`                   | Base path for your app. Affects callback URL and storage isolation.                     |
| `logLevel`       | `LogLevel`          | `'warn'`                | Logging verbosity: `'debug'` \| `'info'` \| `'warn'` \| `'error'`                       |
| `handleCallback` | `boolean`           | `true`                  | Auto-handle OAuth callbacks on the callback route.                                      |
| `callbackPath`   | `string`            | `'{basePath}/callback'` | Custom OAuth callback path.                                                             |
| `modalHtmlPath`  | `string`            | --                      | Custom path to setup modal HTML.                                                        |
| `client`         | `UIClient`          | --                      | Provide a custom client instance (advanced).                                            |
| `clientConfig`   | `WebUIClientParams` | --                      | Configuration passed to auto-created `WebUIClient`.                                     |

## Check Connection and Login

Use the `useBodhi()` hook to access SDK state and actions:

```typescript
import { useBodhi } from '@bodhiapp/bodhi-js-react';

function Dashboard() {
  const {
    isOverallReady,  // true when connection + server are both ready
    isAuthenticated, // true when auth.status === 'authenticated'
    canLogin,        // true when ready and not loading
    login,
    showSetup,
    client,
  } = useBodhi();

  if (!isOverallReady) {
    return (
      <div>
        <p>Not connected to Bodhi server.</p>
        <button onClick={showSetup}>Open Setup</button>
      </div>
    );
  }

  if (!isAuthenticated) {
    return <button onClick={login} disabled={!canLogin}>Login</button>;
  }

  return <ChatInterface />;
}
```

### Key Context Properties

| Property          | Type                 | Description                                   |
| ----------------- | -------------------- | --------------------------------------------- |
| `client`          | `UIClient`           | The SDK client for API calls                  |
| `isOverallReady`  | `boolean`            | Connection and server both ready              |
| `isReady`         | `boolean`            | Client has a connection (extension or direct) |
| `isServerReady`   | `boolean`            | Backend server is operational                 |
| `isAuthenticated` | `boolean`            | User is authenticated                         |
| `canLogin`        | `boolean`            | Ready to login (not loading)                  |
| `isInitializing`  | `boolean`            | Client init in progress                       |
| `isExtension`     | `boolean`            | Using extension connection mode               |
| `isDirect`        | `boolean`            | Using direct HTTP connection mode             |
| `auth`            | `AuthState`          | Full auth state object                        |
| `clientState`     | `ClientContextState` | Full connection state object                  |
| `login(options?)` | function             | Initiate login flow                           |
| `logout()`        | function             | Log out                                       |
| `showSetup()`     | function             | Open the setup modal                          |
| `hideSetup()`     | function             | Close the setup modal                         |

## Setup Modal for Troubleshooting

When the connection is not ready, call `showSetup()` to open the guided setup modal. It walks users through:

- Platform compatibility check
- Bodhi App installation confirmation
- Extension installation (if applicable)
- Direct connection configuration
- Connection mode selection

```typescript
const { isOverallReady, showSetup } = useBodhi();

if (!isOverallReady) {
  return <button onClick={showSetup}>Troubleshoot Connection</button>;
}
```

## List Models

Models are returned as an `AsyncGenerator`. Collect them into an array:

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

## Chat Completions (Streaming)

Streaming returns an `AsyncGenerator` of chunks:

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
    // Update UI with incremental content
  }
  return fullResponse;
}
```

## Chat Completions (Non-Streaming)

Non-streaming returns a `Promise` with the complete response:

```typescript
const { client } = useBodhi();

async function chat(model: string, userMessage: string) {
  const response = await client.chat.completions.create({
    model,
    messages: [{ role: 'user', content: userMessage }],
  });
  const content = response.choices[0].message.content;
  return content;
}
```

## Embeddings

Generate vector embeddings from text:

```typescript
const { client } = useBodhi();

async function embed(model: string, text: string) {
  const response = await client.embeddings.create({
    model,
    input: text,
  });
  const embedding = response.data[0].embedding; // number[]
  return embedding;
}
```

## MCP Tool Discovery and Execution

List available MCP servers, discover their tools, and execute them:

```typescript
const { client } = useBodhi();

// List available MCP servers
const { mcps } = await client.mcps.list();

// List tools for a specific MCP server
const { tools } = await client.mcps.listTools(mcp.id);

// Execute a tool
const result = await client.mcps.executeTool(mcp.id, 'tool_name', { param: 'value' });
```

Each MCP has a `slug` identifier and a `tools_cache` array. Tools have `name`, `description`, and `input_schema` fields. See [Advanced: MCP Agentic Patterns](/docs/developer/bodhi-js-sdk/advanced#mcp-agentic-patterns) for building agentic chat loops with tool calls.

## OAuth Callback Handling

By default, `BodhiProvider` auto-handles OAuth callbacks when `handleCallback` is `true` (the default). After login, the auth server redirects to your callback URL (`{basePath}/callback`), and the provider exchanges the authorization code for tokens automatically.

**No additional setup is needed for most applications.**

For manual callback handling (e.g., in custom routing), disable auto-handling and call the method directly:

```typescript
<BodhiProvider authClientId="your-client-id" handleCallback={false}>
  <App />
</BodhiProvider>
```

Then in your callback route:

```typescript
import { useBodhi } from '@bodhiapp/bodhi-js-react';
import { isWebUIClient } from '@bodhiapp/bodhi-js-react';

function CallbackPage() {
  const { client } = useBodhi();

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const code = params.get('code');
    const state = params.get('state');

    if (code && state && isWebUIClient(client)) {
      client.handleOAuthCallback(code, state);
    }
  }, [client]);

  return <p>Processing login...</p>;
}
```

For access request callbacks (when returning from an admin review URL):

```typescript
if (isWebUIClient(client)) {
  await client.handleAccessRequestCallback(requestId);
}
```

## Next Steps

- [Advanced Patterns](/docs/developer/bodhi-js-sdk/advanced) -- Login with MCP access requests, agentic tool calling, extension SDK, error handling
- SDK source and additional documentation: [GitHub](https://github.com/BodhiSearch/bodhi-browser/tree/main/bodhi-js-sdk/docs)
