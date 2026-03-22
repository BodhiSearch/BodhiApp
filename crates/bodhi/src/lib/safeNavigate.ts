/**
 * Safely navigate to a URL, blocking dangerous URI schemes like `javascript:` and `data:`.
 * Returns false if navigation was blocked.
 *
 * This function prevents stored XSS attacks where an attacker stores a `javascript:` URI
 * in the database and it gets assigned to `window.location.href` by the frontend.
 *
 * Only `http:` and `https:` protocols are allowed.
 */
export function safeNavigate(url: string): boolean {
  try {
    const trimmed = url.trim();
    if (!trimmed) {
      console.error('Blocked navigation to empty URL');
      return false;
    }
    const parsed = new URL(trimmed, window.location.origin);
    if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
      console.error('Blocked navigation to unsafe URL scheme:', parsed.protocol);
      return false;
    }
    window.location.href = trimmed;
    return true;
  } catch {
    console.error('Blocked navigation to invalid URL:', url);
    return false;
  }
}
