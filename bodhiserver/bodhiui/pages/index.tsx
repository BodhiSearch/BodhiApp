import { Chat } from "@/components/chat";
import { useLocalStorage } from "@/lib/hooks/use-local-storage";
import { nanoid } from "@/lib/utils";
import { Message } from "ai/react";

export default function Home() {
  const id = nanoid();
  const initialMessages: Message[] = [];
  const _ = useLocalStorage('newChatId', id)
  return (
    <Chat id={id} initialMessages={initialMessages} isLoading={false} />
  );
}
