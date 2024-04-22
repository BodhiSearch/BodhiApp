import axios from 'axios'
import { API_BASE_URL } from '@/lib/utils'

export const client = axios.create()

export async function getChats() {
  let { data, status } = await client.get(`${API_BASE_URL}/api/getChats`) // TODO - change to RESTful
  return { data, status }
}

export async function getChat(id: string) {
  let { data, status } = await client.get(`${API_BASE_URL}/api/getChat?id=${id}`) // TODO - change to RESTful
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

export async function getModels() {
  let { data, status } = await client.get(`${API_BASE_URL}/api/getModels`) // TODO - change to RESTful
  return { data, status }
}
