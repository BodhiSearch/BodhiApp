'use client';

import { useCallback, useEffect, useRef, useState } from 'react';

import { useChatDB } from '@/hooks/use-chat-db';
import { Chat } from '@/types/chat';

const LOCAL_STORAGE_KEY = 'bodhi-last-toolset-selection';

/**
 * Get the last used toolset selection from localStorage.
 */
function getLastToolsetSelection(): string[] {
  if (typeof window === 'undefined') return [];
  try {
    const stored = localStorage.getItem(LOCAL_STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      if (Array.isArray(parsed)) {
        return parsed;
      }
    }
  } catch {
    // Ignore parse errors
  }
  return [];
}

/**
 * Save toolset selection to localStorage for inheritance by new chats.
 */
function saveLastToolsetSelection(toolsets: string[]): void {
  if (typeof window === 'undefined') return;
  try {
    localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(toolsets));
  } catch {
    // Ignore storage errors
  }
}

export interface UseToolsetSelectionReturn {
  /** Currently selected toolset IDs */
  enabledToolsets: string[];
  /** Toggle a toolset on/off */
  toggleToolset: (toolsetId: string) => void;
  /** Check if a specific toolset is enabled */
  isToolsetEnabled: (toolsetId: string) => boolean;
  /** Enable a toolset */
  enableToolset: (toolsetId: string) => void;
  /** Disable a toolset */
  disableToolset: (toolsetId: string) => void;
  /** Set all enabled toolsets at once */
  setEnabledToolsets: (toolsets: string[]) => void;
  /** Whether the selection has changed from the persisted chat state */
  hasChanges: boolean;
}

/**
 * Hook to manage toolset selection for the current chat.
 *
 * - Initializes from currentChat.enabledToolsets if available
 * - Falls back to last used selection for new chats (inheritance)
 * - Maintains temporary state until chat is saved
 * - Saves selection to localStorage for new chat inheritance
 */
export function useToolsetSelection(): UseToolsetSelectionReturn {
  const { currentChat } = useChatDB();

  // Track the initial state for detecting changes
  const initialStateRef = useRef<string[] | null>(null);

  // Initialize state based on current chat or last selection
  const [enabledToolsets, setEnabledToolsetsState] = useState<string[]>(() => {
    if (currentChat?.enabledToolsets) {
      return currentChat.enabledToolsets;
    }
    // For new chats, inherit from last used selection
    return getLastToolsetSelection();
  });

  // Update state when current chat changes
  useEffect(() => {
    if (currentChat?.enabledToolsets) {
      setEnabledToolsetsState(currentChat.enabledToolsets);
      initialStateRef.current = currentChat.enabledToolsets;
    } else if (currentChat === null) {
      // New chat - inherit from last selection
      const lastSelection = getLastToolsetSelection();
      setEnabledToolsetsState(lastSelection);
      initialStateRef.current = null;
    }
  }, [currentChat]);

  // Save selection to localStorage whenever it changes
  useEffect(() => {
    saveLastToolsetSelection(enabledToolsets);
  }, [enabledToolsets]);

  const toggleToolset = useCallback((toolsetId: string) => {
    setEnabledToolsetsState((prev) => {
      if (prev.includes(toolsetId)) {
        return prev.filter((id) => id !== toolsetId);
      }
      return [...prev, toolsetId];
    });
  }, []);

  const isToolsetEnabled = useCallback(
    (toolsetId: string) => {
      return enabledToolsets.includes(toolsetId);
    },
    [enabledToolsets]
  );

  const enableToolset = useCallback((toolsetId: string) => {
    setEnabledToolsetsState((prev) => {
      if (prev.includes(toolsetId)) return prev;
      return [...prev, toolsetId];
    });
  }, []);

  const disableToolset = useCallback((toolsetId: string) => {
    setEnabledToolsetsState((prev) => prev.filter((id) => id !== toolsetId));
  }, []);

  const setEnabledToolsets = useCallback((toolsets: string[]) => {
    setEnabledToolsetsState(toolsets);
  }, []);

  // Check if selection has changed from initial state
  const hasChanges = (() => {
    if (initialStateRef.current === null) {
      // New chat - consider changed if anything is selected
      return enabledToolsets.length > 0;
    }
    // Compare arrays
    if (enabledToolsets.length !== initialStateRef.current.length) return true;
    return enabledToolsets.some((id) => !initialStateRef.current!.includes(id));
  })();

  return {
    enabledToolsets,
    toggleToolset,
    isToolsetEnabled,
    enableToolset,
    disableToolset,
    setEnabledToolsets,
    hasChanges,
  };
}
