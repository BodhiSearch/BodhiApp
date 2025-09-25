import * as z from 'zod';

// Test prompt schema for API connectivity testing
export const testPromptSchema = z.object({
  api_key: z.string().min(1, 'API key is required'),
  base_url: z.string().url('Must be a valid URL'),
  model: z.string().min(1, 'Model is required'),
  prompt: z.string().min(1).max(30),
});

export type TestPromptFormData = z.infer<typeof testPromptSchema>;
