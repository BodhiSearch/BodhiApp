import { useCallback, useEffect, useMemo, useRef } from 'react';

import { createFileRoute, useNavigate, useSearch } from '@tanstack/react-router';
import { PanelLeftClose, PanelLeftOpen, Settings2, X } from 'lucide-react';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { useShellChrome } from '@/components/shell/ShellSlotsContext';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { useChatSettingsStore } from '@/stores/chatSettingsStore';
import { useChatStore } from '@/stores/chatStore';
import { hydrateStoresForCurrentChat, initChatStoreSubscriptions } from '@/stores/initStores';

import { ChatHistory } from './-components/ChatHistory';
import { ChatUI } from './-components/ChatUI';
import { NewChatButton } from './-components/NewChatButton';
import { SettingsSidebar } from './-components/settings/SettingsSidebar';

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

  useEffect(() => {
    if (model) {
      useChatSettingsStore.getState().setModel(model);
    }
  }, [model]);

  const toggleHistory = useCallback(() => setHistoryOpen((o) => !o), [setHistoryOpen]);
  const toggleSettings = useCallback(() => setSettingsOpen((o) => !o), [setSettingsOpen]);

  const sidebar = useMemo(
    () => (
      <div className="flex h-full min-h-0 flex-col">
        <NewChatButton />
        {historyOpen && <ChatHistory />}
      </div>
    ),
    [historyOpen]
  );

  const rail = useMemo(() => (settingsOpen ? <SettingsSidebar /> : null), [settingsOpen]);

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
