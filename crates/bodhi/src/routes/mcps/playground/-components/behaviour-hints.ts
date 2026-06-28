import type { McpClientTool, McpToolAnnotations } from '@/hooks/mcps/useMcpClient';

export type HintTone = 'safe' | 'warn' | 'danger' | 'info' | 'muted';
export type HintKey = keyof Pick<
  McpToolAnnotations,
  'readOnlyHint' | 'destructiveHint' | 'idempotentHint' | 'openWorldHint'
>;

export interface Hint {
  key: HintKey;
  label: string;
  tone: HintTone;
  icon: string;
  term: string;
  tip: string;
}

/** Fixed presentation order (matches the prototype). */
export const HINT_ORDER: HintKey[] = ['readOnlyHint', 'destructiveHint', 'idempotentHint', 'openWorldHint'];

function hintFor(key: HintKey, val: boolean): Hint {
  switch (key) {
    case 'readOnlyHint':
      return val
        ? {
            key,
            label: 'Read-only',
            tone: 'safe',
            icon: 'eye',
            term: 'readOnlyHint: true',
            tip: 'Only reads data — it won\u2019t change anything.',
          }
        : {
            key,
            label: 'Makes changes',
            tone: 'warn',
            icon: 'pencil-line',
            term: 'readOnlyHint: false',
            tip: 'Can create, update or change data.',
          };
    case 'destructiveHint':
      return val
        ? {
            key,
            label: 'Can delete',
            tone: 'danger',
            icon: 'trash-2',
            term: 'destructiveHint: true',
            tip: 'May remove or overwrite existing data.',
          }
        : {
            key,
            label: 'Non-destructive',
            tone: 'safe',
            icon: 'shield-check',
            term: 'destructiveHint: false',
            tip: 'Additive only — nothing gets removed.',
          };
    case 'idempotentHint':
      return val
        ? {
            key,
            label: 'Safe to repeat',
            tone: 'safe',
            icon: 'repeat',
            term: 'idempotentHint: true',
            tip: 'Running it again with the same input has the same effect.',
          }
        : {
            key,
            label: 'Repeats add up',
            tone: 'warn',
            icon: 'repeat-2',
            term: 'idempotentHint: false',
            tip: 'Each run may have an additional effect.',
          };
    case 'openWorldHint':
      return val
        ? {
            key,
            label: 'Reaches out',
            tone: 'info',
            icon: 'globe',
            term: 'openWorldHint: true',
            tip: 'Talks to systems beyond this workspace (e.g. the internet).',
          }
        : {
            key,
            label: 'Stays in workspace',
            tone: 'muted',
            icon: 'lock',
            term: 'openWorldHint: false',
            tip: 'Only touches this workspace\u2019s own data.',
          };
  }
}

/** Compute the friendly hints to show for a tool, in fixed order. */
export function hintsForTool(tool: Pick<McpClientTool, 'annotations'>): Hint[] {
  const a = tool.annotations;
  if (!a) return [];
  const out: Hint[] = [];
  for (const k of HINT_ORDER) {
    if (k in a && typeof a[k] === 'boolean') {
      out.push(hintFor(k, a[k] as boolean));
    }
  }
  return out;
}
