import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { createFileRoute, useNavigate, useSearch } from '@tanstack/react-router';
import { PanelLeftClose, PanelLeftOpen, Settings2, X } from 'lucide-react';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { useShellChrome } from '@/components/shell/ShellSlotsContext';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { useViewTransition } from '@/hooks/useViewTransition';
import { useChatSettingsStore } from '@/stores/chatSettingsStore';
import { useChatStore } from '@/stores/chatStore';
import { hydrateStoresForCurrentChat, initChatStoreSubscriptions } from '@/stores/initStores';
import { useMcpSelectionStore } from '@/stores/mcpSelectionStore';

import { ChatHistorySidebar } from './-components/ChatHistorySidebar';
import { ChatRailTabs, type ChatRailTab } from './-components/ChatRailTabs';
import { ChatUI } from './-components/ChatUI';
import { McpServersPane } from './-components/settings/McpServersPane';
import { ParametersPane } from './-components/settings/ParametersPane';

export const Route = createFileRoute('/chat/')({
  validateSearch: z.object({
    model: z.string().optional(),
    id: z.string().optional(),
  }),
  component: ChatPage,
});

const CHAT_BREADCRUMB = [{ label: 'Chat' }];

/** Keeps the `?model=&id=` URL in sync with the current chat / model selection. */
function ChatUrlSync({ chatIdFromUrl, model }: { chatIdFromUrl?: string; model?: string }) {
  const currentChatId = useChatStore((s) => s.currentChatId);
  const isLoaded = useChatStore((s) => s.isLoaded);
  const chats = useChatStore((s) => s.chats);
  const setCurrentChatId = useChatStore((s) => s.setCurrentChatId);
  const navigate = useNavigate();
  const isInitialSync = useRef(true);

  useEffect(() => {
    if (!isLoaded) return;
    if (chatIdFromUrl && isInitialSync.current) {
      const chatExists = chats.some((c) => c.id === chatIdFromUrl);
      if (chatExists) {
        setCurrentChatId(chatIdFromUrl);
      }
    }
    isInitialSync.current = false;
  }, [chatIdFromUrl, chats, setCurrentChatId, isLoaded]);

  useEffect(() => {
    if (isInitialSync.current) return;

    const search: Record<string, string> = {};
    if (model) search.model = model;
    if (currentChatId) search.id = currentChatId;

    navigate({ to: '/chat/', search, replace: true });
  }, [currentChatId, model, navigate]);

  return null;
}

function ChatScreen() {
  const search = useSearch({ from: '/chat/' });
  const model = search.model;
  const chatIdFromUrl = search.id;

  // History / settings panels are published into the shell's sidebar + rail slots. We own their
  // open state (persisted) so the legacy header toggles below — which the E2E page objects key on —
  // can hide/show the panel content without touching the shell's own nav-collapse.
  const [historyOpen, setHistoryOpen] = useLocalStorage('sidebar-history-open', true);
  const [settingsOpen, setSettingsOpen] = useLocalStorage('sidebar-settings-open', true);
  const [railTab, setRailTab] = useState<ChatRailTab>('parameters');
  const withViewTransition = useViewTransition();

  // Number of MCP servers with at least one tool enabled in this chat (rail-tab badge).
  const enabledMcpTools = useMcpSelectionStore((s) => s.enabledTools);
  const mcpCount = useMemo(
    () => Object.values(enabledMcpTools).filter((tools) => tools.length > 0).length,
    [enabledMcpTools]
  );

  useEffect(() => {
    if (model) {
      useChatSettingsStore.getState().setModel(model);
    }
  }, [model]);

  const toggleHistory = useCallback(() => setHistoryOpen((o) => !o), [setHistoryOpen]);
  const toggleSettings = useCallback(() => setSettingsOpen((o) => !o), [setSettingsOpen]);

  // Cross-fade only the rail PANE on tab swap (reduced-motion aware); never the grid columns.
  const selectRailTab = useCallback(
    (tab: ChatRailTab) => withViewTransition(() => setRailTab(tab)),
    [withViewTransition]
  );

  const sidebar = useMemo(() => <ChatHistorySidebar listOpen={historyOpen} />, [historyOpen]);

  const railHeader = useMemo(
    () => (settingsOpen ? <ChatRailTabs value={railTab} onChange={selectRailTab} mcpCount={mcpCount} /> : null),
    [settingsOpen, railTab, selectRailTab, mcpCount]
  );

  const rail = useMemo(() => {
    if (!settingsOpen) return null;
    return (
      <div className="chat-rail-vt" style={{ viewTransitionName: 'chat-rail-pane' }}>
        {railTab === 'parameters' ? <ParametersPane /> : <McpServersPane />}
      </div>
    );
  }, [settingsOpen, railTab]);

  const headerActions = useMemo(
    () => (
      <>
        <button
          type="button"
          className="shell-icon-btn"
          aria-label="Toggle history"
          data-testid="chat-history-toggle"
          onClick={toggleHistory}
        >
          {historyOpen ? <PanelLeftClose className="h-4 w-4" /> : <PanelLeftOpen className="h-4 w-4" />}
        </button>
        <button
          type="button"
          className="shell-icon-btn"
          aria-label="Toggle settings"
          data-testid="settings-toggle-button"
          onClick={toggleSettings}
        >
          {settingsOpen ? <X className="h-4 w-4" /> : <Settings2 className="h-4 w-4" />}
        </button>
      </>
    ),
    [historyOpen, settingsOpen, toggleHistory, toggleSettings]
  );

  useShellChrome({
    breadcrumb: CHAT_BREADCRUMB,
    headerActions,
    sidebar,
    rail,
    railHeader,
    railDefaultOpen: true,
    mainScroll: false,
    railScroll: false,
    railWidth: 360,
    sidebarWidth: 260,
    contentClass: 'flush',
    resizeKey: 'chat',
    section: 'chat',
  });

  return (
    <>
      <ChatUrlSync chatIdFromUrl={chatIdFromUrl} model={model} />
      <ChatUI />
    </>
  );
}

function ChatPageContent() {
  const loadChats = useChatStore((s) => s.loadChats);
  const search = useSearch({ from: '/chat/' });
  const urlModel = search.model;
  const urlChatId = search.id;

  useEffect(() => {
    initChatStoreSubscriptions();
    const result = loadChats();
    if (result && typeof result.then === 'function') {
      result.then(() => {
        if (urlModel && !urlChatId) {
          // URL has ?model=X without ?id=Y — start fresh with that model; don't hydrate a previous
          // chat's settings which would overwrite the URL model.
          useChatSettingsStore.getState().setModel(urlModel);
        } else {
          hydrateStoresForCurrentChat();
        }
      });
    }
  }, [loadChats, urlModel, urlChatId]);

  return <ChatScreen />;
}

export default function ChatPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ChatPageContent />
    </AppInitializer>
  );
}
