import * as oauth from 'oauth4webapi';
import type { OAuthConfig } from '@/context/AuthContext';

export interface PkceParams {
  codeVerifier: string;
  codeChallenge: string;
  state: string;
}

export async function generatePkce(): Promise<PkceParams> {
  const codeVerifier = oauth.generateRandomCodeVerifier();
  const codeChallenge = await oauth.calculatePKCECodeChallenge(codeVerifier);
  const state = oauth.generateRandomState();
  return { codeVerifier, codeChallenge, state };
}

function trimSlash(url: string): string {
  return url.replace(/\/+$/, '');
}

function authorizationEndpoint(config: OAuthConfig): string {
  return `${trimSlash(config.bodhiServerUrl)}/ui/apps/auth/`;
}

// No discovery: authorize on BodhiApp's consent page, tokens straight from Keycloak.
export function authorizationServer(config: OAuthConfig): oauth.AuthorizationServer {
  const issuer = `${trimSlash(config.authServerUrl)}/realms/${config.realm}`;
  return {
    issuer,
    authorization_endpoint: authorizationEndpoint(config),
    token_endpoint: `${issuer}/protocol/openid-connect/token`,
  };
}

export function buildAuthUrl(
  config: OAuthConfig,
  codeChallenge: string,
  state: string,
  sourceAccessRequestId?: string
): string {
  const url = new URL(authorizationEndpoint(config));
  url.searchParams.set('client_id', config.clientId);
  url.searchParams.set('redirect_uri', config.redirectUri);
  url.searchParams.set('response_type', 'code');
  url.searchParams.set('state', state);
  url.searchParams.set('code_challenge', codeChallenge);
  url.searchParams.set('code_challenge_method', 'S256');
  url.searchParams.set('scope', config.scope);
  if (sourceAccessRequestId) {
    url.searchParams.set('source_access_request_id', sourceAccessRequestId);
  }
  return url.toString();
}

export async function exchangeCodeForToken(
  code: string,
  state: string,
  config: OAuthConfig
): Promise<oauth.TokenEndpointResponse> {
  if (!config.codeVerifier || !config.state) {
    throw new Error('Missing PKCE verifier or state in saved config');
  }
  const as = authorizationServer(config);
  const client: oauth.Client = { client_id: config.clientId };
  // PKCE is always on; confidential mode additionally authenticates with client_secret_post.
  const clientAuth =
    config.isConfidential && config.clientSecret
      ? oauth.ClientSecretPost(config.clientSecret)
      : oauth.None();

  const callbackParams = oauth.validateAuthResponse(
    as,
    client,
    new URLSearchParams({ code, state }),
    config.state
  );
  const response = await oauth.authorizationCodeGrantRequest(
    as,
    client,
    clientAuth,
    callbackParams,
    config.redirectUri,
    config.codeVerifier
  );
  return oauth.processAuthorizationCodeResponse(as, client, response);
}

export function decodeJwtPayload(token: string): Record<string, unknown> | null {
  const segment = token.split('.')[1];
  if (!segment) return null;
  try {
    const b64 = segment.replace(/-/g, '+').replace(/_/g, '/');
    const padded = b64.padEnd(b64.length + ((4 - (b64.length % 4)) % 4), '=');
    return JSON.parse(atob(padded)) as Record<string, unknown>;
  } catch {
    return null;
  }
}
