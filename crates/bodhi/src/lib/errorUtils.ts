import { BodhiErrorResponse } from '@bodhiapp/ts-client';
import { AxiosError } from 'axios';

type MaybeAxios = AxiosError<BodhiErrorResponse> | { message?: string } | unknown;

function asBodhiError(error: unknown): BodhiErrorResponse['error'] | undefined {
  const data = (error as AxiosError<BodhiErrorResponse> | undefined)?.response?.data;
  return data && typeof data === 'object' ? (data as BodhiErrorResponse).error : undefined;
}

/**
 * Resolve a user-facing message from an API error.
 *
 * When the server returns a structured Bodhi envelope, its `message` is used and
 * the supplied fallback covers a message-less envelope — we do NOT surface the
 * raw axios "Request failed with status code N" string for server-shaped errors.
 * Only for transport/network errors (no envelope at all) do we fall back to the
 * raw Error message before the supplied default.
 */
export function extractErrorMessage(error: MaybeAxios, fallback: string): string {
  const enveloped = asBodhiError(error);
  if (enveloped) return enveloped.message || fallback;
  const raw = (error as { message?: string } | undefined)?.message;
  return raw || fallback;
}

/** Resolve the Bodhi error `code` from an API error, if present. */
export function extractErrorCode(error: unknown): string | undefined {
  return asBodhiError(error)?.code ?? undefined;
}
