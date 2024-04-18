import { Chat } from "@/components/chat";
import { getChat } from "@/lib/backend";
import { Message } from "ai/react";
import { useSearchParams } from "next/navigation";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

export default function ChatPage() {
  const query = useSearchParams();
  const [_, setLoading] = useState(true);
  const [messages, setMessages] = useState([])
  const router = useRouter();
  const id = query?.get('id');
  if (!id) {
    router.push('/').then(() => { }).catch((err) => { console.log(`id missing, error routing to home: ${err}`) });
    return;
  }

  useEffect(() => {
    (async () => {
      let { data, status } = await getChat(id);
      if (status !== 200) {
        setMessages(data)
        setLoading(false)
      }
    })()
  }, []);
  return (
    <Chat id={id} initialMessages={messages} />
  );
}
