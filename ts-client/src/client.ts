import type { operations } from './types/api';

export interface ClientOptions {
  baseUrl: string;
  apiKey?: string;
  headers?: Record<string, string>;
}

export class BodhiClient {
  private baseUrl: string;
  private headers: Record<string, string>;

  constructor(options: ClientOptions) {
    this.baseUrl = options.baseUrl.endsWith('/') ? options.baseUrl.slice(0, -1) : options.baseUrl;
    this.headers = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    if (options.apiKey) {
      this.headers['Authorization'] = `Bearer ${options.apiKey}`;
    }
  }

  /**
   * Make a request to the Bodhi API
   */
  public async request<T>(
    method: string,
    path: string,
    body?: unknown,
    additionalHeaders?: Record<string, string>
  ): Promise<T> {
    const url = `${this.baseUrl}${path.startsWith('/') ? path : `/${path}`}`;

    const response = await fetch(url, {
      method,
      headers: {
        ...this.headers,
        ...additionalHeaders,
      },
      body: body ? JSON.stringify(body) : undefined,
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Request failed with status ${response.status}: ${errorText}`);
    }

    return response.json() as Promise<T>;
  }

  /**
   * Create a chat completion
   */
  public async createChatCompletion(
    params: operations['createChatCompletion']['requestBody']['content']['application/json']
  ): Promise<operations['createChatCompletion']['responses']['200']['content']['application/json']> {
    return this.request('POST', '/v1/chat/completions', params);
  }

  /**
   * List available models
   */
  public async listModels(): Promise<operations['listModels']['responses']['200']['content']['application/json']> {
    return this.request('GET', '/v1/models');
  }
} 