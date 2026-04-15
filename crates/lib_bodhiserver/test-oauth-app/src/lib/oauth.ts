import type { OAuthConfig } from '@/context/AuthContext';

export function generateRandomString(length: number): string {
  const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += charset.charAt(Math.floor(Math.random() * charset.length));
  }
  return result;
}

export function generateCodeVerifier(): string {
  return generateRandomString(128);
}

export async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const digest = await window.crypto.subtle.digest('SHA-256', data);
  return btoa(String.fromCharCode(...new Uint8Array(digest)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
}

export function generateState(): string {
  return generateRandomString(32);
}

export function buildAuthUrl(config: OAuthConfig, codeChallenge: string, state: string): string {
  const authUrl = new URL(
    `${config.authServerUrl}/realms/${config.realm}/protocol/openid-connect/auth`
  );
  authUrl.searchParams.append('client_id', config.clientId);
  authUrl.searchParams.append('redirect_uri', config.redirectUri);
  authUrl.searchParams.append('response_type', 'code');
  authUrl.searchParams.append('scope', config.scope);
  authUrl.searchParams.append('state', state);
  if (!config.isConfidential) {
    authUrl.searchParams.append('code_challenge', codeChallenge);
    authUrl.searchParams.append('code_challenge_method', 'S256');
  }
  return authUrl.toString();
}

export async function exchangeCodeForToken(
  code: string,
  config: OAuthConfig
): Promise<{ access_token: string; [key: string]: unknown }> {
  const tokenUrl = `${config.authServerUrl}/realms/${config.realm}/protocol/openid-connect/token`;
  const params = new URLSearchParams();
  params.append('grant_type', 'authorization_code');
  params.append('client_id', config.clientId);
  params.append('code', code);
  params.append('redirect_uri', config.redirectUri);
  if (config.isConfidential && config.clientSecret) {
    params.append('client_secret', config.clientSecret);
  }
  if (!config.isConfidential && config.codeVerifier) {
    params.append('code_verifier', config.codeVerifier);
  }
  const response = await fetch(tokenUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: params,
  });
  const data = await response.json();
  if (!response.ok) {
    throw new Error(data.error_description || data.error || 'Token exchange failed');
  }
  return data;
}
