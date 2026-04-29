import type { LlmLibertyEnvelope } from '@bodhiapp/ts-client';

export interface ParsedEnvelopeSummary {
  provider: string;
  expiresAt: Date;
  hasRefreshToken: boolean;
}

export type EnvelopeValidation =
  | { ok: true; envelope: LlmLibertyEnvelope; summary: ParsedEnvelopeSummary }
  | { ok: false; error: string };

/**
 * Validate the JSON output of `npx @bodhiapp/llm-liberty@latest login` against
 * BodhiApp's supported envelope contract (v1.0.0, anthropic provider only).
 *
 * Returns `{ ok: true, envelope, summary }` on success, `{ ok: false, error }`
 * with a human-readable message on any failure (parse error, wrong version,
 * unsupported provider, missing required field).
 */
export function validateLlmLibertyEnvelope(text: string): EnvelopeValidation {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    return { ok: false, error: 'Invalid JSON — paste the full JSON output from llm-liberty.' };
  }

  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    return { ok: false, error: 'Envelope must be a JSON object.' };
  }
  const obj = parsed as Record<string, unknown>;

  if (obj.version !== '1.0.0') {
    return { ok: false, error: `Unsupported envelope version "${String(obj.version)}". Expected "1.0.0".` };
  }
  if (obj.provider !== 'anthropic') {
    return { ok: false, error: `Unsupported provider "${String(obj.provider)}". Only "anthropic" is supported in v1.` };
  }
  if (!obj.access_token || !obj.refresh_token || !obj.expires_at) {
    return {
      ok: false,
      error: 'Envelope is missing required token fields (access_token, refresh_token, expires_at).',
    };
  }
  if (!obj.oauth || typeof obj.oauth !== 'object') {
    return { ok: false, error: 'Envelope is missing required oauth fields.' };
  }
  if (!obj.api || typeof obj.api !== 'object') {
    return { ok: false, error: 'Envelope is missing required api fields.' };
  }

  const envelope = parsed as LlmLibertyEnvelope;
  const summary: ParsedEnvelopeSummary = {
    provider: envelope.provider,
    expiresAt: new Date(envelope.expires_at * 1000),
    hasRefreshToken: Boolean(envelope.refresh_token),
  };
  return { ok: true, envelope, summary };
}
