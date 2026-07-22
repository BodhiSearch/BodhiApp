import { beforeEach, describe, expect, it } from 'vitest';

import { useMcpFormStore } from '@/stores/mcpFormStore';

describe('mcpFormStore', () => {
  beforeEach(() => {
    useMcpFormStore.getState().reset();
  });

  it('setConnected toggles isConnected without touching oauthTokenId', () => {
    // Edit-load marks an existing OAuth MCP connected without a known token id (Defect C).
    useMcpFormStore.getState().setConnected(true);
    expect(useMcpFormStore.getState().isConnected).toBe(true);
    expect(useMcpFormStore.getState().oauthTokenId).toBeNull();

    useMcpFormStore.getState().setConnected(false);
    expect(useMcpFormStore.getState().isConnected).toBe(false);
    expect(useMcpFormStore.getState().oauthTokenId).toBeNull();
  });

  it('completeOAuthFlow sets both the real token id and connected state', () => {
    useMcpFormStore.getState().completeOAuthFlow('tok-real');
    expect(useMcpFormStore.getState().oauthTokenId).toBe('tok-real');
    expect(useMcpFormStore.getState().isConnected).toBe(true);
  });
});
