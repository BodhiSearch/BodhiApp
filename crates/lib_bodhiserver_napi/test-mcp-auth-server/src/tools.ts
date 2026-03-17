import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import type { Request } from 'express';
import { z } from 'zod';

export function registerTools(server: McpServer, getLastRequest: () => Request | null): void {
  server.tool('echo', { text: z.string() }, async ({ text }) => ({
    content: [{ type: 'text' as const, text: `echo: ${text}` }],
  }));

  server.tool('get_auth_info', {}, async () => {
    const lastReq = getLastRequest();
    const headers: Record<string, string> = {};
    const query: Record<string, string> = {};

    if (lastReq) {
      // Capture all non-standard headers (skip typical HTTP headers)
      const skipHeaders = new Set([
        'host',
        'connection',
        'content-type',
        'content-length',
        'accept',
        'accept-encoding',
        'accept-language',
        'user-agent',
        'origin',
        'referer',
        'mcp-session-id',
      ]);
      for (const [key, value] of Object.entries(lastReq.headers)) {
        if (!skipHeaders.has(key) && typeof value === 'string') {
          headers[key] = value;
        }
      }

      // Capture all query params
      for (const [key, value] of Object.entries(lastReq.query)) {
        if (typeof value === 'string') {
          query[key] = value;
        }
      }
    }

    return {
      content: [
        {
          type: 'text' as const,
          text: JSON.stringify({ headers, query }),
        },
      ],
    };
  });
}
