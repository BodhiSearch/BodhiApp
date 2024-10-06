import { type Message } from "@/lib/types";
import { Separator } from "@/components/ui/separator";
import { ChatMessage } from "@/components/chat-message";
import { EmptyScreen } from "./empty-screen";
import { Skeleton } from "@/components/ui/skeleton";
import { ChatScrollAnchor } from "@/components/chat-scroll-anchor";

export interface ChatListProps {
  chatLoading: boolean
  chatStreaming: boolean
  messages: Message[]
}

export function ChatList({ chatLoading, chatStreaming, messages }: ChatListProps) {
  if (!chatLoading && messages.length === 0) {
    return <EmptyScreen />
  }
  if (chatLoading) {
    return <div className="chat_list_tsx relative mx-auto max-w-2xl px-4">
      {Array.from({ length: 2 }).map((_, i) => (
        <div key={i}>
          <div className="relative mb-4 flex items-start md:-ml-12">
            <Skeleton className="flex size-8 items-center justify-center rounded-md"></Skeleton>
            <Skeleton className="flex-1 px-1 ml-4 space-y-2 overflow-hidden">&nbsp;</Skeleton>
          </div>
          <Skeleton data-orientation="horizontal" role="none" className="shrink-0 bg-border h-[1px] w-full my-4"></Skeleton>
        </div>
      ))}
    </div>
  }
  return (
    <>
      <div className="chat_list_tsx relative mx-auto max-w-2xl px-4">
        {messages.map((message, index) => (
          <div key={index}>
            <ChatMessage message={message} />
            {index < messages.length - 1 && <Separator className="my-4" />}
          </div>
        ))}
      </div>
      <ChatScrollAnchor trackVisibility={chatStreaming} />
    </>

  )
}