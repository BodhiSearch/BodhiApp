import { type Message, useChat } from "ai/react";
import { ChatList } from "@/components/chat-list";
import { EmptyScreen } from "@/components/empty-screen";


export interface ChatProps {
  id: string
  initialMessages: Message[]
}

export function Chat({ id, initialMessages }: ChatProps) {
  const { messages } = useChat({ initialMessages, id });
  return (<div className="group w-full overflow-auto pl-0 peer-[[data-state=open]]:lg:pl-[250px] peer-[[data-state=open]]:xl:pl-[300px]">
    {messages.length ? <ChatList messages={messages} /> : <EmptyScreen />}
  </div>)
}

