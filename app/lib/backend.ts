import axios from 'axios'
import { API_BASE_URL } from '@/lib/utils'
import { type Chat } from '@/lib/types'

export const client = axios.create()

export async function getChats() {
  let { data, status } = await client.get(`${API_BASE_URL}api/ui/chats`)
  return { data, status }
}

export async function getChat(id: string) {
  let { data, status } = await client.get(`${API_BASE_URL}api/ui/chats/${id}`)
  return { data, status }
}

export async function updateChat(chat: Chat) {
  let { data, status } = await client.post(`${API_BASE_URL}api/ui/chats/${chat.id}`, chat)
  return { data, status }
}

export async function removeChat(id: string) {
  let { data, status } = await client.delete(`${API_BASE_URL}api/ui/chats/${id}`)
  return { data, status }
}

export async function clearChats() {
  let { data, status } = await client.delete(`${API_BASE_URL}api/ui/chats`)
  return { data, status }
}

// TODO - check openai api to get models
export async function getModels() {
  let { data, status } = await client.get(`${API_BASE_URL}api/ui/models`)
  return { data, status }
}
