import { type ClassValue, clsx } from 'clsx';
import { customAlphabet } from 'nanoid';
import { twMerge } from 'tailwind-merge';

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
  try {
    const redirectUrl = new URL(location);
    const currentUrl = new URL(window.location.href);

    // Check if scheme and host match (same origin)
    if (redirectUrl.protocol === currentUrl.protocol && redirectUrl.host === currentUrl.host) {
      // Same origin - use Next.js router with pathname + search + hash
      const internalPath = redirectUrl.pathname + redirectUrl.search + redirectUrl.hash;
      router.push(internalPath);
    } else {
      // Different origin - use window.location.href for external redirects (OAuth provider)
      window.location.href = location;
    }
  } catch (error) {
    // If URL parsing fails, treat as external URL
    window.location.href = location;
  }
}
