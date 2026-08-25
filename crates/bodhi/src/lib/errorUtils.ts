import { BodhiErrorResponse } from '@bodhiapp/ts-client';
import { AxiosError } from 'axios';

type MaybeAxios = AxiosError<BodhiErrorResponse> | { message?: string } | unknown;

function asBodhiError(error: unknown): BodhiErrorResponse['error'] | undefined {
  const data = (error as AxiosError<BodhiErrorResponse> | undefined)?.response?.data;
  return data && typeof data === 'object' ? (data as BodhiErrorResponse).error : undefined;
}

// For a structured Bodhi envelope, never surfaces the raw axios "Request failed
// with status code N" message; only transport errors (no envelope) fall back to it.
export function extractErrorMessage(error: MaybeAxios, fallback: string): string {
  const enveloped = asBodhiError(error);
  if (enveloped) return enveloped.message || fallback;
  const raw = (error as { message?: string } | undefined)?.message;
  return raw || fallback;
}

export function extractErrorCode(error: unknown): string | undefined {
  return asBodhiError(error)?.code ?? undefined;
}
