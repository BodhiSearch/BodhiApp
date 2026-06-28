/**
 * MSW v2 handlers for MCP Streamable HTTP protocol.
 *
 * These handlers simulate an MCP server at the protocol level, handling
 * JSON-RPC requests that the MCP SDK's StreamableHTTPClientTransport sends.
 * Uses plain JSON responses (not SSE) since the SDK accepts both formats.
 */
import { http, HttpResponse, type HttpHandler } from 'msw';

export interface MockMcpToolAnnotations {
  title?: string;
  readOnlyHint?: boolean;
  destructiveHint?: boolean;
  idempotentHint?: boolean;
  openWorldHint?: boolean;
}

export interface MockMcpTool {
  name: string;
  description?: string;
  title?: string;
  inputSchema?: Record<string, unknown>;
  annotations?: MockMcpToolAnnotations;
}

export interface MockMcpPromptArg {
  name: string;
  description?: string;
  required?: boolean;
}

export interface MockMcpPrompt {
  name: string;
  title?: string;
  description?: string;
  arguments?: MockMcpPromptArg[];
}

export interface MockMcpResource {
  uri: string;
  name: string;
  title?: string;
  description?: string;
  mimeType?: string;
}

export interface MockMcpResourceTemplate {
  uriTemplate: string;
  name: string;
  title?: string;
  description?: string;
  mimeType?: string;
}

export interface MockPromptMessage {
  role: 'user' | 'assistant';
  content: unknown;
}

export interface MockPromptGetResult {
  description?: string;
  messages: MockPromptMessage[];
}

export interface MockResourceContent {
  uri: string;
  mimeType?: string;
  text?: string;
  blob?: string;
}

export interface MockResourceReadResult {
  contents: MockResourceContent[];
}

export interface MockMcpServerConfig {
  /** The endpoint path, e.g. '/bodhi/v1/apps/mcps/test-id/mcp' */
  endpoint: string;
  /** Tools the mock server advertises via tools/list */
  tools?: MockMcpTool[];
  /** Prompts the mock server advertises via prompts/list. Pass `undefined` to NOT advertise the prompts capability. */
  prompts?: MockMcpPrompt[];
  /** Resources the mock server advertises via resources/list. Pass `undefined` to NOT advertise the resources capability. */
  resources?: MockMcpResource[];
  /** Resource templates the mock server advertises via resources/templates/list. Implies the resources capability is advertised. */
  resourceTemplates?: MockMcpResourceTemplate[];
  /** Server name reported in initialize result (default: 'mock-mcp') */
  serverName?: string;
  /** Custom handler for tools/call requests */
  toolCallHandler?: (toolName: string, args: Record<string, unknown>) => { text: string; isError?: boolean };
  /** Custom handler for prompts/get requests */
  promptGetHandler?: (name: string, args: Record<string, string>) => MockPromptGetResult;
  /** Custom handler for resources/read requests */
  resourceReadHandler?: (uri: string) => MockResourceReadResult;
}

interface JsonRpcRequest {
  jsonrpc: string;
  id?: number | string;
  method: string;
  params?: Record<string, unknown>;
}

function defaultToolCallHandler(toolName: string, _args: Record<string, unknown>): { text: string; isError?: boolean } {
  return { text: `Mock result from ${toolName}`, isError: false };
}

function defaultPromptGetHandler(name: string, args: Record<string, string>): MockPromptGetResult {
  return {
    description: `Mock prompt ${name}`,
    messages: [
      {
        role: 'user',
        content: { type: 'text', text: `Prompt ${name} with args ${JSON.stringify(args)}` },
      },
    ],
  };
}

function defaultResourceReadHandler(uri: string): MockResourceReadResult {
  return {
    contents: [
      {
        uri,
        mimeType: 'text/plain',
        text: `Mock contents of ${uri}`,
      },
    ],
  };
}

/**
 * Create MSW handlers that simulate an MCP Streamable HTTP server.
 *
 * Handles the JSON-RPC methods the MCP SDK sends:
 * - initialize -> server capabilities + Mcp-Session-Id header
 * - notifications/initialized -> 202 Accepted
 * - tools/list -> configured tool list
 * - tools/call -> tool execution result
 * - prompts/list -> configured prompt list (only if prompts/resources configured)
 * - prompts/get -> prompt result
 * - resources/list -> configured resource list
 * - resources/read -> resource read result
 * - resources/templates/list -> configured resource template list
 * - DELETE -> session close (202 Accepted)
 *
 * Unknown methods receive a JSON-RPC -32601 error; the SDK and the playground
 * hook treat that as a missing capability and fall back to empty lists.
 */
export function createMcpProtocolHandlers(config: MockMcpServerConfig): HttpHandler[] {
  const {
    endpoint,
    tools,
    prompts,
    resources,
    resourceTemplates,
    serverName = 'mock-mcp',
    toolCallHandler = defaultToolCallHandler,
    promptGetHandler = defaultPromptGetHandler,
    resourceReadHandler = defaultResourceReadHandler,
  } = config;

  const sessionId = crypto.randomUUID();

  const capabilities: Record<string, Record<string, unknown>> = { tools: {} };
  if (prompts !== undefined) capabilities.prompts = {};
  if (resources !== undefined || resourceTemplates !== undefined) capabilities.resources = {};

  const promptList = prompts ?? [];
  const resourceList = resources ?? [];
  const templateList = resourceTemplates ?? [];
  const toolList = tools ?? [];

  const postHandler = http.post(endpoint, async ({ request }) => {
    const body = (await request.json()) as JsonRpcRequest;
    const { method, id } = body;

    switch (method) {
      case 'initialize': {
        return HttpResponse.json(
          {
            jsonrpc: '2.0',
            id,
            result: {
              protocolVersion: '2025-03-26',
              capabilities,
              serverInfo: { name: serverName, version: '1.0.0' },
            },
          },
          {
            headers: { 'Mcp-Session-Id': sessionId },
          }
        );
      }

      case 'notifications/initialized': {
        return new HttpResponse(null, {
          status: 202,
          headers: { 'Mcp-Session-Id': sessionId },
        });
      }

      case 'tools/list': {
        const out = toolList.map((t) => ({
          name: t.name,
          ...(t.title !== undefined ? { title: t.title } : {}),
          description: t.description ?? '',
          inputSchema: t.inputSchema ?? { type: 'object', properties: {} },
          ...(t.annotations ? { annotations: t.annotations } : {}),
        }));

        return HttpResponse.json(
          {
            jsonrpc: '2.0',
            id,
            result: { tools: out },
          },
          {
            headers: { 'Mcp-Session-Id': sessionId },
          }
        );
      }

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

      case 'prompts/list': {
        if (prompts === undefined) {
          return HttpResponse.json(
            {
              jsonrpc: '2.0',
              id,
              error: { code: -32601, message: 'Method not found: prompts/list' },
            },
            { status: 200, headers: { 'Mcp-Session-Id': sessionId } }
          );
        }
        return HttpResponse.json(
          {
            jsonrpc: '2.0',
            id,
            result: { prompts: promptList },
          },
          { headers: { 'Mcp-Session-Id': sessionId } }
        );
      }

      case 'prompts/get': {
        const params = body.params ?? {};
        const promptName = params.name as string;
        const promptArgs = (params.arguments ?? {}) as Record<string, string>;
        const result = promptGetHandler(promptName, promptArgs);
        return HttpResponse.json(
          {
            jsonrpc: '2.0',
            id,
            result,
          },
          { headers: { 'Mcp-Session-Id': sessionId } }
        );
      }

      case 'resources/list': {
        if (resources === undefined && resourceTemplates === undefined) {
          return HttpResponse.json(
            {
              jsonrpc: '2.0',
              id,
              error: { code: -32601, message: 'Method not found: resources/list' },
            },
            { status: 200, headers: { 'Mcp-Session-Id': sessionId } }
          );
        }
        return HttpResponse.json(
          {
            jsonrpc: '2.0',
            id,
            result: { resources: resourceList },
          },
          { headers: { 'Mcp-Session-Id': sessionId } }
        );
      }

      case 'resources/read': {
        const params = body.params ?? {};
        const uri = params.uri as string;
        const result = resourceReadHandler(uri);
        return HttpResponse.json(
          {
            jsonrpc: '2.0',
            id,
            result,
          },
          { headers: { 'Mcp-Session-Id': sessionId } }
        );
      }

      case 'resources/templates/list': {
        if (resources === undefined && resourceTemplates === undefined) {
          return HttpResponse.json(
            {
              jsonrpc: '2.0',
              id,
              error: { code: -32601, message: 'Method not found: resources/templates/list' },
            },
            { status: 200, headers: { 'Mcp-Session-Id': sessionId } }
          );
        }
        return HttpResponse.json(
          {
            jsonrpc: '2.0',
            id,
            result: { resourceTemplates: templateList },
          },
          { headers: { 'Mcp-Session-Id': sessionId } }
        );
      }

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

  const deleteHandler = http.delete(endpoint, () => {
    return new HttpResponse(null, {
      status: 202,
      headers: { 'Mcp-Session-Id': sessionId },
    });
  });

  return [postHandler, getHandler, deleteHandler];
}
