import { create } from 'zustand';
import { McpTool } from '@bodhiapp/ts-client';

interface McpFormState {
  selectedAuthConfigId: string | null;
  selectedAuthConfigType: string | null;
  oauthTokenId: string | null;
  isConnected: boolean;

  fetchedTools: McpTool[];
  selectedTools: Set<string>;
  toolsFetched: boolean;

  setSelectedAuthConfig: (id: string | null, type: string | null) => void;
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

export const OAUTH_FORM_STORAGE_KEY = 'mcp_oauth_form_state';

export const useMcpFormStore = create<McpFormState>((set, get) => ({
  selectedAuthConfigId: null,
  selectedAuthConfigType: null,
  oauthTokenId: null,
  isConnected: false,
  fetchedTools: [],
  selectedTools: new Set<string>(),
  toolsFetched: false,

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
      selected_auth_config_id: state.selectedAuthConfigId,
      selected_auth_config_type: state.selectedAuthConfigType,
      oauth_token_id: state.oauthTokenId,
      tools_cache: state.fetchedTools.length > 0 ? state.fetchedTools : undefined,
      tools_filter: Array.from(state.selectedTools),
      server_url: serverInfo?.url,
      server_name: serverInfo?.name,
      return_url: typeof window !== 'undefined' ? window.location.pathname + window.location.search : undefined,
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
      fetchedTools: [],
      selectedTools: new Set(),
      toolsFetched: false,
    });
  },
}));
