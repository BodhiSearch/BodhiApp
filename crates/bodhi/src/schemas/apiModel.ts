import * as z from 'zod';
import type {
  ApiFormat,
  ApiKey,
  ApiKeyUpdateAction,
  ApiModelResponse,
  CreateApiModelRequest,
  UpdateApiModelRequest,
} from '@bodhiapp/ts-client';

// API format presets for AI APIs
export const API_FORMAT_PRESETS = {
  openai: {
    name: 'OpenAI',
    baseUrl: 'https://api.openai.com/v1',
    models: [] as string[], // Models will always be fetched from API
  },
};

export type ApiFormatPreset = keyof typeof API_FORMAT_PRESETS;

// Zod schema for creating API models
export const createApiModelSchema = z
  .object({
    api_format: z.string().min(1, 'API format is required').max(20, 'API format must be less than 20 characters'),
    base_url: z.string().url('Base URL must be a valid URL').min(1, 'Base URL is required'),
    api_key: z.string().optional(),
    models: z.array(z.string().min(1, 'Model name cannot be empty')).max(20, 'Maximum 20 models allowed'),
    prefix: z.string().optional(),
    usePrefix: z.boolean().default(false),
    useApiKey: z.boolean().default(false),
    forward_all_with_prefix: z.boolean().default(false),
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
  });

// Zod schema for updating API models
export const updateApiModelSchema = z
  .object({
    api_format: z.string().min(1, 'API format is required').max(20, 'API format must be less than 20 characters'),
    base_url: z.string().url('Base URL must be a valid URL').min(1, 'Base URL is required'),
    api_key: z.string().optional(),
    models: z.array(z.string().min(1, 'Model name cannot be empty')).max(20, 'Maximum 20 models allowed'),
    prefix: z.string().optional(),
    usePrefix: z.boolean().default(false),
    useApiKey: z.boolean().default(false),
    forward_all_with_prefix: z.boolean().default(false),
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
  });

// Form data types
export type ApiModelFormData = z.infer<typeof createApiModelSchema>;
export type UpdateApiModelFormData = z.infer<typeof updateApiModelSchema>;

/**
 * Converts form data to CreateApiModelRequest for the API
 *
 * @param formData - The form data from the UI
 * @returns CreateApiModelRequest with proper ApiKey type (string | null)
 *
 * Note: When useApiKey is false, we send null (not undefined) to explicitly
 * indicate "no API key" for public APIs
 */
export const convertFormToCreateRequest = (formData: ApiModelFormData): CreateApiModelRequest => ({
  api_format: formData.api_format as ApiFormat,
  base_url: formData.base_url,
  api_key: formData.useApiKey && formData.api_key ? formData.api_key : null,
  models: formData.models,
  prefix: formData.usePrefix && formData.prefix ? formData.prefix : null,
  forward_all_with_prefix: formData.forward_all_with_prefix,
});

/**
 * Converts form data to UpdateApiModelRequest for the API
 *
 * @param formData - The form data from the UI
 * @param initialData - The original API model data (optional, used to track changes)
 * @returns UpdateApiModelRequest with ApiKeyUpdateAction
 *
 * Note: API key update logic:
 * - Checkbox checked + user typed new value → {action: 'set', value: newKey}
 * - Checkbox checked + no input + had stored key → {action: 'keep'}
 * - Checkbox checked + no input + no stored key → {action: 'set', value: null}
 * - Checkbox unchecked → {action: 'set', value: null}
 */
export const convertFormToUpdateRequest = (
  formData: UpdateApiModelFormData,
  initialData?: ApiModelResponse
): UpdateApiModelRequest => ({
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
      else if (initialData?.api_key_masked === '***') {
        return { action: 'keep' as const };
      }
      // User didn't type anything and no existing key
      else {
        return { action: 'set' as const, value: null as ApiKey };
      }
    }
    // Checkbox is unchecked - remove API key
    else {
      return { action: 'set' as const, value: null as ApiKey };
    }
  })(),
  models: formData.models,
  prefix: formData.usePrefix && formData.prefix ? formData.prefix : null,
  forward_all_with_prefix: formData.forward_all_with_prefix,
});

/**
 * Converts API response to form data for editing
 *
 * @param apiData - The API model response from the backend
 * @returns Form data with correct checkbox states
 *
 * Note: api_key_masked semantics:
 * - "***": API key is stored → checkbox CHECKED (has key)
 * - null: No API key stored → checkbox UNCHECKED (no key)
 */
export const convertApiToForm = (apiData: ApiModelResponse): ApiModelFormData => ({
  api_format: apiData.api_format,
  base_url: apiData.base_url,
  api_key: '',
  models: apiData.models,
  prefix: apiData.prefix || '',
  usePrefix: Boolean(apiData.prefix),
  useApiKey: apiData.api_key_masked != null, // "***" = has key stored, checkbox checked
  forward_all_with_prefix: apiData.forward_all_with_prefix || false,
});

/**
 * Converts API response to update form data
 *
 * @param apiData - The API model response from the backend
 * @returns Update form data with correct checkbox states
 *
 * Note: api_key_masked semantics:
 * - "***": API key is stored → checkbox CHECKED (has key)
 * - null: No API key stored → checkbox UNCHECKED (no key)
 */
export const convertApiToUpdateForm = (apiData: ApiModelResponse): UpdateApiModelFormData => ({
  api_format: apiData.api_format,
  base_url: apiData.base_url,
  api_key: '',
  models: apiData.models,
  prefix: apiData.prefix || '',
  usePrefix: Boolean(apiData.prefix),
  useApiKey: apiData.api_key_masked != null, // "***" = has key stored, checkbox checked
  forward_all_with_prefix: apiData.forward_all_with_prefix || false,
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

// Helper function to format prefixed model name for display
// The prefix should include its own separator (e.g., "azure/", "azure:", "provider-")
export const formatPrefixedModel = (model: string, prefix?: string | null): string => {
  if (!prefix) return model;
  return `${prefix}${model}`;
};
