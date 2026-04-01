/**
 * MSW v2 handlers for MCP Streamable HTTP protocol.
 *
 * These handlers simulate an MCP server at the protocol level, handling
 * JSON-RPC requests that the MCP SDK's StreamableHTTPClientTransport sends.
 * Uses plain JSON responses (not SSE) since the SDK accepts both formats.
 */
import { http, HttpResponse, type HttpHandler } from 'msw';

// ============================================================================
// Types
// ============================================================================

export interface MockMcpTool {
  name: string;
  description?: string;
  inputSchema?: Record<string, unknown>;
}

export interface MockMcpServerConfig {
  /** The endpoint path, e.g. '/bodhi/v1/apps/mcps/test-id/mcp' */
  endpoint: string;
  /** Tools the mock server advertises via tools/list */
  tools?: MockMcpTool[];
  /** Server name reported in initialize result (default: 'mock-mcp') */
  serverName?: string;
  /** Custom handler for tools/call requests */
  toolCallHandler?: (toolName: string, args: Record<string, unknown>) => { text: string; isError?: boolean };
}

interface JsonRpcRequest {
  jsonrpc: string;
  id?: number | string;
  method: string;
  params?: Record<string, unknown>;
}

// ============================================================================
// Default tool call handler
// ============================================================================

function defaultToolCallHandler(toolName: string, _args: Record<string, unknown>): { text: string; isError?: boolean } {
  return { text: `Mock result from ${toolName}`, isError: false };
}

// ============================================================================
// Handler Factory
// ============================================================================

/**
 * Create MSW handlers that simulate an MCP Streamable HTTP server.
 *
 * Handles the JSON-RPC methods the MCP SDK sends:
 * - initialize -> server capabilities + Mcp-Session-Id header
 * - notifications/initialized -> 202 Accepted
 * - tools/list -> configured tool list
 * - tools/call -> tool execution result
 * - DELETE -> session close (202 Accepted)
 */
export function createMcpProtocolHandlers(config: MockMcpServerConfig): HttpHandler[] {
  const { endpoint, tools = [], serverName = 'mock-mcp', toolCallHandler = defaultToolCallHandler } = config;

  const sessionId = crypto.randomUUID();

  const postHandler = http.post(endpoint, async ({ request }) => {
    const body = (await request.json()) as JsonRpcRequest;
    const { method, id } = body;

    switch (method) {
      // ----------------------------------------------------------------
      // initialize — return server capabilities with session header
      // ----------------------------------------------------------------
      case 'initialize': {
        return HttpResponse.json(
          {
            jsonrpc: '2.0',
            id,
            result: {
              protocolVersion: '2025-03-26',
              capabilities: { tools: {} },
              serverInfo: { name: serverName, version: '1.0.0' },
            },
          },
          {
            headers: { 'Mcp-Session-Id': sessionId },
          }
        );
      }

      // ----------------------------------------------------------------
      // notifications/initialized — acknowledge with 202
      // ----------------------------------------------------------------
      case 'notifications/initialized': {
        return new HttpResponse(null, {
          status: 202,
          headers: { 'Mcp-Session-Id': sessionId },
        });
      }

      // ----------------------------------------------------------------
      // tools/list — return configured tool list
      // ----------------------------------------------------------------
      case 'tools/list': {
        const toolList = tools.map((t) => ({
          name: t.name,
          description: t.description ?? '',
          inputSchema: t.inputSchema ?? { type: 'object', properties: {} },
        }));

        return HttpResponse.json(
          {
            jsonrpc: '2.0',
            id,
            result: { tools: toolList },
          },
          {
            headers: { 'Mcp-Session-Id': sessionId },
          }
        );
      }

      // ----------------------------------------------------------------
      // tools/call — execute tool via handler
      // ----------------------------------------------------------------
      case 'tools/call': {
        const params = body.params ?? {};
        const toolName = params.name as string;
        const toolArgs = (params.arguments ?? {}) as Record<string, unknown>;
        const result = toolCallHandler(toolName, toolArgs);

        return HttpResponse.json(
          {
            jsonrpc: '2.0',
            id,
            result: {
              content: [{ type: 'text', text: result.text }],
              isError: result.isError ?? false,
            },
          },
          {
            headers: { 'Mcp-Session-Id': sessionId },
          }
        );
      }

      // ----------------------------------------------------------------
      // Unknown method — return JSON-RPC method not found error
      // ----------------------------------------------------------------
      default: {
        return HttpResponse.json(
          {
            jsonrpc: '2.0',
            id,
            error: { code: -32601, message: `Method not found: ${method}` },
          },
          {
            status: 200,
            headers: { 'Mcp-Session-Id': sessionId },
          }
        );
      }
    }
  });

  // GET handler — the SDK may send GET requests for SSE session resumption;
  // return 405 Method Not Allowed (the spec-compliant response for servers
  // that only support POST-based Streamable HTTP), which suppresses MSW
  // "unhandled request" warnings.
  const getHandler = http.get(endpoint, () => {
    return new HttpResponse(null, {
      status: 405,
      headers: { 'Mcp-Session-Id': sessionId },
    });
  });

  // DELETE handler for session close
  const deleteHandler = http.delete(endpoint, () => {
    return new HttpResponse(null, {
      status: 202,
      headers: { 'Mcp-Session-Id': sessionId },
    });
  });

  return [postHandler, getHandler, deleteHandler];
}
