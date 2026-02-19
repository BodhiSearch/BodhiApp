import { create } from 'zustand';
import { McpTool, OAuthConfigResponse } from '@bodhiapp/ts-client';

interface McpFormState {
  oauthConfigs: OAuthConfigResponse[];
  selectedOAuthConfigId: string | null;
  isNewOAuthConfig: boolean;
  oauthTokenId: string | null;
  isConnected: boolean;

  fetchedTools: McpTool[];
  selectedTools: Set<string>;
  toolsFetched: boolean;

  setOAuthConfigs: (configs: OAuthConfigResponse[]) => void;
  selectOAuthConfig: (id: string | null) => void;
  setNewOAuthConfig: (isNew: boolean) => void;
  completeOAuthFlow: (tokenId: string) => void;
  disconnect: () => void;
  setFetchedTools: (tools: McpTool[]) => void;
  setSelectedTools: (tools: Set<string>) => void;
  toggleTool: (toolName: string) => void;
  selectAllTools: () => void;
  deselectAllTools: () => void;
  setToolsFetched: (fetched: boolean) => void;

  saveToSession: (formValues: Record<string, unknown>, serverInfo?: { url: string; name: string }) => void;
  restoreFromSession: () => Record<string, unknown> | null;
  clearSession: () => void;

  reset: () => void;
}

const OAUTH_FORM_STORAGE_KEY = 'mcp_oauth_form_state';

export const useMcpFormStore = create<McpFormState>((set, get) => ({
  oauthConfigs: [],
  selectedOAuthConfigId: null,
  isNewOAuthConfig: false,
  oauthTokenId: null,
  isConnected: false,
  fetchedTools: [],
  selectedTools: new Set<string>(),
  toolsFetched: false,

  setOAuthConfigs: (configs) => set({ oauthConfigs: configs }),
  selectOAuthConfig: (id) => set({ selectedOAuthConfigId: id, isNewOAuthConfig: false }),
  setNewOAuthConfig: (isNew) =>
    set({ isNewOAuthConfig: isNew, selectedOAuthConfigId: isNew ? null : get().selectedOAuthConfigId }),

  completeOAuthFlow: (tokenId) =>
    set({
      oauthTokenId: tokenId,
      isConnected: true,
      isNewOAuthConfig: false,
    }),

  disconnect: () =>
    set({
      oauthTokenId: null,
      isConnected: false,
    }),

  setFetchedTools: (tools) => set({ fetchedTools: tools }),
  setSelectedTools: (tools) => set({ selectedTools: tools }),
  toggleTool: (toolName) =>
    set((state) => {
      const next = new Set(state.selectedTools);
      if (next.has(toolName)) next.delete(toolName);
      else next.add(toolName);
      return { selectedTools: next };
    }),
  selectAllTools: () => set((state) => ({ selectedTools: new Set(state.fetchedTools.map((t) => t.name)) })),
  deselectAllTools: () => set({ selectedTools: new Set() }),
  setToolsFetched: (fetched) => set({ toolsFetched: fetched }),

  saveToSession: (formValues, serverInfo) => {
    const state = get();
    const data = {
      ...formValues,
      oauth_config_id: state.selectedOAuthConfigId,
      oauth_token_id: state.oauthTokenId,
      tools_cache: state.fetchedTools.length > 0 ? state.fetchedTools : undefined,
      tools_filter: Array.from(state.selectedTools),
      server_url: serverInfo?.url,
      server_name: serverInfo?.name,
    };
    sessionStorage.setItem(OAUTH_FORM_STORAGE_KEY, JSON.stringify(data));
  },

  restoreFromSession: () => {
    const saved = sessionStorage.getItem(OAUTH_FORM_STORAGE_KEY);
    if (!saved) return null;
    sessionStorage.removeItem(OAUTH_FORM_STORAGE_KEY);
    return JSON.parse(saved);
  },

  clearSession: () => sessionStorage.removeItem(OAUTH_FORM_STORAGE_KEY),

  reset: () => {
    sessionStorage.removeItem(OAUTH_FORM_STORAGE_KEY);
    set({
      oauthConfigs: [],
      selectedOAuthConfigId: null,
      isNewOAuthConfig: false,
      oauthTokenId: null,
      isConnected: false,
      fetchedTools: [],
      selectedTools: new Set(),
      toolsFetched: false,
    });
  },
}));
