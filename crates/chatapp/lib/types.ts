export type ServerActionResult<Result> = Promise<
  | Result
  | {
    error: string
  }
>

export type ApiResult<R> = R | { error: string }

export interface ChatPreview {
  id: string,
  title: string,
  createdAt: number
}

export interface Message {
  role: 'system' | 'user' | 'assistant' | 'function' | 'data' | 'tool',
  content: string,
}

export interface Chat extends ChatPreview {
  messages: Message[]
}
