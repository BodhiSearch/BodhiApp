import { expect, test } from '@/fixtures.mjs';

test.describe('Access Request Version Validation', { tag: ['@oauth'] }, () => {
  test('rejects unknown version in requested resources', async ({ sharedServerUrl, request }) => {
    const response = await request.post(`${sharedServerUrl}/bodhi/v1/apps/request-access`, {
      data: {
        app_client_id: 'test-app-client',
        flow_type: 'popup',
        requested_role: 'scope_user_user',
        requested: {
          version: '99',
          mcp_servers: [{ url: 'https://example.com/mcp' }],
        },
      },
    });

    expect(response.status()).toBe(400);
    const body = await response.json();
    expect(body.error.code).toBe('json_rejection_error');
    expect(body.error.message).toContain('Unsupported resources version');
  });

  test('rejects missing version in requested resources', async ({ sharedServerUrl, request }) => {
    const response = await request.post(`${sharedServerUrl}/bodhi/v1/apps/request-access`, {
      data: {
        app_client_id: 'test-app-client',
        flow_type: 'popup',
        requested_role: 'scope_user_user',
        requested: {
          mcp_servers: [{ url: 'https://example.com/mcp' }],
        },
      },
    });

    expect(response.status()).toBe(400);
    const body = await response.json();
    expect(body.error.code).toBe('json_rejection_error');
    expect(body.error.message).toContain('version');
  });
});
