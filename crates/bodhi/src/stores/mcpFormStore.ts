import { create } from 'zustand';

import { BASE_PATH } from '@/lib/constants';

interface McpFormState {
  selectedAuthConfigId: string | null;
  selectedAuthConfigType: string | null;
  oauthTokenId: string | null;
  isConnected: boolean;
  credentialValues: Record<string, string>;

  setSelectedAuthConfig: (id: string | null, type: string | null) => void;
  completeOAuthFlow: (tokenId: string) => void;
  disconnect: () => void;
  setCredentialValue: (key: string, value: string) => void;
  clearCredentialValues: () => void;

  saveToSession: (formValues: Record<string, unknown>, serverInfo?: { url: string; name: string }) => void;
  restoreFromSession: () => Record<string, unknown> | null;
  clearSession: () => void;

  reset: () => void;
}

export const OAUTH_FORM_STORAGE_KEY = 'mcp_oauth_form_state';

export const useMcpFormStore = create<McpFormState>((set, get) => ({
  selectedAuthConfigId: null,
  selectedAuthConfigType: null,
  oauthTokenId: null,
  isConnected: false,
  credentialValues: {},

  setSelectedAuthConfig: (id, type) => set({ selectedAuthConfigId: id, selectedAuthConfigType: type }),

  completeOAuthFlow: (tokenId) =>
    set({
      oauthTokenId: tokenId,
      isConnected: true,
    }),

  disconnect: () =>
    set({
      oauthTokenId: null,
      isConnected: false,
      credentialValues: {},
    }),

  setCredentialValue: (key, value) =>
    set((state) => ({
      credentialValues: { ...state.credentialValues, [key]: value },
    })),

  clearCredentialValues: () => set({ credentialValues: {} }),

  saveToSession: (formValues, serverInfo) => {
    const state = get();
    const data = {
      ...formValues,
      selected_auth_config_id: state.selectedAuthConfigId,
      selected_auth_config_type: state.selectedAuthConfigType,
      oauth_token_id: state.oauthTokenId,
      server_url: serverInfo?.url,
      server_name: serverInfo?.name,
      return_url:
        typeof window !== 'undefined'
          ? (() => {
              const pathname = window.location.pathname.startsWith(BASE_PATH)
                ? window.location.pathname.slice(BASE_PATH.length) || '/'
                : window.location.pathname;
              return pathname + window.location.search;
            })()
          : undefined,
    };
    sessionStorage.setItem(OAUTH_FORM_STORAGE_KEY, JSON.stringify(data));
  },

  restoreFromSession: () => {
    const saved = sessionStorage.getItem(OAUTH_FORM_STORAGE_KEY);
    if (!saved) return null;
    try {
      const parsed = JSON.parse(saved);
      sessionStorage.removeItem(OAUTH_FORM_STORAGE_KEY);
      return parsed;
    } catch {
      sessionStorage.removeItem(OAUTH_FORM_STORAGE_KEY);
      return null;
    }
  },

  clearSession: () => sessionStorage.removeItem(OAUTH_FORM_STORAGE_KEY),

  reset: () => {
    sessionStorage.removeItem(OAUTH_FORM_STORAGE_KEY);
    set({
      selectedAuthConfigId: null,
      selectedAuthConfigType: null,
      oauthTokenId: null,
      isConnected: false,
      credentialValues: {},
    });
  },
}));
