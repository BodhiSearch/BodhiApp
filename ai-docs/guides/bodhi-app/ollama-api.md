# Ollama-Compatible API

BodhiApp provides full compatibility with Ollama's API endpoints, enabling seamless migration from Ollama to BodhiApp without code changes. This compatibility layer translates Ollama requests to BodhiApp's internal format while maintaining the same request/response structure.

## Overview

The Ollama-compatible API endpoints are available under the `/api/` path prefix and support the same request/response formats as Ollama. This compatibility enables:

- **Seamless Migration**: Switch from Ollama to BodhiApp without modifying existing code
- **Tool Compatibility**: Use existing Ollama-based tools and integrations
- **Client Library Support**: Compatible with Ollama client libraries
- **Local Privacy**: Keep all data processing local while maintaining API compatibility

## Authentication

All Ollama-compatible endpoints require authentication with **User** level access:

**Required**: `user` role OR `scope_token_user` OR `scope_user_user`

```typescript
const headers = {
  'Authorization': `Bearer ${apiToken}`,
  'Content-Type': 'application/json'
};
```

## Model Management Endpoints

### List Available Models

#### Endpoint: `GET /api/tags`

List all available models in Ollama format.

```typescript
const response = await fetch('http://localhost:1135/api/tags', {
  headers: { 'Authorization': `Bearer ${apiToken}` }
});

const models = await response.json();
```

**Response Format**:
```json
{
  "models": [
    {
      "model": "llama3:instruct",
      "modified_at": "2024-01-20T12:00:00.000000000Z",
      "size": 0,
      "digest": "5007652f7a641fe7170e0bad4f63839419bd9213",
      "details": {
        "parent_model": null,
        "format": "gguf",
        "family": "unknown",
        "families": null,
        "parameter_size": "",
        "quantization_level": ""
      }
    }
  ]
}
```

**Response Fields**:
- `model`: Model alias identifier
- `modified_at`: Model creation/modification timestamp in RFC3339 format
- `size`: Model file size (currently returns 0)
- `digest`: Model snapshot/commit identifier
- `details`: Model metadata including format and family information

### Show Model Details

#### Endpoint: `POST /api/show`

Get detailed information about a specific model.

```typescript
const response = await fetch('http://localhost:1135/api/show', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${apiToken}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    name: 'llama3:instruct'
  })
});

const modelDetails = await response.json();
```

**Request Format**:
```json
{
  "name": "llama3:instruct"
}
```

**Response Format**:
```json
{
  "details": {
    "parent_model": null,
    "format": "gguf",
    "family": "unknown",
    "families": null,
    "parameter_size": "",
    "quantization_level": ""
  },
  "license": "",
  "model_info": {},
  "modelfile": "",
  "modified_at": "2024-01-20T12:00:00.000000000Z",
  "parameters": "n_keep: 24\nstop:\n- <|start_header_id|>\n- <|end_header_id|>\n- <|eot_id|>\n",
  "template": ""
}
```

**Response Fields**:
- `details`: Model format and family information
- `license`: Model license information (empty in current implementation)
- `model_info`: Additional model metadata (empty object)
- `modelfile`: Ollama modelfile content (empty in current implementation)
- `modified_at`: Model modification timestamp
- `parameters`: Model configuration parameters in YAML format
- `template`: Chat template (empty as llama.cpp handles templates internally)

## Chat Completion Endpoint

### Chat with Model

#### Endpoint: `POST /api/chat`

Generate responses using Ollama's chat format.

```typescript
const response = await fetch('http://localhost:1135/api/chat', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${apiToken}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    model: 'llama3:instruct',
    messages: [
      {
        role: 'system',
        content: 'You are a helpful assistant.'
      },
      {
        role: 'user',
        content: 'Hello! How are you today?'
      }
    ],
    stream: false,
    options: {
      temperature: 0.7,
      num_predict: 100
    }
  })
});

const result = await response.json();
```

**Request Format**:
```json
{
  "model": "llama3:instruct",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant.",
      "images": null
    },
    {
      "role": "user", 
      "content": "Hello! How are you today?",
      "images": null
    }
  ],
  "stream": false,
  "format": null,
  "keep_alive": null,
  "options": {
    "temperature": 0.7,
    "num_predict": 100,
    "top_p": 0.9,
    "seed": 42
  }
}
```

### Request Parameters

#### Message Format
- `role`: Message role (`system`, `user`, `assistant`)
- `content`: Message content text
- `images`: Array of image URLs/data (optional, not currently supported)

#### Options Parameters
All options are optional and will use model alias defaults if not specified:

| Parameter | Type | Description |
|-----------|------|-------------|
| `num_keep` | integer | Number of tokens to keep from context |
| `seed` | integer | Random seed for reproducible outputs |
| `num_predict` | integer | Maximum tokens to generate |
| `top_k` | integer | Top-k sampling parameter |
| `top_p` | float | Top-p (nucleus) sampling parameter |
| `temperature` | float | Sampling temperature (0.0-2.0) |
| `repeat_penalty` | float | Repetition penalty |
| `presence_penalty` | float | Presence penalty |
| `frequency_penalty` | float | Frequency penalty |
| `stop` | string[] | Stop sequences |
| `num_ctx` | integer | Context window size |
| `num_thread` | integer | Number of CPU threads |

### Response Formats

#### Non-Streaming Response

```json
{
  "model": "llama3:instruct",
  "created_at": "2024-01-20T12:00:00.000000000Z",
  "message": {
    "role": "assistant",
    "content": "Hello! I'm doing well, thank you for asking. How can I help you today?",
    "images": null
  },
  "done": true,
  "done_reason": "stop",
  "total_duration": 0.0,
  "load_duration": "-1",
  "prompt_eval_count": 25,
  "prompt_eval_duration": "-1",
  "eval_count": 15,
  "eval_duration": "-1"
}
```

#### Streaming Response

When `stream: true`, responses are sent as Server-Sent Events:

```
data: {"model":"llama3:instruct","created_at":"2024-01-20T12:00:00.000000000Z","message":{"role":"assistant","content":"Hello","images":null},"done":false}

data: {"model":"llama3:instruct","created_at":"2024-01-20T12:00:00.000000000Z","message":{"role":"assistant","content":"!","images":null},"done":false}

data: {"model":"llama3:instruct","created_at":"2024-01-20T12:00:00.000000000Z","message":{"role":"assistant","content":"","images":null},"done":true}
```

**Response Fields**:
- `model`: Model identifier used for generation
- `created_at`: Response timestamp in RFC3339 format
- `message`: Generated message with role and content
- `done`: Whether generation is complete
- `done_reason`: Reason for completion (`stop`, `length`, etc.)
- `total_duration`: Total processing time (currently 0.0)
- `load_duration`: Model loading time (currently "-1")
- `prompt_eval_count`: Number of prompt tokens processed
- `prompt_eval_duration`: Prompt processing time (currently "-1")
- `eval_count`: Number of completion tokens generated
- `eval_duration`: Generation time (currently "-1")

## Advanced Usage Patterns

### Basic Chat Implementation

```typescript
class OllamaClient {
  private baseURL: string = 'http://localhost:1135';
  private apiToken: string;

  constructor(apiToken: string) {
    this.apiToken = apiToken;
  }

  async listModels() {
    const response = await fetch(`${this.baseURL}/api/tags`, {
      headers: { 'Authorization': `Bearer ${this.apiToken}` }
    });
    return response.json();
  }

  async showModel(name: string) {
    const response = await fetch(`${this.baseURL}/api/show`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.apiToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ name })
    });
    return response.json();
  }

  async chat(model: string, messages: Array<{role: string, content: string}>, options = {}) {
    const response = await fetch(`${this.baseURL}/api/chat`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.apiToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model,
        messages,
        stream: false,
        options
      })
    });
    return response.json();
  }

  async chatStream(model: string, messages: Array<{role: string, content: string}>, options = {}) {
    const response = await fetch(`${this.baseURL}/api/chat`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.apiToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model,
        messages,
        stream: true,
        options
      })
    });
    return response;
  }
}

// Usage
const client = new OllamaClient('your-api-token');

// List available models
const models = await client.listModels();
console.log('Available models:', models.models.map(m => m.model));

// Chat with a model
const response = await client.chat('llama3:instruct', [
  { role: 'system', content: 'You are a helpful assistant.' },
  { role: 'user', content: 'What is the capital of France?' }
], {
  temperature: 0.7,
  num_predict: 50
});

console.log('Response:', response.message.content);
```

### Streaming Chat Implementation

```typescript
async function streamingChat(client: OllamaClient, model: string, messages: any[]) {
  const response = await client.chatStream(model, messages);
  
  if (!response.body) {
    throw new Error('No response body');
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let fullContent = '';

  try {
    while (true) {
      const { done, value } = await reader.read();
      
      if (done) break;
      
      const chunk = decoder.decode(value);
      const lines = chunk.split('\n');
      
      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const data = line.slice(6);
          if (data.trim()) {
            try {
              const parsed = JSON.parse(data);
              process.stdout.write(parsed.message.content);
              fullContent += parsed.message.content;
              
              if (parsed.done) {
                console.log('\n--- Streaming complete ---');
                return fullContent;
              }
            } catch (e) {
              // Skip invalid JSON chunks
            }
          }
        }
      }
    }
  } finally {
    reader.releaseLock();
  }
  
  return fullContent;
}

// Usage
const fullResponse = await streamingChat(client, 'llama3:instruct', [
  { role: 'user', content: 'Write a short story about a robot.' }
]);
```

### Multi-turn Conversation

```typescript
class ConversationManager {
  private client: OllamaClient;
  private messages: Array<{role: string, content: string}> = [];

  constructor(client: OllamaClient, systemPrompt?: string) {
    this.client = client;
    if (systemPrompt) {
      this.messages.push({ role: 'system', content: systemPrompt });
    }
  }

  async sendMessage(content: string, model: string = 'llama3:instruct', options = {}) {
    // Add user message
    this.messages.push({ role: 'user', content });
    
    // Get response
    const response = await this.client.chat(model, this.messages, options);
    
    // Add assistant response to conversation
    this.messages.push({
      role: 'assistant',
      content: response.message.content
    });
    
    return response.message.content;
  }

  getConversation() {
    return [...this.messages];
  }

  clearConversation() {
    const systemMessage = this.messages.find(m => m.role === 'system');
    this.messages = systemMessage ? [systemMessage] : [];
  }
}

// Usage
const conversation = new ConversationManager(
  client,
  'You are a helpful coding assistant.'
);

const response1 = await conversation.sendMessage(
  'How do I create a REST API in Python?'
);
console.log('Assistant:', response1);

const response2 = await conversation.sendMessage(
  'Can you show me a simple example?'
);
console.log('Assistant:', response2);
```

## Migration from Ollama

### API Endpoint Mapping

| Ollama Endpoint | BodhiApp Equivalent | Status |
|----------------|---------------------|---------|
| `GET /api/tags` | `GET /api/tags` | ✅ Full compatibility |
| `POST /api/show` | `POST /api/show` | ✅ Full compatibility |
| `POST /api/chat` | `POST /api/chat` | ✅ Full compatibility |
| `POST /api/generate` | Not supported | ❌ Use `/api/chat` instead |
| `POST /api/pull` | Use `/bodhi/v1/modelfiles/pull` | ⚠️ Different endpoint |
| `DELETE /api/delete` | Not supported | ❌ Use BodhiApp UI |

### Code Migration Example

**Original Ollama Code**:
```typescript
// Ollama client
const response = await fetch('http://localhost:11434/api/chat', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    model: 'llama2',
    messages: [{ role: 'user', content: 'Hello' }]
  })
});
```

**BodhiApp Migration**:
```typescript
// BodhiApp client - just change URL and add authentication
const response = await fetch('http://localhost:1135/api/chat', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${apiToken}`,  // Add authentication
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    model: 'llama2:chat',  // Use model alias
    messages: [{ role: 'user', content: 'Hello' }]
  })
});
```

### Key Differences

1. **Authentication Required**: BodhiApp requires API token authentication
2. **Model Names**: Use BodhiApp model aliases (e.g., `llama3:instruct` vs `llama3`)
3. **Port**: Default port is 1135 instead of 11434
4. **Model Management**: Use BodhiApp's model management system instead of `ollama pull`

## Error Handling

### Common Error Responses

```typescript
// Model not found
{
  "error": "model not found"
}

// Invalid request parameters
{
  "error": "invalid request parameters"
}

// Chat completion error
{
  "error": "chat completion error: failed to process request"
}

// Authentication error (401)
{
  "error": {
    "message": "Authentication required",
    "type": "authentication_error",
    "code": "unauthorized"
  }
}
```

### Error Handling Implementation

```typescript
async function safeOllamaRequest(requestFn: () => Promise<Response>) {
  try {
    const response = await requestFn();
    
    if (!response.ok) {
      const error = await response.json();
      
      switch (response.status) {
        case 401:
          throw new Error('Authentication required - check your API token');
        case 404:
          if (error.error === 'model not found') {
            throw new Error('Model not found - check model name and availability');
          }
          throw new Error('Resource not found');
        case 400:
          throw new Error(`Invalid request: ${error.error}`);
        case 500:
          throw new Error(`Server error: ${error.error}`);
        default:
          throw new Error(`HTTP ${response.status}: ${error.error}`);
      }
    }
    
    return response.json();
  } catch (error) {
    if (error instanceof TypeError && error.message.includes('fetch')) {
      throw new Error('Unable to connect to BodhiApp server');
    }
    throw error;
  }
}

// Usage
try {
  const models = await safeOllamaRequest(() => 
    fetch('http://localhost:1135/api/tags', {
      headers: { 'Authorization': `Bearer ${apiToken}` }
    })
  );
  console.log('Models:', models);
} catch (error) {
  console.error('Request failed:', error.message);
}
```

## Best Practices

### Performance Optimization

1. **Model Reuse**: Keep using the same model to avoid loading overhead
2. **Context Management**: Manage conversation history to stay within context limits
3. **Streaming**: Use streaming for long responses to improve perceived performance
4. **Connection Reuse**: Reuse HTTP connections for multiple requests

### Resource Management

1. **Memory Usage**: Monitor memory usage with large conversations
2. **Token Limits**: Be aware of model context window limits
3. **Concurrent Requests**: Limit concurrent requests to avoid overwhelming the server
4. **Error Recovery**: Implement retry logic with exponential backoff

### Security Considerations

1. **Token Security**: Store API tokens securely, never in client-side code
2. **Input Validation**: Validate user inputs before sending to the API
3. **Rate Limiting**: Implement client-side rate limiting to avoid overwhelming the server
4. **Error Information**: Don't expose sensitive error information to end users

## Next Steps

Now that you understand the Ollama-compatible API:

1. **[Handle Errors](error-handling.md)** - Implement robust error handling
2. **[See Examples](examples.md)** - Complete integration examples  
3. **[API Reference](api-reference.md)** - Quick endpoint reference
4. **[Back to Overview](overview.md)** - System overview and capabilities

---

*The Ollama-compatible API provides a seamless migration path from Ollama while adding BodhiApp's enterprise features like authentication and advanced model management.* 