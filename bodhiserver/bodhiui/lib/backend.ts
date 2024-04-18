import axios from 'axios'

export const client = axios.create()
export const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export async function getChats() {
  let { data, status } = await client.get(`${API_BASE_URL}/api/getChats`) // TODO - change to RESTful
  return { data, status }
}

export async function removeChat(id: string) {
  let { data, status } = await client.post(`${API_BASE_URL}/api/removeChat?id=${id}`) // TODO - change to RESTful
  return { data, status }
}

export async function clearChats() {
  let { data, status } = await client.delete(`${API_BASE_URL}/api/clearChats`) // TODO - change to RESTful
  return { data, status }
}
