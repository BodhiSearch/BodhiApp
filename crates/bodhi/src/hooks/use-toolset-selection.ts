'use client';

import { useCallback, useEffect, useRef, useState } from 'react';

import { useChatDB } from '@/hooks/use-chat-db';

const LOCAL_STORAGE_KEY = 'bodhi-last-toolset-selection';

type EnabledTools = Record<string, string[]>;

/**
 * Get the last used toolset selection from localStorage.
 */
function getLastToolsetSelection(): EnabledTools {
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

/**
 * Save toolset selection to localStorage for inheritance by new chats.
 */
function saveLastToolsetSelection(tools: EnabledTools): void {
  if (typeof window === 'undefined') return;
  try {
    localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(tools));
  } catch {
    // Ignore storage errors
  }
}

export type CheckboxState = 'checked' | 'unchecked' | 'indeterminate';

export interface UseToolsetSelectionReturn {
  /** Currently enabled tools by toolset */
  enabledTools: EnabledTools;
  /** Toggle a specific tool on/off */
  toggleTool: (toolsetId: string, toolName: string) => void;
  /** Toggle entire toolset (enable all or disable all) */
  toggleToolset: (toolsetId: string, allToolNames: string[]) => void;
  /** Check if a specific toolset is enabled */
  isToolsetEnabled: (toolsetId: string) => boolean;
  /** Check if a specific tool is enabled */
  isToolEnabled: (toolsetId: string, toolName: string) => boolean;
  /** Get checkbox state for toolset (checked/unchecked/indeterminate) */
  getToolsetCheckboxState: (toolsetId: string, totalTools: number) => CheckboxState;
  /** Set all enabled tools at once */
  setEnabledTools: (tools: EnabledTools) => void;
  /** Whether the selection has changed from the persisted chat state */
  hasChanges: boolean;
}

/**
 * Hook to manage toolset selection for the current chat.
 *
 * - Initializes from currentChat.enabledTools if available
 * - Falls back to last used selection for new chats (inheritance)
 * - Maintains temporary state until chat is saved
 * - Saves selection to localStorage for new chat inheritance
 */
export function useToolsetSelection(): UseToolsetSelectionReturn {
  const { currentChat } = useChatDB();

  // Track the initial state for detecting changes
  const initialStateRef = useRef<EnabledTools | null>(null);

  // Initialize state based on current chat or last selection
  const [enabledTools, setEnabledToolsState] = useState<EnabledTools>(() => {
    if (currentChat?.enabledTools) {
      return currentChat.enabledTools;
    }
    // For new chats, inherit from last used selection
    return getLastToolsetSelection();
  });

  // Update state when current chat changes
  useEffect(() => {
    if (currentChat?.enabledTools) {
      setEnabledToolsState(currentChat.enabledTools);
      initialStateRef.current = currentChat.enabledTools;
    } else if (currentChat === null) {
      // New chat - inherit from last selection
      const lastSelection = getLastToolsetSelection();
      setEnabledToolsState(lastSelection);
      initialStateRef.current = null;
    }
  }, [currentChat]);

  // Save selection to localStorage whenever it changes
  useEffect(() => {
    saveLastToolsetSelection(enabledTools);
  }, [enabledTools]);

  const toggleTool = useCallback((toolsetId: string, toolName: string) => {
    setEnabledToolsState((prev) => {
      const currentTools = prev[toolsetId] || [];
      if (currentTools.includes(toolName)) {
        // Remove tool
        const newTools = currentTools.filter((name) => name !== toolName);
        if (newTools.length === 0) {
          // Remove toolset entirely if no tools left
          const { [toolsetId]: _, ...rest } = prev;
          return rest;
        }
        return { ...prev, [toolsetId]: newTools };
      } else {
        // Add tool
        return { ...prev, [toolsetId]: [...currentTools, toolName] };
      }
    });
  }, []);

  const toggleToolset = useCallback((toolsetId: string, allToolNames: string[]) => {
    setEnabledToolsState((prev) => {
      const currentTools = prev[toolsetId] || [];
      if (currentTools.length > 0) {
        // Has some tools enabled - disable all
        const { [toolsetId]: _, ...rest } = prev;
        return rest;
      } else {
        // No tools enabled - enable all
        return { ...prev, [toolsetId]: allToolNames };
      }
    });
  }, []);

  const isToolsetEnabled = useCallback(
    (toolsetId: string) => {
      return toolsetId in enabledTools && enabledTools[toolsetId].length > 0;
    },
    [enabledTools]
  );

  const isToolEnabled = useCallback(
    (toolsetId: string, toolName: string) => {
      const tools = enabledTools[toolsetId];
      return tools ? tools.includes(toolName) : false;
    },
    [enabledTools]
  );

  const getToolsetCheckboxState = useCallback(
    (toolsetId: string, totalTools: number): CheckboxState => {
      const enabledCount = enabledTools[toolsetId]?.length || 0;
      if (enabledCount === 0) return 'unchecked';
      if (enabledCount === totalTools) return 'checked';
      return 'indeterminate';
    },
    [enabledTools]
  );

  const setEnabledTools = useCallback((tools: EnabledTools) => {
    setEnabledToolsState(tools);
  }, []);

  // Check if selection has changed from initial state
  const hasChanges = (() => {
    if (initialStateRef.current === null) {
      // New chat - consider changed if anything is selected
      return Object.keys(enabledTools).length > 0;
    }
    // Compare objects
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
