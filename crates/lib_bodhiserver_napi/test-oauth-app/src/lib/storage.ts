import type { OAuthConfig } from '@/context/AuthContext';

export function saveConfig(config: OAuthConfig): void {
  sessionStorage.setItem('oauthConfig', JSON.stringify(config));
}

export function loadConfig(): OAuthConfig | null {
  const raw = sessionStorage.getItem('oauthConfig');
  if (!raw) return null;
  try {
    return JSON.parse(raw) as OAuthConfig;
  } catch {
    return null;
  }
}

export function saveToken(token: string): void {
  sessionStorage.setItem('accessToken', token);
}

export function loadToken(): string | null {
  return sessionStorage.getItem('accessToken');
}

export function clearAll(): void {
  sessionStorage.clear();
}
