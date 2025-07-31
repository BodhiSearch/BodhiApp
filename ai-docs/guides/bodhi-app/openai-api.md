# OpenAI-Compatible API

BodhiApp provides full compatibility with OpenAI's API, making it a drop-in replacement for OpenAI's endpoints. This allows you to use existing OpenAI client libraries and tools with your local LLM inference.

## Overview

The OpenAI-compatible API endpoints are available under the `/v1/` path prefix and support the same request/response formats as OpenAI's API. This compatibility enables:

- **Seamless Migration**: Switch from OpenAI to BodhiApp without code changes
- **Client Library Support**: Use official OpenAI SDKs and third-party libraries
- **Tool Integration**: Compatible with existing OpenAI-based tools and services
- **Local Privacy**: Keep all data processing local while maintaining API compatibility

## Authentication

All OpenAI-compatible endpoints require authentication with **User** level access:

**Required**: `user` role OR `scope_token_user` OR `scope_user_user`

```typescript
const headers = {
  'Authorization': `Bearer ${apiToken}`,
  'Content-Type': 'application/json'
};
```

## Chat Completions API

### Endpoint: `POST /v1/chat/completions`

The chat completions endpoint is the primary interface for generating AI responses using the OpenAI chat format.

#### Basic Usage

```typescript
const response = await fetch('http://localhost:1135/v1/chat/completions', {
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
    max_tokens: 150,
    temperature: 0.7
  })
});

const result = await response.json();
console.log(result.choices[0].message.content);
```

#### Using OpenAI SDK

```typescript
import OpenAI from 'openai';

const client = new OpenAI({
  apiKey: 'your-api-token-here',
  baseURL: 'http://localhost:1135/v1'
});

const completion = await client.chat.completions.create({
  model: 'llama3:instruct',
  messages: [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'Explain quantum computing in simple terms.' }
  ],
  max_tokens: 200,
  temperature: 0.8
});

console.log(completion.choices[0].message.content);
```

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `model` | string | Yes | Model alias to use for completion |
| `messages` | array | Yes | Array of message objects |
| `max_tokens` | integer | No | Maximum tokens to generate (default: model-dependent) |
| `temperature` | number | No | Sampling temperature 0.0-2.0 (default: 1.0) |
| `top_p` | number | No | Nucleus sampling parameter 0.0-1.0 (default: 1.0) |
| `frequency_penalty` | number | No | Frequency penalty -2.0 to 2.0 (default: 0.0) |
| `presence_penalty` | number | No | Presence penalty -2.0 to 2.0 (default: 0.0) |
| `stop` | string/array | No | Stop sequences |
| `stream` | boolean | No | Enable streaming responses (default: false) |

#### Message Format

```typescript
interface Message {
  role: 'system' | 'user' | 'assistant';
  content: string;
}
```

#### Response Format

**Non-streaming Response**:
```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "llama3:instruct",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! I'm doing well, thank you for asking..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 25,
    "completion_tokens": 32,
    "total_tokens": 57
  }
}
```

### Streaming Responses

Enable real-time response streaming by setting `stream: true`:

```typescript
const response = await fetch('http://localhost:1135/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${apiToken}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    model: 'llama3:instruct',
    messages: [
      { role: 'user', content: 'Tell me a story' }
    ],
    stream: true
  })
});

const reader = response.body?.getReader();
const decoder = new TextDecoder();

while (true) {
  const { done, value } = await reader.read();
  if (done) break;
  
  const chunk = decoder.decode(value);
  const lines = chunk.split('\n').filter(line => line.trim());
  
  for (const line of lines) {
    if (line.startsWith('data: ')) {
      const data = line.slice(6);
      if (data === '[DONE]') return;
      
      try {
        const parsed = JSON.parse(data);
        const content = parsed.choices[0].delta.content;
        if (content) {
          process.stdout.write(content);
        }
      } catch (e) {
        // Skip malformed chunks
      }
    }
  }
}
```

#### Using OpenAI SDK for Streaming

```typescript
const stream = await client.chat.completions.create({
  model: 'llama3:instruct',
  messages: [
    { role: 'user', content: 'Write a short poem about AI' }
  ],
  stream: true
});

for await (const chunk of stream) {
  const content = chunk.choices[0]?.delta?.content || '';
  process.stdout.write(content);
}
```

## Models API

### Endpoint: `GET /v1/models`

List all available models (model aliases) in OpenAI format.

#### Basic Usage

```typescript
const response = await fetch('http://localhost:1135/v1/models', {
  headers: {
    'Authorization': `Bearer ${apiToken}`
  }
});

const models = await response.json();
console.log(models.data); // Array of model objects
```

#### Using OpenAI SDK

```typescript
const models = await client.models.list();
console.log(models.data);
```

#### Response Format

```json
{
  "object": "list",
  "data": [
    {
      "id": "llama3:instruct",
      "object": "model",
      "created": 1677652288,
      "owned_by": "system"
    },
    {
      "id": "phi3:mini",
      "object": "model", 
      "created": 1677652300,
      "owned_by": "system"
    }
  ]
}
```

### Endpoint: `GET /v1/models/{model_id}`

Get details about a specific model.

#### Basic Usage

```typescript
const response = await fetch('http://localhost:1135/v1/models/llama3:instruct', {
  headers: {
    'Authorization': `Bearer ${apiToken}`
  }
});

const model = await response.json();
```

#### Using OpenAI SDK

```typescript
const model = await client.models.retrieve('llama3:instruct');
console.log(model);
```

#### Response Format

```json
{
  "id": "llama3:instruct",
  "object": "model",
  "created": 1677652288,
  "owned_by": "system"
}
```

## Error Handling

BodhiApp returns OpenAI-compatible error responses:

### Common Error Codes

| HTTP Status | Error Type | Description |
|-------------|------------|-------------|
| 400 | `invalid_request_error` | Invalid request format or parameters |
| 401 | `invalid_request_error` | Invalid or missing API token |
| 404 | `invalid_request_error` | Model not found |
| 500 | `api_error` | Internal server error |

### Error Response Format

```json
{
  "error": {
    "message": "Model 'nonexistent:model' not found",
    "type": "invalid_request_error",
    "code": "model_not_found"
  }
}
```

### Error Handling in Code

```typescript
try {
  const completion = await client.chat.completions.create({
    model: 'invalid-model',
    messages: [{ role: 'user', content: 'Hello' }]
  });
} catch (error) {
  if (error.status === 404) {
    console.error('Model not found:', error.message);
  } else if (error.status === 401) {
    console.error('Authentication failed:', error.message);
  } else {
    console.error('API error:', error.message);
  }
}
```

## Advanced Usage Patterns

### Multi-turn Conversations

```typescript
class ChatSession {
  private messages: Array<{role: string, content: string}> = [];
  
  constructor(systemPrompt?: string) {
    if (systemPrompt) {
      this.messages.push({ role: 'system', content: systemPrompt });
    }
  }
  
  async sendMessage(content: string): Promise<string> {
    this.messages.push({ role: 'user', content });
    
    const completion = await client.chat.completions.create({
      model: 'llama3:instruct',
      messages: this.messages,
      max_tokens: 200
    });
    
    const response = completion.choices[0].message.content;
    this.messages.push({ role: 'assistant', content: response });
    
    return response;
  }
}

// Usage
const chat = new ChatSession('You are a helpful coding assistant.');
const response1 = await chat.sendMessage('How do I create a REST API?');
const response2 = await chat.sendMessage('Can you show me an example?');
```

### Function Calling Simulation

While BodhiApp doesn't support native function calling, you can simulate it:

```typescript
async function simulateFunctionCall(userQuery: string) {
  const systemPrompt = `You are an assistant that can call functions. 
When you need to call a function, respond with JSON in this format:
{"function_call": {"name": "function_name", "arguments": {"arg1": "value1"}}}

Available functions:
- get_weather(location: string): Get weather information
- calculate(expression: string): Perform calculations`;

  const completion = await client.chat.completions.create({
    model: 'llama3:instruct',
    messages: [
      { role: 'system', content: systemPrompt },
      { role: 'user', content: userQuery }
    ]
  });

  const response = completion.choices[0].message.content;
  
  try {
    const parsed = JSON.parse(response);
    if (parsed.function_call) {
      // Handle function call
      return await handleFunctionCall(parsed.function_call);
    }
  } catch {
    // Regular response
    return response;
  }
}
```

### Custom Model Parameters

Use model aliases to set custom parameters:

```typescript
// Create a model alias with custom parameters via BodhiApp API
await fetch('http://localhost:1135/bodhi/v1/models', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${powerUserToken}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    alias: 'creative-writer',
    model_file: 'llama3-8b-instruct.gguf',
    request_params: {
      temperature: 1.2,
      top_p: 0.9,
      max_tokens: 500
    },
    context_params: {
      n_ctx: 4096
    }
  })
});

// Use the custom alias in OpenAI API
const completion = await client.chat.completions.create({
  model: 'creative-writer',
  messages: [
    { role: 'user', content: 'Write a creative story about time travel' }
  ]
  // Parameters from alias are automatically applied
});
```

## Migration from OpenAI

### Configuration Changes

**Before (OpenAI)**:
```typescript
const client = new OpenAI({
  apiKey: process.env.OPENAI_API_KEY
});
```

**After (BodhiApp)**:
```typescript
const client = new OpenAI({
  apiKey: process.env.BODHI_API_TOKEN,
  baseURL: 'http://localhost:1135/v1'
});
```

### Model Name Mapping

Replace OpenAI model names with your BodhiApp model aliases:

```typescript
// OpenAI models → BodhiApp aliases
const modelMap = {
  'gpt-3.5-turbo': 'llama3:instruct',
  'gpt-4': 'llama3:70b',
  'gpt-4-turbo': 'mixtral:8x7b'
};

// Use in requests
const model = modelMap['gpt-3.5-turbo'] || 'llama3:instruct';
```

### Feature Compatibility

| OpenAI Feature | BodhiApp Support | Notes |
|----------------|------------------|-------|
| Chat Completions | ✅ Full | Complete compatibility |
| Streaming | ✅ Full | Server-sent events format |
| Function Calling | ❌ Not supported | Can simulate with prompting |
| Vision | ❌ Not supported | Text-only models |
| Embeddings | ❌ Not supported | Chat completions only |
| Fine-tuning | ❌ Not supported | Use model aliases for customization |

## Best Practices

### Performance Optimization

1. **Use Streaming**: Enable streaming for better user experience
2. **Manage Context**: Keep conversation history reasonable length
3. **Model Selection**: Choose appropriate model size for your use case
4. **Connection Pooling**: Reuse HTTP connections for multiple requests

### Error Handling

1. **Retry Logic**: Implement exponential backoff for transient errors
2. **Graceful Degradation**: Handle model unavailability gracefully
3. **Validation**: Validate inputs before sending requests
4. **Monitoring**: Log API usage and error rates

### Security

1. **Token Management**: Secure API token storage and rotation
2. **Input Validation**: Sanitize user inputs
3. **Rate Limiting**: Implement client-side rate limiting
4. **Local Processing**: Leverage local inference for privacy

## Next Steps

Now that you understand the OpenAI-compatible API:

1. **[Learn Model Management](model-management.md)** - Create and manage model aliases
2. **[Explore BodhiApp APIs](bodhi-api.md)** - Access advanced BodhiApp features
3. **[Try Ollama APIs](ollama-api.md)** - Use Ollama-compatible endpoints
4. **[Handle Errors](error-handling.md)** - Implement robust error handling

---

*The OpenAI-compatible API provides seamless integration with existing tools and workflows while keeping all processing local.* 