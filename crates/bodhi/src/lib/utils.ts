import { AliasResponse, ApiAliasResponse, UserAliasResponse, ModelAliasResponse } from '@bodhiapp/ts-client';
import { type ClassValue, clsx } from 'clsx';
import { customAlphabet } from 'nanoid';
import { twMerge } from 'tailwind-merge';

import { BASE_PATH } from '@/lib/constants';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export const nanoid = customAlphabet('0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz', 7);

/**
 * Smart URL handling utility that determines whether to use Next.js router or window.location.href
 * based on same-origin vs external URL detection.
 *
 * @param location - The URL to redirect to
 * @param router - Next.js router instance from useRouter()
 */
export function handleSmartRedirect(location: string, router: { push: (href: string) => void }): void {
  if (location.startsWith('/')) {
    const path = location.startsWith(BASE_PATH) ? location.slice(BASE_PATH.length) || '/' : location;
    router.push(path);
    return;
  }

  try {
    const redirectUrl = new URL(location);
    const currentUrl = new URL(window.location.href);
    if (redirectUrl.protocol === currentUrl.protocol && redirectUrl.host === currentUrl.host) {
      let internalPath = redirectUrl.pathname + redirectUrl.search + redirectUrl.hash;
      if (internalPath.startsWith(BASE_PATH)) {
        internalPath = internalPath.slice(BASE_PATH.length) || '/';
      }
      router.push(internalPath);
    } else {
      window.location.href = location;
    }
  } catch {
    window.location.href = location;
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
