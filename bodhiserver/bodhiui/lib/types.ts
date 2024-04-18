import { Message } from "ai/react"

export type ServerActionResult<Result> = Promise<
  | Result
  | {
    error: string
  }
>

export type ApiResult<R> = R | { error: string }

export interface Chat extends Record<string, any> {
  id: string
  title: string
  createdAt: Date
  userId: string
  path: string
  messages: Message[]
}
