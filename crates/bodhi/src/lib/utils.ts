import { AliasResponse, ApiAliasResponse, UserAliasResponse, ModelAliasResponse } from '@bodhiapp/ts-client';
import { type ClassValue, clsx } from 'clsx';
import { customAlphabet } from 'nanoid';
import { twMerge } from 'tailwind-merge';

import { safeNavigate } from '@/lib/safeNavigate';

import { BASE_PATH } from '@/lib/constants';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export const nanoid = customAlphabet('0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz', 7);

/**
 * Smart URL handling utility that determines whether to use TanStack Router navigate or window.location.href
 * based on same-origin vs external URL detection.
 *
 * @param location - The URL to redirect to
 * @param navigate - TanStack Router navigate function from useNavigate()
 */
export function handleSmartRedirect(
  location: string,
  navigate: (opts: { to: string; search?: Record<string, string> }) => void
): void {
  if (location.startsWith('/')) {
    const origin = `${window.location.protocol}//${window.location.host}`;
    const parsed = new URL(location, origin);
    let pathname = parsed.pathname;
    if (pathname.startsWith(BASE_PATH)) {
      pathname = pathname.slice(BASE_PATH.length) || '/';
    }
    if (pathname !== '/' && !pathname.endsWith('/')) {
      pathname += '/';
    }
    const search = Object.fromEntries(parsed.searchParams.entries());
    navigate({ to: pathname, ...(Object.keys(search).length > 0 && { search }) });
    return;
  }

  try {
    const redirectUrl = new URL(location);
    const currentUrl = new URL(window.location.href);
    if (redirectUrl.protocol === currentUrl.protocol && redirectUrl.host === currentUrl.host) {
      let pathname = redirectUrl.pathname;
      if (pathname.startsWith(BASE_PATH)) {
        pathname = pathname.slice(BASE_PATH.length) || '/';
      }
      if (pathname !== '/' && !pathname.endsWith('/')) {
        pathname += '/';
      }
      const search = Object.fromEntries(redirectUrl.searchParams.entries());
      navigate({ to: pathname, ...(Object.keys(search).length > 0 && { search }) });
    } else {
      safeNavigate(location);
    }
  } catch {
    safeNavigate(location);
  }
}

/**
 * Type guard helper functions for AliasResponse discriminated union
 */
export const isApiAlias = (model: AliasResponse): model is ApiAliasResponse => model.source === 'api';

export const isUserAlias = (model: AliasResponse): model is UserAliasResponse => model.source === 'user';

export const isModelAlias = (model: AliasResponse): model is ModelAliasResponse => model.source === 'model';

export const isLocalAlias = (model: AliasResponse): model is UserAliasResponse | ModelAliasResponse =>
  model.source === 'user' || model.source === 'model';

// Helper type for local aliases that have repo, filename, snapshot properties
export type LocalAlias = UserAliasResponse | ModelAliasResponse;

// Type guard that ensures the model has local file properties
export const hasLocalFileProperties = (model: AliasResponse): model is LocalAlias => isLocalAlias(model);

// Type guard for models that can have metadata (local models only)
export const hasModelMetadata = (model: AliasResponse): model is UserAliasResponse | ModelAliasResponse =>
  model.source === 'user' || model.source === 'model';
