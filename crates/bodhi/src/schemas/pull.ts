import * as z from 'zod';

export const pullModelSchema = z.object({
  repo: z.string().min(1, 'Repository is required'),
  filename: z.string().min(1, 'Filename is required'),
});

export type PullModelFormData = z.infer<typeof pullModelSchema>;
