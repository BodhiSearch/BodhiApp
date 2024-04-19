import { NextApiRequest, NextApiResponse } from 'next'
import { kv } from '@vercel/kv'
import { userId } from '@/lib/utils'

export default async function getChats(req: NextApiRequest, res: NextApiResponse) {
  try {
    const pipeline = kv.pipeline()
    const chats: string[] = await kv.zrange(`user:chat:${userId}`, 0, -1, {
      rev: true
    })

    for (const chat of chats) {
      pipeline.hgetall(chat)
    }

    const results = await pipeline.exec()
    if (results.length != 0) {
      res.status(200).json(results);
    } else {
      res.status(200).json([]);
    }
  } catch (error) {
    if ((error as Error).message === "Pipeline is empty") {
      return res.status(200).json([]);
    }
    console.log(`error in getChats: ${error}`);
    res.status(500).json({ error: (error as Error).message })
  }
}