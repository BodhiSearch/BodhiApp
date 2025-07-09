# BodhiApp AI Integration Guide

## Overview

BodhiApp provides a comprehensive API suite for building AI applications powered by local Large Language Models (LLMs). This guide focuses on the APIs available to Users and Power Users for creating chat applications, model management, and AI-powered features.

## Quick Start

### Base URLs
- **App-specific APIs**: `https://your-domain/bodhi/v1/`
- **OpenAI-compatible APIs**: `https://your-domain/v1/`

### Authentication
BodhiApp uses OAuth2/OpenID Connect for authentication. Two authentication methods are supported:

1. **Session-based**: Using browser cookies (for web applications)
2. **API Token**: Using Bearer tokens (for programmatic access)

```typescript
// API Token authentication
const headers = {
  'Authorization': 'Bearer YOUR_API_TOKEN',
  'Content-Type': 'application/json'
}
```

### User Roles & Permissions
- **User**: Basic access to chat APIs and model listing
- **Power User**: Additional access to model creation, updates, and downloads

## Core Chat APIs

### 1. List Available Models

**Endpoint**: `GET /v1/models`  
**Access**: User, Power User  
**Purpose**: Get all available models for chat completions

```typescript
// Request
const response = await fetch('/v1/models', {
  headers: {
    'Authorization': 'Bearer YOUR_API_TOKEN'
  }
});

// Response
{
  "object": "list",
  "data": [
    {
      "id": "llama-3.1-8b-instruct",
      "object": "model",
      "created": 1728849600,
      "owned_by": "meta"
    }
  ]
}
```

### 2. Chat Completions (Core AI Feature)

**Endpoint**: `POST /v1/chat/completions`  
**Access**: User, Power User  
**Purpose**: Generate AI responses using local LLMs

```typescript
// Non-streaming request
const response = await fetch('/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_API_TOKEN',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    model: "llama-3.1-8b-instruct",
    messages: [
      {
        role: "user",
        content: "Hello, how are you?"
      }
    ],
    temperature: 0.7,
    max_tokens: 150
  })
});

// Response
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1728849600,
  "model": "llama-3.1-8b-instruct",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! I'm doing well, thank you for asking. How can I help you today?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 20,
    "total_tokens": 30
  }
}
```

### 3. Streaming Chat Completions

For real-time chat applications, use streaming responses:

```typescript
// Streaming request
const response = await fetch('/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_API_TOKEN',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    model: "llama-3.1-8b-instruct",
    messages: [
      {
        role: "user",
        content: "Tell me a story"
      }
    ],
    stream: true
  })
});

// Handle streaming response
const reader = response.body.getReader();
while (true) {
  const { done, value } = await reader.read();
  if (done) break;
  
  const chunk = new TextDecoder().decode(value);
  const lines = chunk.split('\n');
  
  for (const line of lines) {
    if (line.startsWith('data: ')) {
      const data = line.slice(6);
      if (data === '[DONE]') return;
      
      try {
        const parsed = JSON.parse(data);
        const content = parsed.choices[0]?.delta?.content;
        if (content) {
          // Process each token as it arrives
          console.log(content);
        }
      } catch (e) {
        // Handle parsing errors
      }
    }
  }
}
```

## Model Management APIs

### 1. List Bodhi Models

**Endpoint**: `GET /bodhi/v1/models`  
**Access**: User, Power User  
**Purpose**: Get detailed model information including download status

```typescript
const response = await fetch('/bodhi/v1/models', {
  headers: {
    'Authorization': 'Bearer YOUR_API_TOKEN'
  }
});

// Response includes download status and model details
{
  "models": [
    {
      "id": "llama-3.1-8b-instruct",
      "name": "Llama 3.1 8B Instruct",
      "size": "4.7GB",
      "status": "ready",
      "downloaded": true,
      "family": "llama",
      "capabilities": ["chat", "completion"]
    }
  ]
}
```

### 2. Create/Update Model (Power User)

**Endpoint**: `POST /bodhi/v1/models`  
**Access**: Power User  
**Purpose**: Add new models or update existing ones

```typescript
const response = await fetch('/bodhi/v1/models', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_API_TOKEN',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    name: "custom-model",
    family: "llama",
    huggingface_repo: "meta-llama/Llama-3.1-8B-Instruct",
    filename: "model.gguf",
    chat_template: "llama3",
    capabilities: ["chat"]
  })
});
```

### 3. Download Model (Power User)

**Endpoint**: `POST /bodhi/v1/models/{model_id}/download`  
**Access**: Power User  
**Purpose**: Download model files to local storage

```typescript
const response = await fetch('/bodhi/v1/models/llama-3.1-8b-instruct/download', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_API_TOKEN'
  }
});
```

## Ollama Compatibility APIs

BodhiApp provides Ollama-compatible endpoints for existing Ollama integrations:

### Generate Completion
**Endpoint**: `POST /ollama/api/generate`  
**Access**: User, Power User

```typescript
const response = await fetch('/ollama/api/generate', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_API_TOKEN',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    model: "llama-3.1-8b-instruct",
    prompt: "Tell me about AI",
    stream: false
  })
});
```

### Chat with Ollama Format
**Endpoint**: `POST /ollama/api/chat`  
**Access**: User, Power User

```typescript
const response = await fetch('/ollama/api/chat', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_API_TOKEN',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    model: "llama-3.1-8b-instruct",
    messages: [
      {
        role: "user",
        content: "Hello"
      }
    ]
  })
});
```

## Error Handling

### Standard Error Response Format
```typescript
{
  "error": {
    "code": "invalid_request",
    "message": "The request is invalid",
    "details": {
      "field": "model",
      "issue": "Model not found"
    }
  }
}
```

### Common Error Codes
- **401**: Authentication required or invalid token
- **403**: Insufficient permissions for the requested operation
- **404**: Model or resource not found
- **429**: Rate limit exceeded
- **500**: Internal server error
- **503**: Service unavailable (model loading)

### Error Handling Best Practices
```typescript
async function makeApiCall(endpoint, options) {
  try {
    const response = await fetch(endpoint, options);
    
    if (!response.ok) {
      const error = await response.json();
      throw new Error(`API Error ${response.status}: ${error.error.message}`);
    }
    
    return await response.json();
  } catch (error) {
    // Handle network errors, parsing errors, etc.
    console.error('API call failed:', error);
    throw error;
  }
}
```

## Building Chat Applications

### Basic Chat Implementation
```typescript
class BodhiChatClient {
  constructor(apiToken, baseUrl = '') {
    this.apiToken = apiToken;
    this.baseUrl = baseUrl;
  }

  async getModels() {
    return await this.makeRequest('/v1/models');
  }

  async sendMessage(model, messages, options = {}) {
    return await this.makeRequest('/v1/chat/completions', {
      method: 'POST',
      body: JSON.stringify({
        model,
        messages,
        ...options
      })
    });
  }

  async streamMessage(model, messages, onToken, options = {}) {
    const response = await fetch(`${this.baseUrl}/v1/chat/completions`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.apiToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model,
        messages,
        stream: true,
        ...options
      })
    });

    const reader = response.body.getReader();
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      
      const chunk = new TextDecoder().decode(value);
      const lines = chunk.split('\n');
      
      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const data = line.slice(6);
          if (data === '[DONE]') return;
          
          try {
            const parsed = JSON.parse(data);
            const content = parsed.choices[0]?.delta?.content;
            if (content) {
              onToken(content);
            }
          } catch (e) {
            // Handle parsing errors
          }
        }
      }
    }
  }

  async makeRequest(endpoint, options = {}) {
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers: {
        'Authorization': `Bearer ${this.apiToken}`,
        'Content-Type': 'application/json',
        ...options.headers
      }
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(`API Error ${response.status}: ${error.error.message}`);
    }

    return await response.json();
  }
}
```

### Usage Example
```typescript
const client = new BodhiChatClient('your-api-token');

// Get available models
const models = await client.getModels();
console.log('Available models:', models.data);

// Send a message
const response = await client.sendMessage('llama-3.1-8b-instruct', [
  { role: 'user', content: 'Hello, how are you?' }
]);
console.log('Response:', response.choices[0].message.content);

// Stream a response
await client.streamMessage('llama-3.1-8b-instruct', [
  { role: 'user', content: 'Tell me a story' }
], (token) => {
  process.stdout.write(token);
});
```

## Best Practices

### 1. Authentication Management
- Store API tokens securely
- Implement token refresh mechanisms
- Use environment variables for sensitive data

### 2. Rate Limiting
- Implement exponential backoff for retries
- Monitor rate limit headers
- Queue requests during high load

### 3. Model Selection
- Always verify model availability before use
- Cache model lists to reduce API calls
- Handle model loading states gracefully

### 4. Streaming Optimization
- Use streaming for real-time applications
- Implement proper error handling for stream interruptions
- Buffer tokens for smooth UI updates

### 5. Error Recovery
- Implement retry logic for transient failures
- Provide meaningful error messages to users
- Log errors for debugging purposes

## Advanced Features

### Custom Model Configuration
Power Users can configure custom models with specific parameters:

```typescript
const modelConfig = {
  name: "custom-instruct-model",
  family: "llama",
  huggingface_repo: "your-org/custom-model",
  filename: "model.gguf",
  chat_template: "custom",
  system_prompt: "You are a helpful assistant specialized in...",
  temperature: 0.7,
  max_tokens: 2048
};

await client.makeRequest('/bodhi/v1/models', {
  method: 'POST',
  body: JSON.stringify(modelConfig)
});
```

### Multi-turn Conversations
Maintain conversation context across multiple requests:

```typescript
class ConversationManager {
  constructor(client, model) {
    this.client = client;
    this.model = model;
    this.messages = [];
  }

  async sendMessage(userMessage) {
    this.messages.push({ role: 'user', content: userMessage });
    
    const response = await this.client.sendMessage(this.model, this.messages);
    const assistantMessage = response.choices[0].message;
    
    this.messages.push(assistantMessage);
    return assistantMessage.content;
  }

  clearHistory() {
    this.messages = [];
  }
}
```

## Conclusion

This guide provides the essential APIs and patterns for building AI applications with BodhiApp. The system's OpenAI-compatible endpoints ensure easy integration with existing AI tools, while the additional Bodhi-specific APIs provide enhanced model management capabilities for Power Users.

For the most up-to-date API documentation and examples, refer to the OpenAPI specification available at `/bodhi/v1/openapi.json`. 