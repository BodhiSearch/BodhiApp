import { useCallback, useEffect, useRef, useState } from 'react';

import { useChatDB } from '@/hooks/chat';

const LOCAL_STORAGE_KEY = 'bodhi-last-mcp-selection';

/**
 * EnabledMcpTools maps MCP instance ID (UUID) to enabled tool names
 * Example: { "uuid-abc-123": ["search", "fetch"] }
 */
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

export interface UseMcpSelectionReturn {
  enabledTools: EnabledMcpTools;
  toggleTool: (mcpId: string, toolName: string) => void;
  toggleToolset: (mcpId: string, allToolNames: string[]) => void;
  isToolsetEnabled: (mcpId: string) => boolean;
  isToolEnabled: (mcpId: string, toolName: string) => boolean;
  getToolsetCheckboxState: (mcpId: string, totalTools: number) => CheckboxState;
  setEnabledTools: (tools: EnabledMcpTools) => void;
  hasChanges: boolean;
}

export function useMcpSelection(): UseMcpSelectionReturn {
  const { currentChat } = useChatDB();

  const initialStateRef = useRef<EnabledMcpTools | null>(null);

  const [enabledTools, setEnabledToolsState] = useState<EnabledMcpTools>(() => {
    if (currentChat?.enabledMcpTools) {
      return currentChat.enabledMcpTools;
    }
    return getLastMcpSelection();
  });

  useEffect(() => {
    if (currentChat?.enabledMcpTools) {
      setEnabledToolsState(currentChat.enabledMcpTools);
      initialStateRef.current = currentChat.enabledMcpTools;
    } else if (currentChat === null) {
      const lastSelection = getLastMcpSelection();
      setEnabledToolsState(lastSelection);
      initialStateRef.current = null;
    }
  }, [currentChat]);

  useEffect(() => {
    saveLastMcpSelection(enabledTools);
  }, [enabledTools]);

  const toggleTool = useCallback((mcpId: string, toolName: string) => {
    setEnabledToolsState((prev) => {
      const currentTools = prev[mcpId] || [];
      if (currentTools.includes(toolName)) {
        const newTools = currentTools.filter((name) => name !== toolName);
        if (newTools.length === 0) {
          const { [mcpId]: _, ...rest } = prev;
          return rest;
        }
        return { ...prev, [mcpId]: newTools };
      } else {
        return { ...prev, [mcpId]: [...currentTools, toolName] };
      }
    });
  }, []);

  const toggleToolset = useCallback((mcpId: string, allToolNames: string[]) => {
    setEnabledToolsState((prev) => {
      const currentTools = prev[mcpId] || [];
      if (currentTools.length > 0) {
        const { [mcpId]: _, ...rest } = prev;
        return rest;
      } else {
        return { ...prev, [mcpId]: allToolNames };
      }
    });
  }, []);

  const isToolsetEnabled = useCallback(
    (mcpId: string) => {
      return mcpId in enabledTools && enabledTools[mcpId].length > 0;
    },
    [enabledTools]
  );

  const isToolEnabled = useCallback(
    (mcpId: string, toolName: string) => {
      const tools = enabledTools[mcpId];
      return tools ? tools.includes(toolName) : false;
    },
    [enabledTools]
  );

  const getToolsetCheckboxState = useCallback(
    (mcpId: string, totalTools: number): CheckboxState => {
      const enabledCount = enabledTools[mcpId]?.length || 0;
      if (enabledCount === 0) return 'unchecked';
      if (enabledCount === totalTools) return 'checked';
      return 'indeterminate';
    },
    [enabledTools]
  );

  const setEnabledTools = useCallback((tools: EnabledMcpTools) => {
    setEnabledToolsState(tools);
  }, []);

  const hasChanges = (() => {
    if (initialStateRef.current === null) {
      return Object.keys(enabledTools).length > 0;
    }
    const initialKeys = Object.keys(initialStateRef.current);
    const currentKeys = Object.keys(enabledTools);
    if (initialKeys.length !== currentKeys.length) return true;
    for (const key of currentKeys) {
      const initialTools = initialStateRef.current[key] || [];
      const currentTools = enabledTools[key] || [];
      if (initialTools.length !== currentTools.length) return true;
      if (currentTools.some((tool) => !initialTools.includes(tool))) return true;
    }
    return false;
  })();

  return {
    enabledTools,
    toggleTool,
    toggleToolset,
    isToolsetEnabled,
    isToolEnabled,
    getToolsetCheckboxState,
    setEnabledTools,
    hasChanges,
  };
}
