/**
 * Shared types + helpers for the V2 playground. The detail components produce a
 * uniform `RunState<T>` so `ResultPanel` can render readable/raw/request tabs
 * generically across Tools/Prompts/Resources/Templates.
 */

import type { McpClientPromptArg, McpClientTool, McpPromptMessage } from '@/hooks/mcps/useMcpClient';

// ── inputs / forms ─────────────────────────────────────────────

/** JSON-schema-ish shape of a tool's `inputSchema`. */
export type InputSchema = {
  type?: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  properties?: Record<string, any>;
  required?: string[];
};

/** Field-list shape (used by prompts + templates, where args are a flat list, not JSON Schema). */
export interface ArgField {
  name: string;
  description?: string;
  required?: boolean;
  placeholder?: string;
}

/**
 * Build sensible default values for an input schema's properties so the form has
 * a stable initial state and re-renders cleanly on tool switch.
 */
export function buildDefaultSchemaParams(schema: InputSchema | undefined): Record<string, unknown> {
  if (!schema?.properties) return {};
  const params: Record<string, unknown> = {};
  for (const [key, prop] of Object.entries(schema.properties)) {
    switch (prop?.type) {
      case 'boolean':
        params[key] = false;
        break;
      case 'array':
        params[key] = [];
        break;
      case 'object':
        params[key] = {};
        break;
      default:
        params[key] = '';
    }
  }
  return params;
}

/** Build default `{ [name]: '' }` for an arg field list (prompts/templates). */
export function buildDefaultFieldParams(args: ArgField[] | undefined): Record<string, string> {
  const out: Record<string, string> = {};
  for (const a of args ?? []) out[a.name] = '';
  return out;
}

/** Strip empty-string and undefined values; mirrors the legacy `cleanParams` for tool calls. */
export function cleanParams(p: Record<string, unknown>): Record<string, unknown> {
  const cleaned: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(p)) {
    if (v !== '' && v !== undefined) cleaned[k] = v;
  }
  return cleaned;
}

/** RFC 6570 level-1 substitution: only `{name}`. Unfilled vars are left as `{name}`. */
export function fillTemplate(uriTemplate: string, values: Record<string, string>): string {
  return uriTemplate.replace(/\{([^}]+)\}/g, (_m, k) => {
    const v = values[k];
    return v ? v : '{' + k + '}';
  });
}

/** Adapt a prompt's flat argument list to the playground's `ArgField` shape. */
export function promptArgsToFields(args: McpClientPromptArg[] | undefined): ArgField[] {
  return (args ?? []).map((a) => ({
    name: a.name,
    description: a.description,
    required: a.required,
  }));
}

// ── result model (readable rendering) ──────────────────────────

export type TextFormat = 'markdown' | 'plain' | 'pre';

export type Block =
  | { type: 'text'; text: string; format?: TextFormat }
  | { type: 'image'; mimeType?: string; data?: string; name?: string; w?: number; h?: number; alt?: string }
  | { type: 'resource_link'; uri: string; name?: string; description?: string; mimeType?: string }
  | { type: 'resource'; resource: { uri: string; mimeType?: string; text?: string; blob?: string } };

/** Readable model for a Tools result. */
export interface ToolReadable {
  content: Block[];
  structuredContent?: unknown;
  isError?: boolean;
}

/** Normalize `callTool`'s raw `content` (an array of wire blocks, a single block, or a string)
 *  into the readable {@link ToolReadable} shape. */
export function toToolReadable(
  content: unknown,
  structuredContent: unknown,
  isError: boolean | undefined
): ToolReadable {
  const blocks: Block[] = [];
  const items: unknown[] = Array.isArray(content) ? content : content == null ? [] : [content];
  for (const raw of items) {
    if (typeof raw === 'string') {
      blocks.push({ type: 'text', text: raw });
      continue;
    }
    if (!raw || typeof raw !== 'object') continue;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const b = raw as any;
    if (b.type === 'image') {
      blocks.push({
        type: 'image',
        mimeType: b.mimeType,
        data: b.data,
        name: b.name,
        w: b.w,
        h: b.h,
        alt: b.alt,
      });
    } else if (b.type === 'resource_link') {
      blocks.push({
        type: 'resource_link',
        uri: b.uri,
        name: b.name,
        description: b.description,
        mimeType: b.mimeType,
      });
    } else if (b.type === 'resource' && b.resource && typeof b.resource === 'object') {
      blocks.push({
        type: 'resource',
        resource: {
          uri: b.resource.uri,
          mimeType: b.resource.mimeType,
          text: b.resource.text,
          blob: b.resource.blob,
        },
      });
    } else {
      // default: treat as text
      const text = typeof b.text === 'string' ? b.text : JSON.stringify(b, null, 2);
      blocks.push({ type: 'text', text });
    }
  }
  return { content: blocks, structuredContent, isError };
}

// ── run state + result kinds ───────────────────────────────────

export type ResultKind = 'tool' | 'messages' | 'resource';

export interface RunRequest {
  method: string;
  params: Record<string, unknown>;
}

export interface ResourceReadable {
  uri: string;
  mimeType?: string;
  // text or DataView-able value for each content item; we pass through as-is
  contents: Array<{ uri: string; mimeType?: string; text?: string; blob?: string }>;
}

export interface RunDoneTool {
  phase: 'done';
  kind: 'tool';
  ok: boolean;
  data: ToolReadable;
  raw: unknown;
  request: RunRequest;
  error?: string;
  token: number;
}

export interface RunDoneMessages {
  phase: 'done';
  kind: 'messages';
  ok: boolean;
  data: { description?: string; messages: McpPromptMessage[] };
  raw: unknown;
  request: RunRequest;
  error?: string;
  token: number;
}

export interface RunDoneResource {
  phase: 'done';
  kind: 'resource';
  ok: boolean;
  data: ResourceReadable;
  raw: unknown;
  request: RunRequest;
  error?: string;
  token: number;
}

export type RunDone = RunDoneTool | RunDoneMessages | RunDoneResource;
export type RunState = { phase: 'idle' } | { phase: 'running'; request: RunRequest } | RunDone;

export function idleRun(): RunState {
  return { phase: 'idle' };
}

// ── tool helpers ───────────────────────────────────────────────

/** Friendly title (falls back to code name) for a tool. */
export function toolFriendlyTitle(t: McpClientTool): string {
  return t.title || t.annotations?.title || t.name;
}
