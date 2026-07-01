const REQUIRED_OAUTH_PARAMS = [
  'response_type',
  'client_id',
  'redirect_uri',
  'scope',
  'code_challenge',
  'code_challenge_method',
  'state',
] as const;

export type AuthUrlValidation = { ok: true } | { ok: false; description: string };

/**
 * The app-supplied `auth_url` must target our own Keycloak authorize endpoint and carry a
 * complete PKCE authorization request — this is the guard that keeps the post-approval
 * redirect pointed only at the configured auth server. No scheme check (http and https
 * are both accepted).
 */
export function validateAuthUrl(authUrl: string, authEndpoint: string, appClientId: string): AuthUrlValidation {
  let url: URL;
  try {
    url = new URL(authUrl);
  } catch {
    return { ok: false, description: 'auth_url is not a valid URL' };
  }
  if (`${url.origin}${url.pathname}` !== authEndpoint) {
    return { ok: false, description: 'auth_url does not target the expected authorization endpoint' };
  }
  const params = url.searchParams;
  for (const key of REQUIRED_OAUTH_PARAMS) {
    if (!params.get(key)) {
      return { ok: false, description: `auth_url is missing required parameter: ${key}` };
    }
  }
  if (params.get('response_type') !== 'code') {
    return { ok: false, description: 'auth_url response_type must be "code"' };
  }
  if (params.get('code_challenge_method') !== 'S256') {
    return { ok: false, description: 'auth_url code_challenge_method must be "S256"' };
  }
  if (params.get('client_id') !== appClientId) {
    return { ok: false, description: 'auth_url client_id does not match the access request' };
  }
  return { ok: true };
}

export function appendScopeToAuthUrl(authUrl: string, scope: string): string {
  const url = new URL(authUrl);
  const scopes = (url.searchParams.get('scope') ?? '').split(/\s+/).filter(Boolean);
  if (!scopes.includes(scope)) {
    scopes.push(scope);
  }
  url.searchParams.set('scope', scopes.join(' '));
  return url.toString();
}

export function readState(authUrl: string): string | null {
  try {
    return new URL(authUrl).searchParams.get('state');
  } catch {
    return null;
  }
}

/** OAuth-style error redirect back to the app, marked `error_source=bodhi` so it can tell Bodhi errors from Keycloak ones. */
export function buildErrorRedirect(
  errorUrl: string,
  params: { error: string; errorDescription: string; state?: string | null }
): string {
  const url = new URL(errorUrl);
  url.searchParams.set('error', params.error);
  url.searchParams.set('error_description', params.errorDescription);
  url.searchParams.set('error_source', 'bodhi');
  if (params.state) {
    url.searchParams.set('state', params.state);
  }
  return url.toString();
}
