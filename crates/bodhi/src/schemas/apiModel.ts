import type {
  ApiFormat,
  ApiKey,
  ApiKeyUpdate,
  ApiModel,
  ApiModelRequest,
  ApiAliasResponse,
  LlmLibertyEnvelope,
} from '@bodhiapp/ts-client';
import * as z from 'zod';

import { validateLlmLibertyEnvelope } from './llmLibertyEnvelope';

const LLM_LIBERTY_OAUTH_FORMAT = 'llm_liberty_oauth';

export const API_FORMAT_PRESETS = {
  openai: {
    name: 'OpenAI - Completions',
    baseUrl: 'https://api.openai.com/v1',
    models: [] as string[],
  },
  openai_responses: {
    name: 'OpenAI - Responses',
    baseUrl: 'https://api.openai.com/v1',
    models: [] as string[],
  },
  anthropic: {
    name: 'Anthropic',
    baseUrl: 'https://api.anthropic.com/v1',
    models: [] as string[],
  },
  anthropic_oauth: {
    name: 'Anthropic Setup Token',
    baseUrl: 'https://api.anthropic.com/v1',
    models: [] as string[],
    defaultHeaders: {
      'anthropic-version': '2023-06-01',
      'anthropic-beta': 'claude-code-20250219,oauth-2025-04-20',
      'user-agent': 'claude-cli/2.1.80 (external, cli)',
    },
    defaultBody: {
      max_tokens: 4096,
      system: [{ type: 'text', text: "You are Claude Code, Anthropic's official CLI for Claude." }],
    },
  },
  llm_liberty_oauth: {
    name: 'LLM Liberty OAuth',
    // No baseUrl — the envelope provides it; the form hides the BaseUrlInput for this format.
    baseUrl: '',
    models: [] as string[],
  },
  gemini: {
    name: 'Google Gemini',
    baseUrl: 'https://generativelanguage.googleapis.com/v1beta',
    models: [] as string[],
  },
};

export type ApiFormatPreset = keyof typeof API_FORMAT_PRESETS;

const FORBIDDEN_EXTRA_HEADER_KEYS = ['authorization', 'x-api-key', 'x-goog-api-key'];

const validateJsonObjectField = (value: string | undefined, fieldName: string, ctx: z.RefinementCtx) => {
  if (!value || value.trim() === '') return;
  let parsed: unknown;
  try {
    parsed = JSON.parse(value);
  } catch {
    ctx.addIssue({
      code: z.ZodIssueCode.custom,
      path: [fieldName],
      message: `${fieldName === 'extra_headers' ? 'Extra Headers' : 'Extra Body'} must be valid JSON`,
    });
    return;
  }
  if (typeof parsed !== 'object' || Array.isArray(parsed) || parsed === null) {
    ctx.addIssue({
      code: z.ZodIssueCode.custom,
      path: [fieldName],
      message: `${fieldName === 'extra_headers' ? 'Extra Headers' : 'Extra Body'} must be a JSON object`,
    });
    return;
  }
  if (fieldName === 'extra_headers') {
    const keys = Object.keys(parsed as Record<string, unknown>);
    const forbidden = keys.find((k) => FORBIDDEN_EXTRA_HEADER_KEYS.includes(k.toLowerCase()));
    if (forbidden) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        path: [fieldName],
        message: `Cannot have pass-through authorization headers. '${forbidden}' is not allowed in extra_headers.`,
      });
    }
  }
};

interface SchemaFlags {
  /** Envelope JSON must be present for llm_liberty_oauth (true on create, false on update). */
  envelopeRequired: boolean;
  /** API key must be non-empty when useApiKey is on (true on create, false on update where empty == keep). */
  apiKeyMinLengthRequired: boolean;
}

const baseShape = {
  name: z.string().min(1, 'Name is required').max(255, 'Name must be 255 characters or fewer'),
  api_format: z.string().min(1, 'API format is required').max(20, 'API format must be less than 20 characters'),
  base_url: z.string().optional(),
  api_key: z.string().optional(),
  models: z.array(z.string().min(1, 'Model name cannot be empty')).max(1000, 'Maximum 1000 models allowed'),
  prefix: z.string().optional(),
  usePrefix: z.boolean().default(false),
  useApiKey: z.boolean().default(false),
  forward_all_with_prefix: z.boolean().default(false),
  extra_headers: z.string().optional(),
  extra_body: z.string().optional(),
  llm_liberty_envelope: z.string().optional(),
};

function buildApiModelSchema({ envelopeRequired, apiKeyMinLengthRequired }: SchemaFlags) {
  return z.object(baseShape).superRefine((data, ctx) => {
    const isLlmLiberty = data.api_format === LLM_LIBERTY_OAUTH_FORMAT;

    if (isLlmLiberty) {
      const text = data.llm_liberty_envelope?.trim() ?? '';
      if (!text) {
        if (envelopeRequired) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            path: ['llm_liberty_envelope'],
            message: 'LLM Liberty OAuth credentials are required',
          });
        }
        // empty + !envelopeRequired = "keep existing", no further validation
      } else {
        const result = validateLlmLibertyEnvelope(text);
        if (!result.ok) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            path: ['llm_liberty_envelope'],
            message: result.error,
          });
        }
      }
    } else {
      if (!data.base_url || !data.base_url.trim()) {
        ctx.addIssue({ code: z.ZodIssueCode.custom, path: ['base_url'], message: 'Base URL is required' });
      } else {
        try {
          new URL(data.base_url);
        } catch {
          ctx.addIssue({ code: z.ZodIssueCode.custom, path: ['base_url'], message: 'Base URL must be a valid URL' });
        }
      }
      if (data.useApiKey && data.api_key !== undefined) {
        if (apiKeyMinLengthRequired && data.api_key.length < 1) {
          ctx.addIssue({
            code: z.ZodIssueCode.too_small,
            minimum: 1,
            type: 'string',
            inclusive: true,
            path: ['api_key'],
            message: 'API key must not be empty',
          });
        }
        if (data.api_key.length > 4096) {
          ctx.addIssue({
            code: z.ZodIssueCode.too_big,
            maximum: 4096,
            type: 'string',
            inclusive: true,
            path: ['api_key'],
            message: 'API key is too long (max 4096 characters)',
          });
        }
      }
      validateJsonObjectField(data.extra_headers, 'extra_headers', ctx);
      validateJsonObjectField(data.extra_body, 'extra_body', ctx);
    }

    if (data.forward_all_with_prefix && (!data.prefix || data.prefix.trim() === '')) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        path: ['prefix'],
        message: 'Prefix is required when forwarding all requests with prefix',
      });
    }
    if (!data.forward_all_with_prefix && data.models.length === 0) {
      ctx.addIssue({
        code: z.ZodIssueCode.too_small,
        minimum: 1,
        type: 'array',
        inclusive: true,
        path: ['models'],
        message: 'At least one model must be selected',
      });
    }
  });
}

export const createApiModelSchema = buildApiModelSchema({
  envelopeRequired: true,
  apiKeyMinLengthRequired: true,
});

export const updateApiModelSchema = buildApiModelSchema({
  envelopeRequired: false,
  apiKeyMinLengthRequired: false,
});

export type ApiModelFormData = z.infer<typeof createApiModelSchema>;
export type UpdateApiModelFormData = z.infer<typeof updateApiModelSchema>;

export const parseJsonField = (value: string | undefined): unknown | null => {
  if (!value || value.trim() === '') return null;
  try {
    return JSON.parse(value);
  } catch {
    return null;
  }
};

export const convertFormToCreateRequest = (formData: ApiModelFormData): ApiModelRequest => {
  if (formData.api_format === LLM_LIBERTY_OAUTH_FORMAT) {
    if (!formData.llm_liberty_envelope) {
      throw new Error('LLM Liberty OAuth credentials are required');
    }
    const envelope = JSON.parse(formData.llm_liberty_envelope) as LlmLibertyEnvelope;
    return {
      api_format: 'llm_liberty_oauth',
      name: formData.name,
      envelope: { action: 'set', value: envelope },
      models: formData.models,
      prefix: formData.usePrefix && formData.prefix ? formData.prefix : null,
      forward_all_with_prefix: formData.forward_all_with_prefix,
    };
  }

  const apiKey: ApiKeyUpdate =
    formData.useApiKey && formData.api_key
      ? { action: 'set', value: formData.api_key as ApiKey }
      : { action: 'set', value: null };

  return {
    api_format: formData.api_format as ApiFormat,
    name: formData.name,
    base_url: formData.base_url || '',
    api_key: apiKey,
    models: formData.models,
    prefix: formData.usePrefix && formData.prefix ? formData.prefix : null,
    forward_all_with_prefix: formData.forward_all_with_prefix,
    extra_headers: parseJsonField(formData.extra_headers),
    extra_body: parseJsonField(formData.extra_body),
  } as ApiModelRequest;
};

export const convertFormToUpdateRequest = (
  formData: UpdateApiModelFormData,
  initialData?: ApiAliasResponse
): ApiModelRequest => {
  if (formData.api_format === LLM_LIBERTY_OAUTH_FORMAT) {
    const envelope: import('@bodhiapp/ts-client').LlmLibertyEnvelopeUpdate =
      formData.llm_liberty_envelope && formData.llm_liberty_envelope.trim()
        ? { action: 'set', value: JSON.parse(formData.llm_liberty_envelope) as LlmLibertyEnvelope }
        : { action: 'keep' };

    return {
      api_format: 'llm_liberty_oauth',
      name: formData.name,
      envelope,
      models: formData.models,
      prefix: formData.usePrefix && formData.prefix ? formData.prefix : null,
      forward_all_with_prefix: formData.forward_all_with_prefix,
    };
  }

  const apiKey: ApiKeyUpdate = (() => {
    if (formData.useApiKey) {
      if (formData.api_key && formData.api_key.trim().length > 0) {
        return { action: 'set', value: formData.api_key as ApiKey };
      } else if (initialData?.has_api_key) {
        return { action: 'keep' };
      } else {
        return { action: 'set', value: null };
      }
    } else {
      return { action: 'set', value: null };
    }
  })();

  return {
    api_format: formData.api_format as ApiFormat,
    name: formData.name,
    base_url: formData.base_url || '',
    api_key: apiKey,
    models: formData.models,
    prefix: formData.usePrefix && formData.prefix ? formData.prefix : null,
    forward_all_with_prefix: formData.forward_all_with_prefix,
    extra_headers: parseJsonField(formData.extra_headers),
    extra_body: parseJsonField(formData.extra_body),
  } as ApiModelRequest;
};

export const serializeJsonField = (value: unknown | null | undefined): string => {
  if (value === null || value === undefined) return '';
  return JSON.stringify(value, null, 2);
};

export const convertApiToForm = (apiData: ApiAliasResponse): ApiModelFormData => ({
  name: apiData.name,
  api_format: apiData.api_format,
  base_url: apiData.base_url,
  api_key: '',
  models: apiData.models.map((m) => getApiModelId(m, apiData.prefix)),
  prefix: apiData.prefix || '',
  usePrefix: Boolean(apiData.prefix),
  useApiKey: apiData.has_api_key,
  forward_all_with_prefix: apiData.forward_all_with_prefix || false,
  extra_headers: serializeJsonField(apiData.extra_headers),
  extra_body: serializeJsonField(apiData.extra_body),
  llm_liberty_envelope: '',
});

// Schemas share the same shape; the update form differs only in the inferred type.
export const convertApiToUpdateForm = (apiData: ApiAliasResponse): UpdateApiModelFormData =>
  convertApiToForm(apiData) as UpdateApiModelFormData;

export const maskApiKey = (apiKey: string): string => {
  if (!apiKey || apiKey.length < 10) {
    return '***';
  }
  const firstPart = apiKey.substring(0, 3);
  const lastPart = apiKey.substring(apiKey.length - 6);
  return `${firstPart}...${lastPart}`;
};

export const isValidApiKey = (apiKey: string): boolean => {
  return !!apiKey && apiKey.length >= 10 && apiKey.length <= 200;
};

export const getApiModelId = (m: ApiModel, aliasPrefix?: string | null): string => {
  if (m.provider !== 'gemini') return m.id;
  // Gemini stores name as "models/{prefix}{bareId}" (prefix baked in).
  // Strip "models/" and the alias prefix to get the upstream-native bare id.
  const afterModels = m.name.startsWith('models/') ? m.name.slice('models/'.length) : m.name;
  if (aliasPrefix && afterModels.startsWith(aliasPrefix)) {
    return afterModels.slice(aliasPrefix.length);
  }
  return afterModels;
};

export const formatPrefixedModel = (model: string, prefix?: string | null): string => {
  if (!prefix) return model;
  return `${prefix}${model}`;
};
