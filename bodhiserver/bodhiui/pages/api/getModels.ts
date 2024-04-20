import { NextApiRequest, NextApiResponse } from 'next';

export default async function getChat(req: NextApiRequest, res: NextApiResponse) {
  return res.status(200).json(['llama-2-13b-chat', 'ggml-starcoder2-15b-q8_0', 'llama-2-7b-chat.q4_K_M', 'gpt-3.5-turbo', 'gpt-4-turbo'])
}
