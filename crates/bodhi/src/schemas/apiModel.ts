import * as z from 'zod';
import type {
  ApiModelResponse,
  CreateApiModelRequest,
  UpdateApiModelRequest,
  TestPromptRequest,
  FetchModelsRequest,
  FetchModelsResponse,
} from '@bodhiapp/ts-client';

// Re-export generated types for use throughout the app
export type {
  ApiModelResponse,
  CreateApiModelRequest,
  UpdateApiModelRequest,
  TestPromptRequest,
  FetchModelsRequest,
  FetchModelsResponse,
};

// Provider presets for AI APIs
export const PROVIDER_PRESETS = {
  openai: {
    name: 'OpenAI',
    baseUrl: 'https://api.openai.com/v1',
    models: [] as string[], // Models will always be fetched from API
  },
};

export type ProviderPreset = keyof typeof PROVIDER_PRESETS;

// Zod schema for creating API models
export const createApiModelSchema = z.object({
  id: z
    .string()
    .min(3, 'ID must be at least 3 characters long')
    .max(50, 'ID must be less than 50 characters')
    .regex(
      /^[a-zA-Z0-9][a-zA-Z0-9_-]*$/,
      'ID must start with alphanumeric character and contain only letters, numbers, hyphens, and underscores'
    ),
  provider: z.string().min(1, 'Provider is required').max(20, 'Provider must be less than 20 characters'),
  base_url: z.string().url('Base URL must be a valid URL').min(1, 'Base URL is required'),
  api_key: z.string().min(10, 'API key must be at least 10 characters long').max(200, 'API key is too long'),
  models: z
    .array(z.string().min(1, 'Model name cannot be empty'))
    .min(1, 'At least one model must be selected')
    .max(20, 'Maximum 20 models allowed'),
});

// Zod schema for updating API models
export const updateApiModelSchema = z.object({
  provider: z.string().min(1, 'Provider is required').max(20, 'Provider must be less than 20 characters').optional(),
  base_url: z.string().url('Base URL must be a valid URL').min(1, 'Base URL is required').optional(),
  api_key: z.string().min(10, 'API key must be at least 10 characters long').max(200, 'API key is too long').optional(),
  models: z
    .array(z.string().min(1, 'Model name cannot be empty'))
    .min(1, 'At least one model must be selected')
    .max(20, 'Maximum 20 models allowed')
    .optional(),
});

// Form data types
export type ApiModelFormData = z.infer<typeof createApiModelSchema>;
export type UpdateApiModelFormData = z.infer<typeof updateApiModelSchema>;

// Conversion functions between form and API formats
export const convertFormToCreateRequest = (formData: ApiModelFormData): CreateApiModelRequest => ({
  id: formData.id,
  provider: formData.provider,
  base_url: formData.base_url,
  api_key: formData.api_key,
  models: formData.models,
});

export const convertFormToUpdateRequest = (formData: UpdateApiModelFormData): UpdateApiModelRequest => ({
  provider: formData.provider,
  base_url: formData.base_url,
  api_key: formData.api_key || undefined,
  models: formData.models,
});

export const convertApiToForm = (apiData: ApiModelResponse): ApiModelFormData => ({
  id: apiData.id,
  provider: apiData.provider,
  base_url: apiData.base_url,
  api_key: '', // API key is masked, will be empty for edit forms
  models: apiData.models,
});

export const convertApiToUpdateForm = (apiData: ApiModelResponse): UpdateApiModelFormData => ({
  provider: apiData.provider,
  base_url: apiData.base_url,
  api_key: '', // API key is masked, will be empty for edit forms
  models: apiData.models,
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

// Helper function to get provider preset
export const getProviderPreset = (provider: string): (typeof PROVIDER_PRESETS)[ProviderPreset] | null => {
  const preset = Object.entries(PROVIDER_PRESETS).find(
    ([key, value]) => key === provider.toLowerCase() || value.name === provider
  );
  return preset ? preset[1] : null;
};
