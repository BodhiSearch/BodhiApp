import * as z from "zod";

export const requestParamsSchema = z.object({
  frequency_penalty: z.coerce.number().min(-2).max(2).optional(),
  max_tokens: z.coerce.number().int().min(0).max(65535).optional(), // u16 range
  presence_penalty: z.coerce.number().min(-2).max(2).optional(),
  seed: z.coerce.number().int().optional(), // i64 range, but JS can't represent full range
  stop: z.array(z.string()).max(4).optional(),
  temperature: z.coerce.number().min(0).max(2).optional(),
  top_p: z.coerce.number().min(0).max(1).optional(),
  user: z.string().optional(),
}).partial();

export const contextParamsSchema = z.object({
  n_seed: z.coerce.number().int().min(0).max(4294967295).optional(), // u32 range
  n_threads: z.coerce.number().int().min(0).max(4294967295).optional(), // u32 range
  n_ctx: z.coerce.number().int().optional(), // i32 range
  n_parallel: z.coerce.number().int().optional(), // i32 range
  n_predict: z.coerce.number().int().optional(), // i32 range
  n_keep: z.coerce.number().int().optional(), // i32 range
}).partial();

export const createAliasSchema = z.object({
  alias: z.string().min(1, "Alias is required"),
  repo: z.string().min(1, "Repo is required"),
  filename: z.string().min(1, "Filename is required"),
  chat_template: z.string().min(1, "Chat template is required"),
  request_params: requestParamsSchema,
  context_params: contextParamsSchema,
});

export type AliasFormData = z.infer<typeof createAliasSchema>;
