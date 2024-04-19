import { NextApiRequest, NextApiResponse } from "next"
import { kv } from '@vercel/kv'
import { userId } from "@/lib/utils";

export default async function clearChats(req: NextApiRequest, res: NextApiResponse) {
  try {
    const chats: string[] = await kv.zrange(`user:chat:${userId}`, 0, -1)
    if (!chats.length) {
      return res.status(200).send({ message: 'No chats found' });
    }
    let totalChats = chats.length;
    const pipeline = kv.pipeline()
    for (const chat of chats) {
      pipeline.del(chat)
      pipeline.zrem(`user:chat:${userId}`, chat)
    }
    await pipeline.exec()
    return res.status(200).send({ message: `${totalChats} chats deleted` });
  } catch (err) {
    return res.status(500).send({ error: `error deleting chat history: ${err}` })
  }
}