import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { createFileRoute, useNavigate, useSearch } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { useShellChrome } from '@/components/shell/ShellChromeContext';
import { useChatMcp } from '@/hooks/chat/useChatMcp';
import { useViewTransition } from '@/hooks/useViewTransition';
import { useChatSettingsStore } from '@/stores/chatSettingsStore';
import { useChatStore } from '@/stores/chatStore';
import { hydrateStoresForCurrentChat, initChatStoreSubscriptions } from '@/stores/initStores';

import { ChatHistorySidebar } from './-components/ChatHistorySidebar';
import { ChatRailTabs, type ChatRailTab } from './-components/ChatRailTabs';
import { ChatTitle } from './-components/ChatTitle';
import { ChatUI } from './-components/ChatUI';
import { McpServersPane } from './-components/settings/McpServersPane';
import { ParametersPane } from './-components/settings/ParametersPane';

export const Route = createFileRoute('/chat/')({
  staticData: { section: 'chat' },
  validateSearch: z.object({
    model: z.string().optional(),
    id: z.string().optional(),
  }),
  component: ChatPage,
});

// The breadcrumb slot renders a custom node as-is; ChatTitle reads the current chat from the store.
const CHAT_BREADCRUMB = <ChatTitle />;

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

  // History / settings panels are published into the shell's sidebar + rail slots; the shell's own
  // sidepanel toggles handle showing/hiding them, so the panel content is always provided here.
  const [railTab, setRailTab] = useState<ChatRailTab>('parameters');
  const withViewTransition = useViewTransition();

  // Single MCP connection manager + selection, shared by the composer (agent tool execution) and
  // the rail's MCP-servers picker.
  const mcp = useChatMcp();

  useEffect(() => {
    if (model) {
      useChatSettingsStore.getState().setModel(model);
    }
  }, [model]);

  // Cross-fade only the rail PANE on tab swap (reduced-motion aware); never the grid columns.
  const selectRailTab = useCallback(
    (tab: ChatRailTab) => withViewTransition(() => setRailTab(tab)),
    [withViewTransition]
  );

  const sidebar = useMemo(() => <ChatHistorySidebar />, []);

  const railHeader = useMemo(
    () => <ChatRailTabs value={railTab} onChange={selectRailTab} mcpCount={mcp.mcpCount} />,
    [railTab, selectRailTab, mcp.mcpCount]
  );

  const rail = useMemo(
    () => (
      <div className="chat-rail-vt" style={{ viewTransitionName: 'chat-rail-pane' }}>
        {railTab === 'parameters' ? (
          <ParametersPane />
        ) : (
          <McpServersPane
            mcps={mcp.mcps}
            enabledMcpTools={mcp.enabledMcpTools}
            onToggleTool={mcp.toggleTool}
            onAdd={mcp.addMcp}
            onRemove={mcp.removeMcp}
            mcpTools={mcp.mcpTools}
            mcpConnectionStatus={mcp.mcpConnectionStatus}
          />
        )}
      </div>
    ),
    [railTab, mcp]
  );

  useShellChrome({
    breadcrumb: CHAT_BREADCRUMB,
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
  });

  return (
    <>
      <ChatUrlSync chatIdFromUrl={chatIdFromUrl} model={model} />
      <ChatUI agentTools={mcp.agentTools} enabledMcpTools={mcp.enabledMcpTools} />
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
