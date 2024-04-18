import { Inter } from "next/font/google";
import { nanoid } from "nanoid";
import { Chat } from "@/components/chat";
import { Message } from "ai/react";

const inter = Inter({ subsets: ["latin"] });

export default function Home() {
  const id = nanoid();
  const initialMessages: Message[] = [];
  return (
    <Chat id={id} initialMessages={initialMessages} />
  );
}
