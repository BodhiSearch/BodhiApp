import OpenAI from 'openai';
import { OpenAIStream, StreamingTextResponse } from 'ai';
import { nanoid } from 'nanoid';
import { kv } from '@vercel/kv'

const openai = new OpenAI({
  apiKey: process.env.OPENAI_API_KEY!,
});

export async function POST(req: Request) {
  console.log(`incoming request: ${JSON.stringify(req)}`)
  const json = await req.json();
  const {messages} = json;
  const response = await openai.chat.completions.create({
    model: 'gpt-3.5-turbo',
    stream: true,
    messages: messages,
  });
  const userId = '29175b6f-44ed-4901-a35b-460c48c1b171'
  const stream = OpenAIStream(response, {
    async onCompletion(completion) {
      const title = json.messages[0].content.substring(0, 100)
      const id = json.id ?? nanoid()
      const createdAt = Date.now()
      const path = `/chat?id=${id}`
      const payload = {
        id,
        title,
        userId,
        createdAt,
        path,
        messages: [
          ...messages,
          {
            content: completion,
            role: 'assistant'
          }
        ]
      }
      await kv.hmset(`chat:${id}`, payload)
      await kv.zadd(`user:chat:${userId}`, {
        score: createdAt,
        member: `chat:${id}`
      })
      console.log(`added chats to `)
    },
  });
  return new StreamingTextResponse(stream);
}
