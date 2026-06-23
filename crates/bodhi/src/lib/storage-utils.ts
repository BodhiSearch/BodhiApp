// SSR-safe, try/catch-guarded wrappers around Web Storage.
//
// All getters return null on a missing key or when storage is unavailable
// (SSR, disabled cookies, private-mode quota errors). All setters/removers
// swallow errors silently. Callers own serialization (JSON.stringify/parse).

export function canAccessStorage(): boolean {
  return typeof window !== 'undefined';
}

function safeGet(storage: 'localStorage' | 'sessionStorage', key: string): string | null {
  if (typeof window === 'undefined') return null;
  try {
    return window[storage].getItem(key);
  } catch {
    return null;
  }
}

function safeSet(storage: 'localStorage' | 'sessionStorage', key: string, value: string): void {
  if (typeof window === 'undefined') return;
  try {
    window[storage].setItem(key, value);
  } catch {
    // storage unavailable / quota exceeded
  }
}

function safeRemove(storage: 'localStorage' | 'sessionStorage', key: string): void {
  if (typeof window === 'undefined') return;
  try {
    window[storage].removeItem(key);
  } catch {
    // storage unavailable
  }
}

export function safeGetLocalStorage(key: string): string | null {
  return safeGet('localStorage', key);
}

export function safeSetLocalStorage(key: string, value: string): void {
  safeSet('localStorage', key, value);
}

export function safeRemoveLocalStorage(key: string): void {
  safeRemove('localStorage', key);
}

export function safeGetSessionStorage(key: string): string | null {
  return safeGet('sessionStorage', key);
}

export function safeSetSessionStorage(key: string, value: string): void {
  safeSet('sessionStorage', key, value);
}

export function safeRemoveSessionStorage(key: string): void {
  safeRemove('sessionStorage', key);
}
