import { randomBytes, createHash } from 'node:crypto';
import { Router, type Request, type Response } from 'express';

interface AuthorizationCode {
  code: string;
  clientId: string;
  redirectUri: string;
  codeChallenge: string;
  codeChallengeMethod: string;
  scope: string;
  state: string;
  createdAt: number;
}

interface AccessToken {
  token: string;
  clientId: string;
  scope: string;
  createdAt: number;
  expiresIn: number;
}

interface RefreshToken {
  token: string;
  clientId: string;
  scope: string;
}

const authorizationCodes = new Map<string, AuthorizationCode>();
const accessTokens = new Map<string, AccessToken>();
const refreshTokens = new Map<string, RefreshToken>();

const CLIENT_ID = process.env.TEST_MCP_OAUTH_CLIENT_ID ?? 'test-mcp-client-id';
const CLIENT_SECRET = process.env.TEST_MCP_OAUTH_CLIENT_SECRET ?? 'test-mcp-client-secret';
const PORT = process.env.TEST_MCP_OAUTH_PORT ?? '55174';

function generateToken(): string {
  return randomBytes(32).toString('hex');
}

function base64urlEncode(buffer: Buffer): string {
  return buffer.toString('base64').replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}

function verifyPkce(codeVerifier: string, codeChallenge: string): boolean {
  const hash = createHash('sha256').update(codeVerifier).digest();
  return base64urlEncode(hash) === codeChallenge;
}

export function isValidAccessToken(token: string): AccessToken | null {
  const record = accessTokens.get(token);
  if (!record) return null;
  const elapsed = (Date.now() - record.createdAt) / 1000;
  if (elapsed > record.expiresIn) {
    accessTokens.delete(token);
    return null;
  }
  return record;
}

export function createOAuthRouter(): Router {
  const router = Router();

  router.get('/.well-known/oauth-authorization-server', (_req: Request, res: Response) => {
    const issuer = `http://localhost:${PORT}`;
    res.json({
      issuer,
      authorization_endpoint: `${issuer}/authorize`,
      token_endpoint: `${issuer}/token`,
      response_types_supported: ['code'],
      grant_types_supported: ['authorization_code', 'refresh_token'],
      code_challenge_methods_supported: ['S256'],
      scopes_supported: ['mcp:tools', 'mcp:read'],
    });
  });

  router.get('/authorize', (req: Request, res: Response) => {
    const {
      response_type,
      client_id,
      redirect_uri,
      state,
      scope,
      code_challenge,
      code_challenge_method,
    } = req.query as Record<string, string>;

    if (response_type !== 'code') {
      res.status(400).send('Unsupported response_type');
      return;
    }
    if (client_id !== CLIENT_ID) {
      res.status(400).send('Unknown client_id');
      return;
    }
    if (!code_challenge || code_challenge_method !== 'S256') {
      res.status(400).send('PKCE S256 required');
      return;
    }

    const html = `<!DOCTYPE html>
<html>
<head><title>Authorize</title></head>
<body>
  <h1>Authorize Application</h1>
  <p>Client <strong>${client_id}</strong> is requesting access to: <strong>${scope ?? 'mcp:tools'}</strong></p>
  <form method="POST" action="/authorize">
    <input type="hidden" name="client_id" value="${client_id}" />
    <input type="hidden" name="redirect_uri" value="${redirect_uri}" />
    <input type="hidden" name="state" value="${state ?? ''}" />
    <input type="hidden" name="scope" value="${scope ?? 'mcp:tools'}" />
    <input type="hidden" name="code_challenge" value="${code_challenge}" />
    <input type="hidden" name="code_challenge_method" value="${code_challenge_method}" />
    <input type="hidden" name="response_type" value="code" />
    <button type="submit" data-testid="approve-btn">Approve</button>
  </form>
</body>
</html>`;
    res.type('html').send(html);
  });

  router.post('/authorize', (req: Request, res: Response) => {
    const { client_id, redirect_uri, state, scope, code_challenge, code_challenge_method } =
      req.body as Record<string, string>;

    if (client_id !== CLIENT_ID) {
      res.status(400).send('Unknown client_id');
      return;
    }

    const code = generateToken();
    authorizationCodes.set(code, {
      code,
      clientId: client_id,
      redirectUri: redirect_uri,
      codeChallenge: code_challenge,
      codeChallengeMethod: code_challenge_method,
      scope: scope ?? 'mcp:tools',
      state: state ?? '',
      createdAt: Date.now(),
    });

    const redirectUrl = new URL(redirect_uri);
    redirectUrl.searchParams.set('code', code);
    if (state) {
      redirectUrl.searchParams.set('state', state);
    }
    res.redirect(redirectUrl.toString());
  });

  router.post('/token', (req: Request, res: Response) => {
    const body = req.body as Record<string, string>;
    const { grant_type, client_id, client_secret } = body;

    if (client_id !== CLIENT_ID || client_secret !== CLIENT_SECRET) {
      res.status(401).json({ error: 'invalid_client' });
      return;
    }

    if (grant_type === 'authorization_code') {
      return handleAuthorizationCodeGrant(body, res);
    }
    if (grant_type === 'refresh_token') {
      return handleRefreshTokenGrant(body, res);
    }

    res.status(400).json({ error: 'unsupported_grant_type' });
  });

  router.all('/register', (_req: Request, res: Response) => {
    res.status(404).json({ error: 'registration_not_supported' });
  });

  return router;
}

function handleAuthorizationCodeGrant(body: Record<string, string>, res: Response): void {
  const { code, code_verifier, redirect_uri } = body;
  const authCode = authorizationCodes.get(code);

  if (!authCode) {
    res
      .status(400)
      .json({ error: 'invalid_grant', error_description: 'Invalid or expired authorization code' });
    return;
  }

  authorizationCodes.delete(code);

  const elapsed = (Date.now() - authCode.createdAt) / 1000;
  if (elapsed > 600) {
    res
      .status(400)
      .json({ error: 'invalid_grant', error_description: 'Authorization code expired' });
    return;
  }

  if (authCode.redirectUri !== redirect_uri) {
    res.status(400).json({ error: 'invalid_grant', error_description: 'redirect_uri mismatch' });
    return;
  }

  if (!code_verifier || !verifyPkce(code_verifier, authCode.codeChallenge)) {
    res.status(400).json({ error: 'invalid_grant', error_description: 'PKCE verification failed' });
    return;
  }

  const accessToken = generateToken();
  const refreshToken = generateToken();

  accessTokens.set(accessToken, {
    token: accessToken,
    clientId: authCode.clientId,
    scope: authCode.scope,
    createdAt: Date.now(),
    expiresIn: 3600,
  });

  refreshTokens.set(refreshToken, {
    token: refreshToken,
    clientId: authCode.clientId,
    scope: authCode.scope,
  });

  res.json({
    access_token: accessToken,
    token_type: 'Bearer',
    expires_in: 3600,
    refresh_token: refreshToken,
    scope: authCode.scope,
  });
}

function handleRefreshTokenGrant(body: Record<string, string>, res: Response): void {
  const { refresh_token } = body;
  const stored = refreshTokens.get(refresh_token);

  if (!stored) {
    res.status(400).json({ error: 'invalid_grant', error_description: 'Invalid refresh token' });
    return;
  }

  refreshTokens.delete(refresh_token);

  const newAccessToken = generateToken();
  const newRefreshToken = generateToken();

  accessTokens.set(newAccessToken, {
    token: newAccessToken,
    clientId: stored.clientId,
    scope: stored.scope,
    createdAt: Date.now(),
    expiresIn: 3600,
  });

  refreshTokens.set(newRefreshToken, {
    token: newRefreshToken,
    clientId: stored.clientId,
    scope: stored.scope,
  });

  res.json({
    access_token: newAccessToken,
    token_type: 'Bearer',
    expires_in: 3600,
    refresh_token: newRefreshToken,
    scope: stored.scope,
  });
}
