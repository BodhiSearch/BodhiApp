import { useChat, type Message as AIMessage } from "ai/react";
import { type Chat as ChatModel, Message } from "@/lib/types";
import { ChatList } from "@/components/chat-list";
import { ChatPanel } from "@/components/chat-panel";
import { API_BASE_URL, RouteChat, cn } from "@/lib/utils";
import { useChatHistory } from "@/lib/hooks/use-chat-history";
import { useChatSettings } from "@/lib/hooks/use-chat-settings";
import { useRouter } from "next/router";
import { useLocalStorage } from "@/lib/hooks/use-local-storage";
import { updateChat } from "@/lib/backend";

export interface ChatProps {
  id?: string
  isLoading: boolean
  initialMessages: Message[]
}

export function Chat({ id, isLoading: chatLoading, initialMessages: apiMessages }: ChatProps) {
  const { update } = useChatHistory();
  const { model } = useChatSettings();
  const router = useRouter();
  let initialMessages = apiMessages as AIMessage[];
  const { messages, input, setInput, isLoading, append, reload } = useChat({
    api: `${API_BASE_URL}v1/chat/completions`,
    streamMode: 'sse',
    initialMessages,
    id,
    body: { model, stream: true },
    onFinish: async (messages: AIMessage[], message: AIMessage) => {
      if (!!messages) {
        let title = messages[0].content.substring(0, 100);
        const chat: ChatModel = {
          id: id as string,
          title,
          messages: [...messages, message],
          createdAt: new Date().getTime()
        };
        await updateChat(chat);
        await router.push(RouteChat(id as string));
        await update();
      }
    }
  });
  return <div className="chat_tsx group w-full overflow-auto pl-0 peer-[[data-state=open]]:lg:pl-[250px] peer-[[data-state=open]]:xl:pl-[300px]">
    <div className={cn('pb-[200px] pt-4 md:pt-10')}>
      <ChatList messages={messages as Message[]} chatLoading={chatLoading} chatStreaming={isLoading} />
    </div>
    <ChatPanel
      id={id}
      isLoading={isLoading}
      stop={stop}
      append={append}
      reload={reload}
      messages={messages}
      input={input}
      setInput={setInput}
    />
  </div>
}

