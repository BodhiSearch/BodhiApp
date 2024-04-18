import { type Message } from "ai/react";
import { Separator } from "@/components/ui/separator";
import { ChatMessage } from "@/components/chat-message";


export interface ChatListProps {
  messages: Message[]
}

export function ChatList({ messages }: ChatListProps) {
  if (!messages.length) {
    return null
  }
  return (
    <div className="relative mx-auto max-w-2xl px-4">
      {messages.map((message, index) => (
        <div key={message.id}>
          <ChatMessage message={message} />
          {index < messages.length - 1 && <Separator className="my-4" />}
        </div>
      ))}
    </div>
  )
}