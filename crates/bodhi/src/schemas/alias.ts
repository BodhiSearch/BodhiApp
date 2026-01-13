import type { CreateAliasRequest, UpdateAliasRequest } from '@bodhiapp/ts-client';
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

// Form schema - context_params as string for textarea
export const createAliasFormSchema = z.object({
  alias: z.string().min(1, 'Alias is required'),
  repo: z.string().min(1, 'Repo is required'),
  filename: z.string().min(1, 'Filename is required'),
  snapshot: z.string().optional(),
  request_params: requestParamsSchema,
  context_params: z.string().optional(), // string for form textarea
});

export type AliasFormData = z.infer<typeof createAliasFormSchema>;

// Conversion functions between form and API formats
export const convertFormToApi = (formData: AliasFormData): CreateAliasRequest => ({
  alias: formData.alias,
  repo: formData.repo,
  filename: formData.filename,
  snapshot: formData.snapshot || null,
  request_params: formData.request_params,
  context_params: formData.context_params
    ? formData.context_params
        .split('\n')
        .map((line) => line.trim())
        .filter((line) => line.length > 0)
    : undefined,
});

export const convertFormToUpdateApi = (formData: AliasFormData): UpdateAliasRequest => ({
  repo: formData.repo,
  filename: formData.filename,
  snapshot: formData.snapshot || null,
  request_params: formData.request_params,
  context_params: formData.context_params
    ? formData.context_params
        .split('\n')
        .map((line) => line.trim())
        .filter((line) => line.length > 0)
    : undefined,
});

export const convertApiToForm = (apiData: LocalAlias): AliasFormData => ({
  alias: apiData.alias,
  repo: apiData.repo,
  filename: apiData.filename,
  snapshot: apiData.snapshot,
  request_params:
    apiData.source === 'user' && 'request_params' in apiData && apiData.request_params
      ? {
          frequency_penalty: apiData.request_params.frequency_penalty ?? undefined,
          max_tokens: apiData.request_params.max_tokens ?? undefined,
          presence_penalty: apiData.request_params.presence_penalty ?? undefined,
          seed: apiData.request_params.seed ?? undefined,
          stop: apiData.request_params.stop,
          temperature: apiData.request_params.temperature ?? undefined,
          top_p: apiData.request_params.top_p ?? undefined,
          user: apiData.request_params.user ?? undefined,
        }
      : {},
  context_params:
    apiData.source === 'user' && 'context_params' in apiData && Array.isArray(apiData.context_params)
      ? apiData.context_params.join('\n')
      : '',
});
