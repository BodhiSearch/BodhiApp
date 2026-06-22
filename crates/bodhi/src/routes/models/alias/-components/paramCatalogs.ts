/** Catalog entries for the click-to-add panels on the local-model form (context flags + request params). */
export interface CatalogEntry {
  key: string;
  type: 'int' | 'float' | 'bool' | 'enum' | 'flag' | 'string';
  range: string;
  desc: string;
}

/** llama-server runtime flags → `context_params` (one `--flag value` per line). */
export const RUNTIME_FLAGS: CatalogEntry[] = [
  { key: '--ctx-size', type: 'int', range: '512 – 131072', desc: 'Context window size in tokens.' },
  {
    key: '--flash-attn',
    type: 'enum',
    range: 'auto | true | false',
    desc: 'Flash attention — faster on supported hardware.',
  },
  { key: '--parallel', type: 'int', range: '1 – 64', desc: 'Number of parallel request slots.' },
  { key: '--cont-batching', type: 'bool', range: 'true | false', desc: 'Continuous batching for better throughput.' },
  {
    key: '--cache-type-k',
    type: 'enum',
    range: 'f16 | q8_0 | q4_0',
    desc: 'KV cache quant type for keys. Saves VRAM.',
  },
  { key: '--cache-type-v', type: 'enum', range: 'f16 | q8_0 | q4_0', desc: 'KV cache quant type for values.' },
  { key: '--rope-scaling', type: 'enum', range: 'none | linear | yarn', desc: 'RoPE scaling for context extension.' },
  { key: '--cache-prompt', type: 'bool', range: 'true | false', desc: 'Cache system prompt KV across requests.' },
  { key: '--grp-attn-n', type: 'int', range: '1 – 16', desc: 'Group attention factor for YaRN.' },
  { key: '--n-predict', type: 'int', range: '-1 – 32768', desc: 'Max tokens to generate. -1 = unlimited.' },
  { key: '--n-batch', type: 'int', range: '1 – 2048', desc: 'Logical batch size for token evaluation.' },
  { key: '--ubatch-size', type: 'int', range: '1 – 2048', desc: 'Physical batch size for prompt processing.' },
  { key: '--keep', type: 'int', range: '0 – ctx-size', desc: 'Tokens kept from initial prompt on reset.' },
  { key: '--mmap', type: 'bool', range: 'true | false', desc: 'Memory-map model file for faster load.' },
  { key: '--mlock', type: 'bool', range: 'true | false', desc: 'Lock model in RAM to prevent swapping.' },
  { key: '--split-mode', type: 'enum', range: 'none | layer | row', desc: 'How to split model across multiple GPUs.' },
  { key: '--n-gpu-layers', type: 'int', range: '0 – max', desc: 'Layers to offload to GPU. -1 = all.' },
  { key: '--main-gpu', type: 'int', range: '0 – N', desc: 'Primary GPU index for tensor ops.' },
  { key: '--no-warmup', type: 'flag', range: '—', desc: 'Skip warm-up on server start.' },
  {
    key: '--chat-template',
    type: 'string',
    range: 'chatml | llama2 | …',
    desc: 'Override auto-detected chat template.',
  },
];

/** Default value used when a flag is appended via the catalog. */
export function defaultFlagValue(type: CatalogEntry['type']): string {
  switch (type) {
    case 'bool':
      return 'true';
    case 'flag':
      return '';
    case 'int':
      return '0';
    case 'float':
      return '0.0';
    default:
      return '<value>';
  }
}

/** Append `--flag value\n` (or just `--flag\n` for valueless flags) to a context-params textarea. */
export function appendFlagLine(current: string, entry: CatalogEntry): string {
  const base = current.endsWith('\n') || current === '' ? current : `${current}\n`;
  const value = defaultFlagValue(entry.type);
  const line = entry.type === 'flag' ? `${entry.key}\n` : `${entry.key} ${value}\n`;
  return base + line;
}

/** Flag keys already present in the textarea (matched on the `--flag` token at line start). */
export function flagKeysInText(text: string): Set<string> {
  const keys = new Set<string>();
  text.split('\n').forEach((line) => {
    const m = line.trim().match(/^(--[\w-]+)/);
    if (m) keys.add(m[1]);
  });
  return keys;
}

/**
 * OpenAI-compatible request params → `request_params` (one `key=value` per line). Only the 8 params
 * the backend actually stores are listed (system_prompt has its own textarea, not the catalog).
 */
export const REQUEST_PARAMS: CatalogEntry[] = [
  { key: 'temperature', type: 'float', range: '0.0 – 2.0', desc: 'Sampling temperature. Lower = more deterministic.' },
  { key: 'top_p', type: 'float', range: '0.0 – 1.0', desc: 'Nucleus sampling probability mass.' },
  { key: 'max_tokens', type: 'int', range: '1 – ctx-size', desc: 'Maximum tokens in the completion.' },
  { key: 'seed', type: 'int', range: '-1 | int', desc: 'Reproducibility seed.' },
  { key: 'frequency_penalty', type: 'float', range: '-2.0 – 2.0', desc: 'Penalise tokens by frequency so far.' },
  { key: 'presence_penalty', type: 'float', range: '-2.0 – 2.0', desc: 'Penalise tokens that appeared at all.' },
  { key: 'stop', type: 'string', range: 'comma-separated', desc: 'Stop sequences — halt generation on match.' },
  { key: 'user', type: 'string', range: '<string>', desc: 'End-user ID for abuse tracking.' },
];

/** Append `key=value\n` to a request-params textarea. */
export function appendParamLine(current: string, entry: CatalogEntry): string {
  const base = current.endsWith('\n') || current === '' ? current : `${current}\n`;
  const value = entry.type === 'int' ? '0' : entry.type === 'float' ? '0.0' : '';
  return `${base}${entry.key}=${value}\n`;
}

/** Param keys already present in a `key=value` textarea. */
export function paramKeysInText(text: string): Set<string> {
  const keys = new Set<string>();
  text.split('\n').forEach((line) => {
    const m = line.trim().match(/^([\w_]+)=/);
    if (m) keys.add(m[1]);
  });
  return keys;
}
