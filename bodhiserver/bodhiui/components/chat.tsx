import { type Message, useChat } from "ai/react";
import { ChatList } from "@/components/chat-list";
import { ChatPanel } from "@/components/chat-panel";
import { cn } from "@/lib/utils";
import { useChatHistory } from "@/lib/hooks/use-chat-history";
import { useChatSettings } from "@/lib/hooks/use-chat-settings";


export interface ChatProps {
  id: string
  isLoading: boolean
  initialMessages: Message[]
}

export function Chat({ id, isLoading: chatLoading, initialMessages }: ChatProps) {
  const { update } = useChatHistory();
  const { model } = useChatSettings();
  const { messages, input, setInput, isLoading, append, reload } = useChat({
    initialMessages,
    id,
    body: { id, model },
    onFinish: async () => {
      await update();
    }
  });
  return <div className="group w-full overflow-auto pl-0 peer-[[data-state=open]]:lg:pl-[250px] peer-[[data-state=open]]:xl:pl-[300px]">
    <div className={cn('pb-[200px] pt-4 md:pt-10')}>
      <ChatList messages={messages} chatLoading={chatLoading} chatStreaming={isLoading} />
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

