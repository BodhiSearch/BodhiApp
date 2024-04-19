import { Chat } from "@/components/chat";
import { getChat } from "@/lib/backend";
import { Root } from "@/lib/utils";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

export default function ChatPage() {
  const router = useRouter()
  const [messages, setMessages] = useState([])
  const [isLoading, setLoading] = useState(true);
  const { id } = router.query;

  useEffect(() => {
    (async () => {
      if (!id) {
        return;
      }
      setLoading(true);
      let { data, status } = await getChat(id as string);
      if (status === 200) {
        setMessages(data.messages)
        setLoading(false);
      }
    })()
  }, [id, setMessages]);
  if (!id) {
    router.push(Root).then(() => { }).catch((err) => { console.log(`id missing, error routing to home: ${err}`) });
    return;
  }
  return (
    <Chat id={id as string} initialMessages={messages} isLoading={isLoading} />
  );
}
