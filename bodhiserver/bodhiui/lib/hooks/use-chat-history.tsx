import { Dispatch, SetStateAction, createContext, useContext, useEffect, useState } from "react";
import { getChats, clearChats as clearChatApi, removeChat as removeChatApi } from "@/lib/backend";
import { Chat } from "@/lib/types";

export type Status = "loading" | "success" | "error";

interface ChatHistoryContext {
  status: Status
  chats: Chat[]
  refresh: () => Promise<void>
  update: () => Promise<void>
  clearChats: () => Promise<void | { error: string }>
  removeChat: (id: string) => Promise<void | { error: string }>
}

const ChatHistoryContext = createContext<ChatHistoryContext | undefined>(undefined);

export function useChatHistory() {
  const context = useContext(ChatHistoryContext);
  if (!context) {
    throw new Error('useChatHistory must be used within a ChatHistoryProvider');
  }
  return context;
}

interface ChatHistoryProviderProps {
  children: React.ReactNode
}


export function ChatHistoryProvider({ children }: ChatHistoryProviderProps) {
  const [chats, setChats] = useState([]);
  const [status, setStatus] = useState<Status>("loading");

  const refresh = async () => {
    setStatus("loading")
    let { data, status } = await getChats();
    if (status === 200) {
      setChats(data);
      setStatus("success");
    } else {
      console.log(`error loading chats: ${data}`);
      setStatus("error");
    }
  }

  const update = async () => {
    let { data, status } = await getChats();
    if (status === 200) {
      setChats(data);
    } else {
      console.log(`error loading chats: ${data}`);
    }
  }

  const clearChats = async () => {
    let { data, status } = await clearChatApi();
    if (status === 200) {
      await refresh();
    } else {
      console.log(`error calling clearChats: ${data}`)
    }
  }

  const removeChat = async (chatId: string) => {
    let { data, status } = await removeChatApi(chatId);
    if (status === 200) {
      await refresh();
    } else {
      console.log(`error deleting chat: ${chatId}, ${data}`);
    }
  }

  useEffect(() => {
    refresh()
  }, []);

  return <ChatHistoryContext.Provider value={{ status, chats, refresh, update, clearChats, removeChat }}>
    {children}
  </ChatHistoryContext.Provider>
}
