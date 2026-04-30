import type { LlmLibertyEnvelope } from '@bodhiapp/ts-client';
import { z } from 'zod';

export interface ParsedEnvelopeSummary {
  provider: string;
  expiresAt: Date;
  hasRefreshToken: boolean;
}

export type EnvelopeValidation =
  | { ok: true; envelope: LlmLibertyEnvelope; summary: ParsedEnvelopeSummary }
  | { ok: false; error: string };

// Validates only the leaves BodhiApp actually relies on at paste time:
// - top-level: version="1.0.0", provider="anthropic", non-empty tokens
// - auth.{in,key,scheme}: drive header construction; v1 hardcodes Bearer
// - oauth.{token_url,client_id}: needed for refresh-token rotation
// Not validated client-side (server is source of truth): oauth.authorize_url,
// api.{base_url,chat_url,models_url}, expires_at type. Server will surface a
// loud error if any of those are missing/malformed.
const envelopeSchema = z
  .object({
    version: z.string(),
    provider: z.string(),
    access_token: z.string().min(1, 'access_token is required.'),
    refresh_token: z.string().min(1, 'refresh_token is required.'),
    expires_at: z.unknown(), // not validated; server-side check
    auth: z.object({
      in: z.string().min(1, 'auth.in is required.'),
      key: z.string().min(1, 'auth.key is required.'),
      scheme: z.string().min(1, 'auth.scheme is required.'),
    }),
    oauth: z
      .object({
        token_url: z.string().min(1, 'oauth.token_url is required.'),
        client_id: z.string().min(1, 'oauth.client_id is required.'),
      })
      .passthrough(),
    api: z.object({}).passthrough(),
  })
  .superRefine((data, ctx) => {
    if (data.version !== '1.0.0') {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        path: ['version'],
        message: `Unsupported envelope version "${data.version}". Expected "1.0.0".`,
      });
    }
    if (data.provider !== 'anthropic') {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        path: ['provider'],
        message: `Unsupported provider "${data.provider}". Only "anthropic" is supported in v1.`,
      });
    }
  });

/**
 * Validate the JSON output of `npx @bodhiapp/llm-liberty@latest login` against
 * BodhiApp's supported envelope contract. Returns `{ ok: true, envelope, summary }`
 * on success, `{ ok: false, error }` with a human-readable message otherwise.
 */
export function validateLlmLibertyEnvelope(text: string): EnvelopeValidation {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    return { ok: false, error: 'Invalid JSON — paste the full JSON output from llm-liberty.' };
  }

  const result = envelopeSchema.safeParse(parsed);
  if (!result.success) {
    const first = result.error.issues[0];
    const path = first.path.length > 0 ? `${first.path.join('.')}: ` : '';
    return { ok: false, error: `${path}${first.message}` };
  }

  const envelope = parsed as LlmLibertyEnvelope;
  const expiresAtRaw = (parsed as { expires_at?: unknown }).expires_at;
  const expiresAtSecs = typeof expiresAtRaw === 'number' ? expiresAtRaw : 0;
  const summary: ParsedEnvelopeSummary = {
    provider: envelope.provider,
    expiresAt: new Date(expiresAtSecs * 1000),
    hasRefreshToken: Boolean(envelope.refresh_token),
  };
  return { ok: true, envelope, summary };
}
