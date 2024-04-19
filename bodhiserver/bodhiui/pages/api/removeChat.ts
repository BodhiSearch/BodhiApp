import { userId } from '@/lib/utils';
import { kv } from '@vercel/kv'
import { NextApiRequest, NextApiResponse } from 'next'

export default async function removeChat(req: NextApiRequest, res: NextApiResponse) {
  let { id } = req.query;
  await kv.del(`chat:${id}`)
  await kv.zrem(`user:chat:${userId}`, `chat:${id}`)
  return res.status(200).json({})
}
