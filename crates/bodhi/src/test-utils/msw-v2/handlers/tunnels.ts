import { ENDPOINT_TUNNEL, ENDPOINT_TUNNEL_SETUP, ENDPOINT_TUNNEL_SYNC } from '@/hooks/tunnels';
import { INTERNAL_SERVER_ERROR, typedHttp, type components } from '@/test-utils/msw-v2/setup';

type TunnelStatus = components['schemas']['TunnelStatus'];

/** A fully set-up, switched-off instance; every mock starts from this. */
export function tunnelStatus(overrides: Partial<TunnelStatus> = {}): TunnelStatus {
  return {
    available: true,
    unavailable_reason: null,
    enabled: false,
    state: 'disabled',
    binary: {
      state: 'ready',
      path: '/opt/homebrew/bin/cloudflared',
      source: 'path',
      version: '2026.9.1',
      minimum_version: '2025.2.0',
      error: null,
    },
    login: {
      state: 'ready',
      cert_path: '~/.cloudflared/cert.pem',
      source: 'standard_location',
      zone: 'example.com',
      error: null,
    },
    hostname: null,
    subdomain: null,
    auto_reconnect: true,
    public_url: null,
    oauth_redirect_uri: null,
    auth_sync: { state: 'not_attempted', error: null },
    error_code: null,
    error_message: null,
    ...overrides,
  } as TunnelStatus;
}

/** The live tunnel, with its address and synced redirect. */
export function liveTunnelStatus(overrides: Partial<TunnelStatus> = {}): TunnelStatus {
  return tunnelStatus({
    enabled: true,
    state: 'connected',
    hostname: 'bodhi.example.com',
    subdomain: 'bodhi',
    public_url: 'https://bodhi.example.com',
    oauth_redirect_uri: 'https://bodhi.example.com/ui/auth/callback',
    auth_sync: { state: 'synced', error: null },
    ...overrides,
  });
}

export function mockTunnelStatus(overrides: Partial<TunnelStatus> = {}) {
  return [typedHttp.get(ENDPOINT_TUNNEL, async ({ response }) => response(200 as const).json(tunnelStatus(overrides)))];
}

export function mockTunnelStatusExact(status: TunnelStatus) {
  return [typedHttp.get(ENDPOINT_TUNNEL, async ({ response }) => response(200 as const).json(status))];
}

export function mockEnableTunnel(
  result: Partial<TunnelStatus> = { state: 'connecting', enabled: true },
  { onBody }: { onBody?: (body: unknown) => void } = {}
) {
  return [
    typedHttp.put(ENDPOINT_TUNNEL, async ({ request, response }) => {
      onBody?.(await request.json());
      return response(200 as const).json(tunnelStatus(result));
    }),
  ];
}

export function mockEnableTunnelError({
  code = INTERNAL_SERVER_ERROR.code,
  message = INTERNAL_SERVER_ERROR.message,
  type = INTERNAL_SERVER_ERROR.type,
  status = INTERNAL_SERVER_ERROR.status,
}: Partial<components['schemas']['BodhiError']> & { status?: 400 | 401 | 403 | 500 } = {}) {
  return [
    typedHttp.put(ENDPOINT_TUNNEL, async ({ response }) => response(status).json({ error: { code, message, type } })),
  ];
}

export function mockDisableTunnel(result: Partial<TunnelStatus> = {}) {
  return [
    typedHttp.delete(ENDPOINT_TUNNEL, async ({ response }) =>
      response(200 as const).json(tunnelStatus({ hostname: 'bodhi.example.com', subdomain: 'bodhi', ...result }))
    ),
  ];
}

export function mockTunnelPreferences(result: Partial<TunnelStatus> = {}) {
  return [typedHttp.patch(ENDPOINT_TUNNEL, async ({ response }) => response(200 as const).json(tunnelStatus(result)))];
}

export function mockTunnelSetup(
  result: Partial<TunnelStatus> = {},
  { onBody }: { onBody?: (body: unknown) => void } = {}
) {
  return [
    typedHttp.put(ENDPOINT_TUNNEL_SETUP, async ({ request, response }) => {
      onBody?.(await request.json());
      return response(200 as const).json(tunnelStatus(result));
    }),
  ];
}

export function mockTunnelSync(result: Partial<TunnelStatus> = {}) {
  return [
    typedHttp.post(ENDPOINT_TUNNEL_SYNC, async ({ response }) => response(200 as const).json(liveTunnelStatus(result))),
  ];
}
