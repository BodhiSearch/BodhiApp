import { kv } from '@vercel/kv'
import { NextApiRequest, NextApiResponse } from 'next';
import { type Chat } from '@/lib/types'
import { userId } from '@/lib/utils';

export default async function getChat(req: NextApiRequest, res: NextApiResponse) {
  let { chatId } = req.query;
  const chat = await kv.hgetall<Chat>(`chat:${chatId}`)
  if (!chat || (userId && chat.userId !== userId)) {
    res.status(404);
  } else {
    res.status(200).json(chat)
  }
}
