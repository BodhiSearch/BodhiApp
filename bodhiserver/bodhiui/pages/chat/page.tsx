import { Chat } from "@/components/chat";
import { getChat } from "@/lib/backend";
import { Root } from "@/lib/utils";
import { useSearchParams } from "next/navigation";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

export default function ChatPage() {
  const query = useSearchParams();
  const [messages, setMessages] = useState([])
  const router = useRouter();
  const id = query?.get('id');
  useEffect(() => {
    (async () => {
      if (!id) {
        return;
      }
      let { data, status } = await getChat(id);
      if (status !== 200) {
        setMessages(data)
      }
    })()
  }, [id]);
  if (!id) {
    router.push(Root).then(() => { }).catch((err) => { console.log(`id missing, error routing to home: ${err}`) });
    return;
  }
  return (
    messages.length === 0 ? <>Loading...</> : <Chat id={id} initialMessages={messages} />
  );
}
