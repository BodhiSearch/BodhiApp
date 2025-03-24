import { describe, it, expect, vi, beforeEach } from 'vitest';
import { BodhiClient } from './client';
import type { operations } from './types/api';

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch;

describe('BodhiClient', () => {
  let client: BodhiClient;

  beforeEach(() => {
    mockFetch.mockReset();
    client = new BodhiClient({
      baseUrl: 'https://api.example.com',
      apiKey: 'test-api-key'
    });
  });

  it('should construct with correct default headers', () => {
    // Verify headers by making a request
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ id: 'test' })
    });

    return client.request('GET', '/test').then(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/test',
        expect.objectContaining({
          headers: {
            'Content-Type': 'application/json',
            'Authorization': 'Bearer test-api-key'
          }
        })
      );
    });
  });

  it('should create chat completion', async () => {
    const mockResponse: operations['createChatCompletion']['responses']['200']['content']['application/json'] = {
      id: 'cmpl-123',
      object: 'chat.completion',
      created: 1677858242,
      model: 'gpt-3.5-turbo',
      choices: [{
        message: {
          role: 'assistant',
          content: 'Hello, how can I help you?'
        },
        index: 0,
        finish_reason: 'stop'
      }]
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    });

    const result = await client.createChatCompletion({
      model: 'gpt-3.5-turbo',
      messages: [{ role: 'user', content: 'Hello' }]
    });

    expect(result).toEqual(mockResponse);
    expect(mockFetch).toHaveBeenCalledWith(
      'https://api.example.com/v1/chat/completions',
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({
          model: 'gpt-3.5-turbo',
          messages: [{ role: 'user', content: 'Hello' }]
        })
      })
    );
  });

  it('should list models', async () => {
    const mockResponse: operations['listModels']['responses']['200']['content']['application/json'] = {
      object: 'list',
      data: [
        { id: 'model-1', object: 'model' },
        { id: 'model-2', object: 'model' }
      ]
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockResponse
    });

    const result = await client.listModels();

    expect(result).toEqual(mockResponse);
    expect(mockFetch).toHaveBeenCalledWith(
      'https://api.example.com/v1/models',
      expect.objectContaining({
        method: 'GET'
      })
    );
  });

  it('should throw error on failed request', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 401,
      text: async () => 'Invalid API key'
    });

    await expect(client.request('GET', '/test')).rejects.toThrow(
      'Request failed with status 401: Invalid API key'
    );
  });
}); 