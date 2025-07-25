import { z } from 'zod';

export const setupFormSchema = z.object({
  name: z
    .string()
    .min(10, 'Server name must be at least 10 characters long')
    .max(100, 'Server name must be less than 100 characters'),
  description: z.string().max(500, 'Description must be less than 500 characters').optional().or(z.literal('')),
});

export type SetupFormData = z.infer<typeof setupFormSchema>;
