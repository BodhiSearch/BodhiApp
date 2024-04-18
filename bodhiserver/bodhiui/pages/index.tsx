import { nanoid } from "nanoid";
import { Chat } from "@/components/chat";
import { Message } from "ai/react";

export default function Home() {
  const id = nanoid();
  const initialMessages: Message[] = [];
  return (
    <Chat id={id} initialMessages={initialMessages} />
  );
}
