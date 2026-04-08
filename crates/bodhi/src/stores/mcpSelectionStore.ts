import { create } from 'zustand';

import { useChatStore } from './chatStore';

const LOCAL_STORAGE_KEY = 'bodhi-last-mcp-selection';

type EnabledMcpTools = Record<string, string[]>;

function getLastMcpSelection(): EnabledMcpTools {
  if (typeof window === 'undefined') return {};
  try {
    const stored = localStorage.getItem(LOCAL_STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      if (typeof parsed === 'object' && !Array.isArray(parsed)) {
        return parsed;
      }
    }
  } catch {
    // Ignore parse errors
  }
  return {};
}

function saveLastMcpSelection(tools: EnabledMcpTools): void {
  if (typeof window === 'undefined') return;
  try {
    localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(tools));
  } catch {
    // Ignore storage errors
  }
}

export type CheckboxState = 'checked' | 'unchecked' | 'indeterminate';

export interface McpSelectionStoreState {
  enabledTools: EnabledMcpTools;
  hasChanges: boolean;

  toggleTool: (mcpId: string, toolName: string) => void;
  toggleMcp: (mcpId: string, allToolNames: string[]) => void;
  isMcpEnabled: (mcpId: string) => boolean;
  isToolEnabled: (mcpId: string, toolName: string) => boolean;
  getMcpCheckboxState: (mcpId: string, totalTools: number) => CheckboxState;
  setEnabledTools: (tools: EnabledMcpTools) => void;
  loadForChat: () => void;
}

let _initialState: EnabledMcpTools | null = null;

function computeHasChanges(enabledTools: EnabledMcpTools): boolean {
  if (_initialState === null) {
    return Object.keys(enabledTools).length > 0;
  }
  const initialKeys = Object.keys(_initialState);
  const currentKeys = Object.keys(enabledTools);
  if (initialKeys.length !== currentKeys.length) return true;
  for (const key of currentKeys) {
    const initialTools = _initialState[key] || [];
    const currentTools = enabledTools[key] || [];
    if (initialTools.length !== currentTools.length) return true;
    if (currentTools.some((tool) => !initialTools.includes(tool))) return true;
  }
  return false;
}

export const useMcpSelectionStore = create<McpSelectionStoreState>((set, get) => ({
  enabledTools: getLastMcpSelection(),
  hasChanges: false,

  toggleTool: (mcpId, toolName) => {
    const prev = get().enabledTools;
    const currentTools = prev[mcpId] || [];
    let next: EnabledMcpTools;
    if (currentTools.includes(toolName)) {
      const newTools = currentTools.filter((name) => name !== toolName);
      if (newTools.length === 0) {
        const { [mcpId]: _, ...rest } = prev;
        next = rest;
      } else {
        next = { ...prev, [mcpId]: newTools };
      }
    } else {
      next = { ...prev, [mcpId]: [...currentTools, toolName] };
    }
    saveLastMcpSelection(next);
    set({ enabledTools: next, hasChanges: computeHasChanges(next) });
  },

  toggleMcp: (mcpId, allToolNames) => {
    const prev = get().enabledTools;
    const currentTools = prev[mcpId] || [];
    let next: EnabledMcpTools;
    if (currentTools.length > 0) {
      const { [mcpId]: _, ...rest } = prev;
      next = rest;
    } else {
      next = { ...prev, [mcpId]: allToolNames };
    }
    saveLastMcpSelection(next);
    set({ enabledTools: next, hasChanges: computeHasChanges(next) });
  },

  isMcpEnabled: (mcpId) => {
    const tools = get().enabledTools;
    return mcpId in tools && tools[mcpId].length > 0;
  },

  isToolEnabled: (mcpId, toolName) => {
    const tools = get().enabledTools[mcpId];
    return tools ? tools.includes(toolName) : false;
  },

  getMcpCheckboxState: (mcpId, totalTools) => {
    const enabledCount = get().enabledTools[mcpId]?.length || 0;
    if (enabledCount === 0) return 'unchecked';
    if (enabledCount === totalTools) return 'checked';
    return 'indeterminate';
  },

  setEnabledTools: (tools) => {
    saveLastMcpSelection(tools);
    set({ enabledTools: tools, hasChanges: computeHasChanges(tools) });
  },

  loadForChat: () => {
    const chatStore = useChatStore.getState();
    const currentChat = chatStore.currentChatId
      ? (chatStore.chats.find((c) => c.id === chatStore.currentChatId) ?? null)
      : null;

    if (currentChat?.enabledMcpTools) {
      _initialState = currentChat.enabledMcpTools;
      set({ enabledTools: currentChat.enabledMcpTools, hasChanges: false });
    } else {
      _initialState = null;
      const lastSelection = getLastMcpSelection();
      set({ enabledTools: lastSelection, hasChanges: Object.keys(lastSelection).length > 0 });
    }
  },
}));

// Cross-store subscription: reload MCP selection when chat changes
let _mcpUnsubscribe: (() => void) | null = null;
export function initMcpSelectionSubscription() {
  _mcpUnsubscribe?.();
  _mcpUnsubscribe = useChatStore.subscribe((state, prevState) => {
    if (state.currentChatId !== prevState.currentChatId) {
      useMcpSelectionStore.getState().loadForChat();
    }
  });
}
