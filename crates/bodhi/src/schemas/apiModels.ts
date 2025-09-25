import * as z from 'zod';
import type { CreateApiModelRequest, ApiFormat } from '@bodhiapp/ts-client';

export const apiModelSetupSchema = z.object({
  api_format: z.enum(['openai', 'placeholder'] as const),
  api_key: z.string().min(1, 'API key is required'),
  base_url: z.string().url('Must be a valid URL'),
  models: z.array(z.string()).min(1, 'Select at least one model'),
  prefix: z.string().optional(),
});

export type ApiModelSetupFormData = z.infer<typeof apiModelSetupSchema>;

// Convert form data to API request format
export const convertToApiRequest = (formData: ApiModelSetupFormData): CreateApiModelRequest => ({
  api_format: formData.api_format as ApiFormat,
  api_key: formData.api_key,
  base_url: formData.base_url,
  models: formData.models,
  prefix: formData.prefix || undefined,
});

// Test prompt schema for API connectivity testing
export const testPromptSchema = z.object({
  api_key: z.string().min(1, 'API key is required'),
  base_url: z.string().url('Must be a valid URL'),
  model: z.string().min(1, 'Model is required'),
  prompt: z.string().min(1).max(30),
});

export type TestPromptFormData = z.infer<typeof testPromptSchema>;
