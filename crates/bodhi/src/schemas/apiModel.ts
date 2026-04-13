import type { ApiFormat, ApiKey, ApiKeyUpdate, ApiModel, ApiModelRequest, ApiAliasResponse } from '@bodhiapp/ts-client';
import * as z from 'zod';

// API format presets for AI APIs
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
    name: 'Anthropic (Claude Code OAuth)',
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

// Zod schema for creating API models
export const createApiModelSchema = z
  .object({
    api_format: z.string().min(1, 'API format is required').max(20, 'API format must be less than 20 characters'),
    base_url: z.string().url('Base URL must be a valid URL').min(1, 'Base URL is required'),
    api_key: z.string().optional(),
    models: z.array(z.string().min(1, 'Model name cannot be empty')).max(1000, 'Maximum 1000 models allowed'),
    prefix: z.string().optional(),
    usePrefix: z.boolean().default(false),
    useApiKey: z.boolean().default(false),
    forward_all_with_prefix: z.boolean().default(false),
    extra_headers: z.string().optional(),
    extra_body: z.string().optional(),
  })
  .superRefine((data, ctx) => {
    // Only validate API key when useApiKey checkbox is checked
    if (data.useApiKey && data.api_key !== undefined) {
      if (data.api_key.length < 1) {
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
    // Validate that prefix is required when forward_all_with_prefix is true
    if (data.forward_all_with_prefix && (!data.prefix || data.prefix.trim() === '')) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        path: ['prefix'],
        message: 'Prefix is required when forwarding all requests with prefix',
      });
    }
    // When NOT using forward_all_with_prefix, require at least one model
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
    // Validate extra_headers and extra_body as valid JSON objects when non-empty
    validateJsonObjectField(data.extra_headers, 'extra_headers', ctx);
    validateJsonObjectField(data.extra_body, 'extra_body', ctx);
  });

// Zod schema for updating API models
export const updateApiModelSchema = z
  .object({
    api_format: z.string().min(1, 'API format is required').max(20, 'API format must be less than 20 characters'),
    base_url: z.string().url('Base URL must be a valid URL').min(1, 'Base URL is required'),
    api_key: z.string().optional(),
    models: z.array(z.string().min(1, 'Model name cannot be empty')).max(1000, 'Maximum 1000 models allowed'),
    prefix: z.string().optional(),
    usePrefix: z.boolean().default(false),
    useApiKey: z.boolean().default(false),
    forward_all_with_prefix: z.boolean().default(false),
    extra_headers: z.string().optional(),
    extra_body: z.string().optional(),
  })
  .superRefine((data, ctx) => {
    // Only validate API key when useApiKey checkbox is checked AND user provided a value
    // Empty api_key in update mode means "keep existing key"
    if (data.useApiKey && data.api_key !== undefined && data.api_key.trim().length > 0) {
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
    // Validate that prefix is required when forward_all_with_prefix is true
    if (data.forward_all_with_prefix && (!data.prefix || data.prefix.trim() === '')) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        path: ['prefix'],
        message: 'Prefix is required when forwarding all requests with prefix',
      });
    }
    // When NOT using forward_all_with_prefix, require at least one model
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
    // Validate extra_headers and extra_body as valid JSON objects when non-empty
    validateJsonObjectField(data.extra_headers, 'extra_headers', ctx);
    validateJsonObjectField(data.extra_body, 'extra_body', ctx);
  });

// Form data types
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

export const convertFormToCreateRequest = (formData: ApiModelFormData): ApiModelRequest => ({
  api_format: formData.api_format as ApiFormat,
  base_url: formData.base_url,
  api_key:
    formData.useApiKey && formData.api_key
      ? { action: 'set' as const, value: formData.api_key as ApiKey }
      : { action: 'set' as const, value: null },
  models: formData.models,
  prefix: formData.usePrefix && formData.prefix ? formData.prefix : null,
  forward_all_with_prefix: formData.forward_all_with_prefix,
  extra_headers: parseJsonField(formData.extra_headers),
  extra_body: parseJsonField(formData.extra_body),
});

export const convertFormToUpdateRequest = (
  formData: UpdateApiModelFormData,
  initialData?: ApiAliasResponse
): ApiModelRequest => ({
  api_format: formData.api_format as ApiFormat,
  base_url: formData.base_url,
  api_key: (() => {
    // Checkbox is checked - user wants to have an API key
    if (formData.useApiKey) {
      // User typed a new key value
      if (formData.api_key && formData.api_key.trim().length > 0) {
        return { action: 'set' as const, value: formData.api_key as ApiKey };
      }
      // User didn't type anything - keep existing key if we have one
      else if (initialData?.has_api_key) {
        return { action: 'keep' as const };
      }
      // User didn't type anything and no existing key
      else {
        return { action: 'set' as const, value: null };
      }
    }
    // Checkbox is unchecked - remove API key
    else {
      return { action: 'set' as const, value: null };
    }
  })() as ApiKeyUpdate,
  models: formData.models,
  prefix: formData.usePrefix && formData.prefix ? formData.prefix : null,
  forward_all_with_prefix: formData.forward_all_with_prefix,
  extra_headers: parseJsonField(formData.extra_headers),
  extra_body: parseJsonField(formData.extra_body),
});

export const serializeJsonField = (value: unknown | null | undefined): string => {
  if (value === null || value === undefined) return '';
  return JSON.stringify(value, null, 2);
};

export const convertApiToForm = (apiData: ApiAliasResponse): ApiModelFormData => ({
  api_format: apiData.api_format,
  base_url: apiData.base_url,
  api_key: '',
  models: apiData.models.map((m) => getApiModelId(m, apiData.prefix)),
  prefix: apiData.prefix || '',
  usePrefix: Boolean(apiData.prefix),
  useApiKey: apiData.has_api_key, // true = has key stored, checkbox checked
  forward_all_with_prefix: apiData.forward_all_with_prefix || false,
  extra_headers: serializeJsonField(apiData.extra_headers),
  extra_body: serializeJsonField(apiData.extra_body),
});

export const convertApiToUpdateForm = (apiData: ApiAliasResponse): UpdateApiModelFormData => ({
  api_format: apiData.api_format,
  base_url: apiData.base_url,
  api_key: '',
  models: apiData.models.map((m) => getApiModelId(m, apiData.prefix)),
  prefix: apiData.prefix || '',
  usePrefix: Boolean(apiData.prefix),
  useApiKey: apiData.has_api_key, // true = has key stored, checkbox checked
  forward_all_with_prefix: apiData.forward_all_with_prefix || false,
  extra_headers: serializeJsonField(apiData.extra_headers),
  extra_body: serializeJsonField(apiData.extra_body),
});

// Helper function to mask API key for display
export const maskApiKey = (apiKey: string): string => {
  if (!apiKey || apiKey.length < 10) {
    return '***';
  }
  const firstPart = apiKey.substring(0, 3);
  const lastPart = apiKey.substring(apiKey.length - 6);
  return `${firstPart}...${lastPart}`;
};

// Helper function to validate API key format
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
