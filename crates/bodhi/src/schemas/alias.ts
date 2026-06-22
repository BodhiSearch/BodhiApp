import type { UserAliasRequest } from '@bodhiapp/ts-client';
import * as z from 'zod';

import { type LocalAlias } from '@/lib/utils';

const preprocessStop = (value: unknown) => {
  if (typeof value === 'string') {
    return value
      .split(',')
      .map((item) => item.trim())
      .filter((item) => item.length > 0);
  }
  return value;
};

export const requestParamsSchema = z
  .object({
    frequency_penalty: z.coerce.number().min(-2).max(2).optional(),
    max_tokens: z.coerce.number().int().min(0).max(65535).optional(), // u16 range
    presence_penalty: z.coerce.number().min(-2).max(2).optional(),
    seed: z.coerce.number().int().optional(), // i64 range, but JS can't represent full range
    stop: z.preprocess(preprocessStop, z.array(z.string()).max(4).optional()),
    temperature: z.coerce.number().min(0).max(2).optional(),
    top_p: z.coerce.number().min(0).max(1).optional(),
    user: z.string().optional(),
  })
  .partial();

// Form schema — the form renders request_params + system_prompt as editable text; the
// conversion helpers below translate to/from the typed `OAIRequestParams` JSON the API accepts.
export const createAliasFormSchema = z.object({
  alias: z.string().min(1, 'Alias is required'),
  repo: z.string().min(1, 'Repo is required'),
  filename: z.string().min(1, 'Filename is required'),
  snapshot: z.string().optional(),
  /** OpenAI request params as `key=value` lines (parsed + validated on submit). */
  request_params_text: z.string().optional(),
  /** Stored inside request_params; surfaced in its own textarea. */
  system_prompt: z.string().optional(),
  context_params: z.string().optional(),
});

export type AliasFormData = z.infer<typeof createAliasFormSchema>;

/** The keys the form's `key=value` request-params editor accepts (mirror `requestParamsSchema`). */
const REQUEST_PARAM_KEYS = [
  'frequency_penalty',
  'max_tokens',
  'presence_penalty',
  'seed',
  'stop',
  'temperature',
  'top_p',
  'user',
] as const;

type RequestParams = z.infer<typeof requestParamsSchema>;

/** Parse `key=value\n` text into a typed (validated) OAIRequestParams object; unknown keys ignored. */
function parseRequestParamsText(text?: string): RequestParams {
  const raw: Record<string, unknown> = {};
  (text ?? '').split('\n').forEach((line) => {
    const trimmed = line.trim();
    if (!trimmed) return;
    const eq = trimmed.indexOf('=');
    if (eq < 0) return;
    const key = trimmed.slice(0, eq).trim();
    const value = trimmed.slice(eq + 1).trim();
    if ((REQUEST_PARAM_KEYS as readonly string[]).includes(key) && value !== '') {
      raw[key] = key === 'stop' ? value : value;
    }
  });
  // requestParamsSchema coerces numbers and preprocesses `stop` into a string[]; drops invalid values.
  const parsed = requestParamsSchema.safeParse(raw);
  return parsed.success ? parsed.data : {};
}

/** Render a typed OAIRequestParams (sans system_prompt) back into `key=value\n` text. */
function requestParamsToText(params?: Record<string, unknown> | null): string {
  if (!params) return '';
  return REQUEST_PARAM_KEYS.map((key) => {
    const value = (params as Record<string, unknown>)[key];
    if (value === undefined || value === null) return null;
    if (Array.isArray(value)) return value.length ? `${key}=${value.join(',')}` : null;
    return `${key}=${value}`;
  })
    .filter((line): line is string => line !== null)
    .join('\n');
}

const buildRequestParams = (formData: AliasFormData) => {
  const params = parseRequestParamsText(formData.request_params_text);
  const system_prompt = formData.system_prompt?.trim();
  return system_prompt ? { ...params, system_prompt } : params;
};

const buildContextParams = (formData: AliasFormData): string[] | undefined =>
  formData.context_params
    ? formData.context_params
        .split('\n')
        .map((line) => line.trim())
        .filter((line) => line.length > 0)
    : undefined;

export const convertFormToApi = (formData: AliasFormData): UserAliasRequest => ({
  alias: formData.alias,
  repo: formData.repo,
  filename: formData.filename,
  snapshot: formData.snapshot || null,
  request_params: buildRequestParams(formData),
  context_params: buildContextParams(formData),
});

export const convertFormToUpdateApi = (formData: AliasFormData): UserAliasRequest => convertFormToApi(formData);

export const convertApiToForm = (apiData: LocalAlias): AliasFormData => {
  const rp =
    apiData.source === 'user' && 'request_params' in apiData && apiData.request_params ? apiData.request_params : null;
  return {
    alias: apiData.alias,
    repo: apiData.repo,
    filename: apiData.filename,
    snapshot: apiData.snapshot,
    request_params_text: requestParamsToText(rp),
    system_prompt: rp?.system_prompt ?? '',
    context_params:
      apiData.source === 'user' && 'context_params' in apiData && Array.isArray(apiData.context_params)
        ? apiData.context_params.join('\n')
        : '',
  };
};
