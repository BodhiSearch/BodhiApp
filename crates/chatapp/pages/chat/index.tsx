import { Chat } from "@/components/chat";
import { getChat } from "@/lib/backend";
import { PageRoot } from "@/lib/utils";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

export default function ChatPage() {
  const router = useRouter();
  const [messages, setMessages] = useState([]);
  const [isLoading, setLoading] = useState(true);
  const [id, setId] = useState<string | undefined>(undefined);

  useEffect(() => {
    (async () => {
      if (!router.isReady) return;
      const { id } = router.query;
      if (!id) {
        return await router.push(PageRoot);
      }
      setLoading(true);
      let { data, status } = await getChat(id as string);
      if (status === 200) {
        setId(id as string);
        setMessages(data.messages);
      }
      setLoading(false);
    })().then(() => { }).catch((err) => { console.log(`${JSON.stringify(err)}`) });
  }, [router, setMessages, setId]);
  return (
    <Chat id={id} initialMessages={messages} isLoading={isLoading} />
  );
}
