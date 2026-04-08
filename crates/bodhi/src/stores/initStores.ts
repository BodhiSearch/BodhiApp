import { useChatSettingsStore, initChatSettingsSubscription } from './chatSettingsStore';
import { initAgentSubscription } from './agentStore';
import { useMcpSelectionStore, initMcpSelectionSubscription } from './mcpSelectionStore';
import { useChatStore } from './chatStore';

let _initialized = false;

export function initChatStoreSubscriptions() {
  if (_initialized) return;
  _initialized = true;
  initChatSettingsSubscription();
  initAgentSubscription();
  initMcpSelectionSubscription();
}

export function hydrateStoresForCurrentChat() {
  const { currentChatId } = useChatStore.getState();
  if (currentChatId) {
    useChatSettingsStore.getState().loadForChat(currentChatId);
    useMcpSelectionStore.getState().loadForChat();
  }
}
