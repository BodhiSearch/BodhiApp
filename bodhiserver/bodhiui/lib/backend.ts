import axios from 'axios'

export const client = axios.create()
export const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export async function getChats() {
  let endpoint = `${API_BASE_URL}/api/getChats`;
  console.log(`calling ${endpoint}`)
  let { data, status } = await client.get(endpoint)
  return { data, status }
}

export async function removeChat(id: string) {
  let { data, status } = await client.post(`${API_BASE_URL}/chats/${id}`)
  return { data, status }
}

export async function clearChats() {
  let { data, status } = await client.delete(`${API_BASE_URL}/chats/clear`)
  return { data, status }
}
