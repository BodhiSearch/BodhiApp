// Thin wrapper around the official @modelcontextprotocol/sdk Streamable HTTP
// client that targets BodhiApp's MCP proxy endpoint. Exists so the SDK-compat
// spec stays focused on protocol assertions rather than transport wiring. The
// dynamic imports keep the module cheap to load in Playwright workers that do
// not exercise the SDK path (e.g. when this file is grepped by unrelated
// specs).

export const BODHI_MCP_PROXY_PATH = (mcpId) => `/bodhi/v1/apps/mcps/${mcpId}/mcp`;

/**
 * Build and connect an MCP Client against BodhiApp's proxy for the given
 * instance. The proxy accepts OAuth JWTs only — passing a `bodhiapp_` API
 * token here will cause `connect()` to reject with an HTTP 401/403 error
 * from the transport's fetch layer.
 *
 * @param {object} opts
 * @param {string} opts.serverUrl - BodhiApp base URL (no trailing slash).
 * @param {string} opts.mcpId - MCP instance UUID.
 * @param {string} opts.token - Bearer token sent as `Authorization: Bearer …`.
 * @param {{ name: string, version: string }} [opts.clientInfo]
 * @param {import('@modelcontextprotocol/sdk/types.js').ClientCapabilities} [opts.capabilities]
 *   Advertised client capabilities (e.g. `{ sampling: {}, elicitation: {}, roots: {} }`).
 *   Must be set at construction time because the server's `initialize` response
 *   depends on them (everything-mcp only registers sampling/elicitation/roots
 *   tools when the matching client capability is declared).
 * @param {(client: import('@modelcontextprotocol/sdk/client/index.js').Client) => void | Promise<void>} [opts.registerHandlers]
 *   Optional callback invoked AFTER the Client is constructed but BEFORE
 *   `connect()`. Use it to wire `client.setRequestHandler(schema, cb)` for
 *   client-side request handlers (sampling/elicitation/roots) and
 *   `client.setNotificationHandler(schema, cb)` for logging/resource-updated
 *   streams. Registering before connect guarantees the handlers exist when
 *   the server starts sending — in particular, everything-mcp's `syncRoots`
 *   fires ~350ms after `notifications/initialized`.
 * @returns {Promise<{ client: import('@modelcontextprotocol/sdk/client/index.js').Client,
 *                    transport: import('@modelcontextprotocol/sdk/client/streamableHttp.js').StreamableHTTPClientTransport }>}
 */
export async function buildMcpClient({
  serverUrl,
  mcpId,
  token,
  clientInfo,
  capabilities,
  registerHandlers,
} = {}) {
  if (!serverUrl) throw new Error('buildMcpClient: serverUrl is required');
  if (!mcpId) throw new Error('buildMcpClient: mcpId is required');
  if (!token) throw new Error('buildMcpClient: token is required');

  const { Client } = await import('@modelcontextprotocol/sdk/client/index.js');
  const { StreamableHTTPClientTransport } = await import(
    '@modelcontextprotocol/sdk/client/streamableHttp.js'
  );

  const url = new URL(`${serverUrl}${BODHI_MCP_PROXY_PATH(mcpId)}`);
  const transport = new StreamableHTTPClientTransport(url, {
    requestInit: {
      headers: { Authorization: `Bearer ${token}` },
    },
  });
  const client = new Client(clientInfo ?? { name: 'bodhiapp-sdk-compat-e2e', version: '0.0.0' }, {
    capabilities: capabilities ?? {},
  });
  if (typeof registerHandlers === 'function') {
    await registerHandlers(client);
  }
  await client.connect(transport);
  return { client, transport };
}

/**
 * Best-effort close — swallows transport errors that can happen after an
 * intentional disconnect test step. Always safe to call in teardown.
 */
export async function safeCloseMcpClient(client) {
  if (!client) return;
  try {
    await client.close();
  } catch {
    // ignore — client may already be closed or transport already torn down
  }
}

/**
 * Wait (up to `timeoutMs`) for `predicate(state)` to become truthy, where
 * `state` is arbitrary mutable caller-owned data being populated by a
 * notification handler. Polls every `intervalMs`. Resolves to `true` on
 * success; rejects with an Error that includes a snapshot of `state` on
 * timeout so the Playwright report shows what actually arrived.
 *
 * @param {() => boolean} predicate
 * @param {{ timeoutMs?: number, intervalMs?: number, label?: string, snapshot?: () => unknown }} [opts]
 */
export async function waitFor(
  predicate,
  { timeoutMs = 10000, intervalMs = 100, label = 'condition', snapshot } = {}
) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      if (predicate()) return true;
    } catch {
      // swallow predicate errors — treat as false
    }
    await new Promise((r) => setTimeout(r, intervalMs));
  }
  const snap =
    typeof snapshot === 'function'
      ? (() => {
          try {
            return JSON.stringify(snapshot());
          } catch {
            return '<unserializable>';
          }
        })()
      : '';
  throw new Error(
    `waitFor(${label}) timed out after ${timeoutMs}ms${snap ? `; state=${snap}` : ''}`
  );
}
