# Integration Examples

This guide provides complete, practical examples for integrating with BodhiApp APIs. Each example includes proper error handling, authentication, and follows best practices for production use.

## Overview

The examples in this guide demonstrate:

- **Complete Chat Applications**: Full-featured chat implementations
- **Model Management Workflows**: Downloading and configuring models
- **Authentication Patterns**: Secure token management
- **Error Handling**: Robust error recovery and retry logic
- **Performance Optimization**: Efficient API usage patterns

## Basic Chat Application

### Simple Chat Interface

A minimal chat application using the OpenAI-compatible API:

```typescript
import { useState } from 'react';

interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

interface ChatAppProps {
  apiToken: string;
  baseURL?: string;
}

export function ChatApp({ apiToken, baseURL = 'http://localhost:1135' }: ChatAppProps) {
  const [messages, setMessages] = useState<Message[]>([
    { role: 'system', content: 'You are a helpful assistant.' }
  ]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const sendMessage = async () => {
    if (!input.trim() || loading) return;

    const userMessage: Message = { role: 'user', content: input };
    const newMessages = [...messages, userMessage];
    setMessages(newMessages);
    setInput('');
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(`${baseURL}/v1/chat/completions`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${apiToken}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          model: 'llama3:instruct',
          messages: newMessages,
          temperature: 0.7,
          max_tokens: 1000
        })
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error?.message || 'Chat request failed');
      }

      const result = await response.json();
      const assistantMessage: Message = {
        role: 'assistant',
        content: result.choices[0].message.content
      };

      setMessages([...newMessages, assistantMessage]);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred');
      console.error('Chat error:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="chat-app">
      <div className="messages">
        {messages.filter(m => m.role !== 'system').map((message, index) => (
          <div key={index} className={`message ${message.role}`}>
            <strong>{message.role}:</strong> {message.content}
          </div>
        ))}
        {loading && <div className="message loading">Assistant is typing...</div>}
        {error && <div className="error">Error: {error}</div>}
      </div>
      
      <div className="input-area">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
          placeholder="Type your message..."
          disabled={loading}
        />
        <button onClick={sendMessage} disabled={loading || !input.trim()}>
          Send
        </button>
      </div>
    </div>
  );
}
```

### Streaming Chat Implementation

A more advanced chat with streaming responses:

```typescript
class StreamingChatClient {
  private apiToken: string;
  private baseURL: string;

  constructor(apiToken: string, baseURL: string = 'http://localhost:1135') {
    this.apiToken = apiToken;
    this.baseURL = baseURL;
  }

  async *streamChat(
    messages: Message[],
    model: string = 'llama3:instruct',
    options: any = {}
  ): AsyncGenerator<string, void, unknown> {
    const response = await fetch(`${this.baseURL}/v1/chat/completions`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.apiToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model,
        messages,
        stream: true,
        temperature: 0.7,
        max_tokens: 1000,
        ...options
      })
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(errorData.error?.message || 'Streaming request failed');
    }

    if (!response.body) {
      throw new Error('No response body for streaming');
    }

    const reader = response.body.getReader();
    const decoder = new TextDecoder();

    try {
      while (true) {
        const { done, value } = await reader.read();
        
        if (done) break;
        
        const chunk = decoder.decode(value, { stream: true });
        const lines = chunk.split('\n');
        
        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = line.slice(6).trim();
            
            if (data === '[DONE]') {
              return;
            }
            
            if (data) {
              try {
                const parsed = JSON.parse(data);
                const content = parsed.choices?.[0]?.delta?.content;
                if (content) {
                  yield content;
                }
              } catch (e) {
                // Skip invalid JSON chunks
                console.warn('Failed to parse streaming chunk:', data);
              }
            }
          }
        }
      }
    } finally {
      reader.releaseLock();
    }
  }
}

// Usage example
async function streamingChatExample() {
  const client = new StreamingChatClient('your-api-token');
  const messages = [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'Write a short story about a robot.' }
  ];

  let fullResponse = '';
  
  try {
    for await (const chunk of client.streamChat(messages)) {
      process.stdout.write(chunk);
      fullResponse += chunk;
    }
    console.log('\n--- Streaming complete ---');
    return fullResponse;
  } catch (error) {
    console.error('Streaming error:', error);
    throw error;
  }
}
```

## Model Management Examples

### Complete Model Management Workflow

```typescript
class ModelManager {
  private apiToken: string;
  private baseURL: string;

  constructor(apiToken: string, baseURL: string = 'http://localhost:1135') {
    this.apiToken = apiToken;
    this.baseURL = baseURL;
  }

  private async apiCall(endpoint: string, options: RequestInit = {}) {
    const response = await fetch(`${this.baseURL}${endpoint}`, {
      ...options,
      headers: {
        'Authorization': `Bearer ${this.apiToken}`,
        'Content-Type': 'application/json',
        ...options.headers
      }
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(errorData.error?.message || `HTTP ${response.status}`);
    }

    return response.json();
  }

  // List available models
  async listModels() {
    return this.apiCall('/bodhi/v1/models');
  }

  // List model files
  async listModelFiles() {
    return this.apiCall('/bodhi/v1/modelfiles');
  }

  // Download a model
  async downloadModel(repo: string, filename: string) {
    const downloadRequest = await this.apiCall('/bodhi/v1/modelfiles/pull', {
      method: 'POST',
      body: JSON.stringify({ repo, filename })
    });

    // Monitor download progress
    return this.monitorDownload(downloadRequest.id);
  }

  // Monitor download progress
  async monitorDownload(downloadId: string) {
    const checkStatus = async () => {
      return this.apiCall(`/bodhi/v1/modelfiles/pull/${downloadId}`);
    };

    return new Promise((resolve, reject) => {
      const interval = setInterval(async () => {
        try {
          const status = await checkStatus();
          
          console.log(`Download ${status.status}: ${status.progress || 0}%`);
          
          if (status.status === 'completed') {
            clearInterval(interval);
            resolve(status);
          } else if (status.status === 'failed') {
            clearInterval(interval);
            reject(new Error(status.error || 'Download failed'));
          }
        } catch (error) {
          clearInterval(interval);
          reject(error);
        }
      }, 2000);
    });
  }

  // Create model alias
  async createAlias(aliasConfig: {
    alias: string;
    repo: string;
    filename: string;
    snapshot?: string;
    request_params?: any;
    context_params?: any;
  }) {
    return this.apiCall('/bodhi/v1/models', {
      method: 'POST',
      body: JSON.stringify(aliasConfig)
    });
  }

  // Update model alias
  async updateAlias(alias: string, updates: any) {
    return this.apiCall(`/bodhi/v1/models/${alias}`, {
      method: 'PUT',
      body: JSON.stringify(updates)
    });
  }

  // Complete setup workflow
  async setupModel(
    repo: string,
    filename: string,
    alias: string,
    config: any = {}
  ) {
    console.log(`Setting up model: ${alias}`);
    
    // 1. Check if model file exists
    const modelFiles = await this.listModelFiles();
    const existingFile = modelFiles.data.find(
      f => f.repo === repo && f.filename === filename
    );

    if (!existingFile) {
      console.log('Model file not found, downloading...');
      await this.downloadModel(repo, filename);
      console.log('Download completed');
    } else {
      console.log('Model file already exists');
    }

    // 2. Check if alias exists
    const models = await this.listModels();
    const existingAlias = models.data.find(m => m.alias === alias);

    if (existingAlias) {
      console.log('Alias exists, updating configuration...');
      await this.updateAlias(alias, config);
    } else {
      console.log('Creating new alias...');
      await this.createAlias({
        alias,
        repo,
        filename,
        snapshot: 'main',
        ...config
      });
    }

    console.log(`Model setup complete: ${alias}`);
    return { alias, repo, filename };
  }
}

// Usage example
async function modelSetupExample() {
  const manager = new ModelManager('your-api-token');

  try {
    // Set up a chat model
    await manager.setupModel(
      'microsoft/Phi-3-mini-4k-instruct-gguf',
      'Phi-3-mini-4k-instruct-q4.gguf',
      'phi3:chat',
      {
        request_params: {
          temperature: 0.7,
          max_tokens: 2048,
          top_p: 0.95
        },
        context_params: {
          n_ctx: 4096,
          n_threads: 8
        }
      }
    );

    // Set up a code model with different parameters
    await manager.setupModel(
      'microsoft/Phi-3-mini-4k-instruct-gguf',
      'Phi-3-mini-4k-instruct-q4.gguf',
      'phi3:code',
      {
        request_params: {
          temperature: 0.1,
          max_tokens: 4096,
          stop: ['```\n\n', '# End']
        },
        context_params: {
          n_ctx: 8192,
          n_predict: 4096
        }
      }
    );

    console.log('All models set up successfully');
  } catch (error) {
    console.error('Model setup failed:', error);
  }
}
```

## Production-Ready Chat Application

### Complete Chat Application with All Features

```typescript
interface ChatConfig {
  apiToken: string;
  baseURL?: string;
  defaultModel?: string;
  maxRetries?: number;
  retryDelay?: number;
}

interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  model?: string;
  error?: boolean;
}

class ProductionChatClient {
  private config: Required<ChatConfig>;
  private retryCount = new Map<string, number>();

  constructor(config: ChatConfig) {
    this.config = {
      baseURL: 'http://localhost:1135',
      defaultModel: 'llama3:instruct',
      maxRetries: 3,
      retryDelay: 1000,
      ...config
    };
  }

  // Send message with full error handling and retry logic
  async sendMessage(
    messages: ChatMessage[],
    options: {
      model?: string;
      temperature?: number;
      max_tokens?: number;
      stream?: boolean;
    } = {}
  ): Promise<ChatMessage> {
    const requestId = Math.random().toString(36).substring(7);
    const model = options.model || this.config.defaultModel;
    
    const requestMessages = messages
      .filter(m => !m.error)
      .map(m => ({ role: m.role, content: m.content }));

    return this.withRetry(requestId, async () => {
      const response = await fetch(`${this.config.baseURL}/v1/chat/completions`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${this.config.apiToken}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          model,
          messages: requestMessages,
          temperature: options.temperature || 0.7,
          max_tokens: options.max_tokens || 1000,
          stream: options.stream || false
        })
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new ChatError(
          errorData.error?.message || 'Request failed',
          errorData.error?.code || 'unknown_error',
          response.status
        );
      }

      const result = await response.json();
      
      return {
        id: Math.random().toString(36).substring(7),
        role: 'assistant' as const,
        content: result.choices[0].message.content,
        timestamp: new Date(),
        model
      };
    });
  }

  // Stream message with proper error handling
  async *streamMessage(
    messages: ChatMessage[],
    options: any = {}
  ): AsyncGenerator<{ chunk: string; complete: boolean }, ChatMessage, unknown> {
    const model = options.model || this.config.defaultModel;
    const requestMessages = messages
      .filter(m => !m.error)
      .map(m => ({ role: m.role, content: m.content }));

    const response = await fetch(`${this.config.baseURL}/v1/chat/completions`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.config.apiToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model,
        messages: requestMessages,
        stream: true,
        ...options
      })
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new ChatError(
        errorData.error?.message || 'Streaming failed',
        errorData.error?.code || 'unknown_error',
        response.status
      );
    }

    let fullContent = '';
    const reader = response.body!.getReader();
    const decoder = new TextDecoder();

    try {
      while (true) {
        const { done, value } = await reader.read();
        
        if (done) break;
        
        const chunk = decoder.decode(value, { stream: true });
        const lines = chunk.split('\n');
        
        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = line.slice(6).trim();
            
            if (data === '[DONE]') {
              yield { chunk: '', complete: true };
              return {
                id: Math.random().toString(36).substring(7),
                role: 'assistant' as const,
                content: fullContent,
                timestamp: new Date(),
                model
              };
            }
            
            if (data) {
              try {
                const parsed = JSON.parse(data);
                const content = parsed.choices?.[0]?.delta?.content || '';
                if (content) {
                  fullContent += content;
                  yield { chunk: content, complete: false };
                }
              } catch (e) {
                // Skip invalid JSON
              }
            }
          }
        }
      }
    } finally {
      reader.releaseLock();
    }

    return {
      id: Math.random().toString(36).substring(7),
      role: 'assistant' as const,
      content: fullContent,
      timestamp: new Date(),
      model
    };
  }

  // Retry logic with exponential backoff
  private async withRetry<T>(
    requestId: string,
    operation: () => Promise<T>
  ): Promise<T> {
    const maxRetries = this.config.maxRetries;
    let lastError: Error;

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        const result = await operation();
        this.retryCount.delete(requestId);
        return result;
      } catch (error) {
        lastError = error as Error;
        
        // Don't retry on client errors
        if (error instanceof ChatError && error.status < 500) {
          throw error;
        }

        if (attempt === maxRetries) {
          break;
        }

        // Exponential backoff with jitter
        const delay = this.config.retryDelay * Math.pow(2, attempt) + 
                     Math.random() * 1000;
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }

    this.retryCount.set(requestId, (this.retryCount.get(requestId) || 0) + 1);
    throw lastError!;
  }

  // Get available models
  async getModels() {
    const response = await fetch(`${this.config.baseURL}/bodhi/v1/models`, {
      headers: { 'Authorization': `Bearer ${this.config.apiToken}` }
    });

    if (!response.ok) {
      throw new Error('Failed to fetch models');
    }

    return response.json();
  }

  // Health check
  async healthCheck() {
    try {
      const response = await fetch(`${this.config.baseURL}/ping`);
      return response.ok;
    } catch {
      return false;
    }
  }
}

class ChatError extends Error {
  constructor(
    message: string,
    public code: string,
    public status: number
  ) {
    super(message);
    this.name = 'ChatError';
  }
}

// React component using the production client
function ProductionChatApp({ apiToken }: { apiToken: string }) {
  const [client] = useState(() => new ProductionChatClient({ apiToken }));
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [models, setModels] = useState<any[]>([]);
  const [selectedModel, setSelectedModel] = useState('llama3:instruct');

  useEffect(() => {
    // Load available models
    client.getModels().then(result => {
      setModels(result.data || []);
    }).catch(console.error);

    // Health check
    client.healthCheck().then(healthy => {
      if (!healthy) {
        setError('BodhiApp server is not responding');
      }
    });
  }, [client]);

  const sendMessage = async () => {
    if (!input.trim() || loading) return;

    const userMessage: ChatMessage = {
      id: Math.random().toString(36).substring(7),
      role: 'user',
      content: input,
      timestamp: new Date()
    };

    const newMessages = [...messages, userMessage];
    setMessages(newMessages);
    setInput('');
    setLoading(true);
    setError(null);

    try {
      const assistantMessage = await client.sendMessage(newMessages, {
        model: selectedModel
      });
      setMessages([...newMessages, assistantMessage]);
    } catch (err) {
      const errorMessage: ChatMessage = {
        id: Math.random().toString(36).substring(7),
        role: 'assistant',
        content: err instanceof Error ? err.message : 'An error occurred',
        timestamp: new Date(),
        error: true
      };
      setMessages([...newMessages, errorMessage]);
      setError(err instanceof Error ? err.message : 'An error occurred');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="production-chat-app">
      <div className="chat-header">
        <select 
          value={selectedModel} 
          onChange={(e) => setSelectedModel(e.target.value)}
          disabled={loading}
        >
          {models.map(model => (
            <option key={model.alias} value={model.alias}>
              {model.alias}
            </option>
          ))}
        </select>
      </div>

      <div className="messages">
        {messages.map(message => (
          <div 
            key={message.id} 
            className={`message ${message.role} ${message.error ? 'error' : ''}`}
          >
            <div className="message-header">
              <strong>{message.role}</strong>
              <span className="timestamp">
                {message.timestamp.toLocaleTimeString()}
              </span>
              {message.model && (
                <span className="model">{message.model}</span>
              )}
            </div>
            <div className="message-content">{message.content}</div>
          </div>
        ))}
        {loading && <div className="loading">Assistant is thinking...</div>}
      </div>

      {error && (
        <div className="error-banner">
          Error: {error}
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      <div className="input-area">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
          placeholder="Type your message..."
          disabled={loading}
        />
        <button onClick={sendMessage} disabled={loading || !input.trim()}>
          {loading ? 'Sending...' : 'Send'}
        </button>
      </div>
    </div>
  );
}
```

## Best Practices Summary

### Performance Optimization

1. **Connection Reuse**: Use a single HTTP client instance
2. **Request Batching**: Combine multiple operations when possible
3. **Streaming**: Use streaming for long responses
4. **Caching**: Cache model lists and configuration data
5. **Resource Management**: Monitor memory usage and cleanup

### Error Handling

1. **Specific Error Types**: Handle different error codes appropriately
2. **Retry Logic**: Implement exponential backoff for server errors
3. **User Feedback**: Provide clear error messages to users
4. **Logging**: Log errors with sufficient context for debugging
5. **Graceful Degradation**: Provide fallback behaviors

### Security

1. **Token Storage**: Store API tokens securely
2. **Token Rotation**: Regularly rotate long-lived tokens
3. **Input Validation**: Validate all user inputs
4. **Error Information**: Don't expose sensitive data in error messages
5. **Rate Limiting**: Implement client-side rate limiting

### Development Workflow

1. **Testing**: Include error scenarios in tests
2. **Monitoring**: Track API usage and error rates
3. **Documentation**: Document integration patterns
4. **Version Management**: Handle API version changes gracefully
5. **Configuration**: Use environment-specific configurations

## Next Steps

Now that you have comprehensive examples:

1. **[API Reference](api-reference.md)** - Quick endpoint reference
2. **[Error Handling](error-handling.md)** - Detailed error handling guide
3. **[Authentication](authentication.md)** - Security best practices
4. **[Back to Overview](overview.md)** - System overview and capabilities

---

*These examples provide production-ready patterns for integrating with BodhiApp. Adapt them to your specific use case while maintaining the error handling and security practices demonstrated.* 