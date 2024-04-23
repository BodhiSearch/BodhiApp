import axios from 'axios'
import { API_BASE_URL } from '@/lib/utils'

export const client = axios.create()

export async function getChats() {
  let { data, status } = await client.get(`${API_BASE_URL}/api/ui/chats`)
  return { data, status }
}

export async function getChat(id: string) {
  let { data, status } = await client.get(`${API_BASE_URL}/api/ui/chats?id=${id}`)
  return { data, status }
}

export async function removeChat(id: string) {
  let { data, status } = await client.delete(`${API_BASE_URL}/api/ui/chats?id=${id}`)
  return { data, status }
}

export async function clearChats() {
  let { data, status } = await client.delete(`${API_BASE_URL}/api/ui/chats`)
  return { data, status }
}

// TODO - check openai api to get models
export async function getModels() {
  let { data, status } = await client.get(`${API_BASE_URL}/api/ui/models`)
  return { data, status }
}
