import { NextApiRequest, NextApiResponse } from "next"
import { kv } from '@vercel/kv'

export default async function saveChat(req: NextApiRequest, res: NextApiResponse) {
  try {
    let chat = req.body;
    const pipeline = kv.pipeline()
    pipeline.hmset(`chat:${chat.id}`, chat)
    pipeline.zadd(`user:chat:${chat.userId}`, {
      score: Date.now(),
      member: `chat:${chat.id}`
    })
    await pipeline.exec()
    return res.status(201).send({});
  } catch (error) {
    console.log(`error in saveChat: ${error}`);
  }
}