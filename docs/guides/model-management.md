# Model Management

BodhiApp provides a comprehensive model management system that allows you to download, organize, and configure Large Language Models for local inference. The system is built around three core concepts: **Model Files**, **Model Aliases**, and **Model Downloads**.

## Overview

The model management system enables:

- **Model Downloads**: Fetch GGUF models from HuggingFace repositories
- **Model Aliases**: Create semantic references with custom parameters
- **Model Files**: Manage local GGUF files and their metadata
- **Parameter Configuration**: Customize inference and context parameters per model

## Authentication Requirements

Model management operations require different permission levels:

- **Read Operations** (list models/files): `user` role OR `scope_token_user`
- **Write Operations** (create/update aliases, downloads): `power_user` role OR `scope_token_power_user`

```typescript
const headers = {
  'Authorization': `Bearer ${apiToken}`,
  'Content-Type': 'application/json'
};
```

## Model Aliases System

### What are Model Aliases?

Model aliases are semantic identifiers that reference specific model configurations with predefined parameters. Instead of specifying a full model path and parameters each time, you use a simple alias like `llama3:instruct` or `creative-writer`.

#### Benefits of Aliases

- **Consistency**: Same parameters applied across all API calls
- **Simplicity**: Use `creative-writer` instead of complex model paths
- **Flexibility**: Switch model files without changing API calls
- **Organization**: Group models by use case or capability

### Alias Structure

Each alias contains:

```typescript
interface Alias {
  alias: string;              // Unique identifier (e.g., "llama3:instruct")
  repo: string;               // HuggingFace repository
  filename: string;           // GGUF model file name
  snapshot: string;           // Git commit/branch reference
  source: "user" | "model";   // Creation source
  request_params: {           // OpenAI API parameters
    temperature?: number;
    max_tokens?: number;
    top_p?: number;
    frequency_penalty?: number;
    presence_penalty?: number;
    stop?: string[];
  };
  context_params: {           // llama.cpp context parameters
    n_ctx?: number;           // Context window size
    n_threads?: number;       // CPU threads
    n_parallel?: number;      // Parallel processing
    n_predict?: number;       // Max tokens to predict
    n_keep?: number;          // Tokens to keep in context
  };
}
```

### Managing Aliases via API

#### List Model Aliases

```typescript
// GET /bodhi/v1/models
const response = await fetch('http://localhost:1135/bodhi/v1/models', {
  headers: { 'Authorization': `Bearer ${apiToken}` }
});

const aliases = await response.json();
```

**Response Format**:
```json
{
  "data": [
    {
      "alias": "llama3:instruct",
      "repo": "microsoft/Phi-3-mini-4k-instruct-gguf",
      "filename": "Phi-3-mini-4k-instruct-q4.gguf",
      "snapshot": "main",
      "source": "user",
      "request_params": {
        "temperature": 0.7,
        "top_p": 0.95,
        "max_tokens": 2048
      },
      "context_params": {
        "n_ctx": 4096,
        "n_threads": 8
      }
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 30
}
```

#### Get Specific Alias

```typescript
// GET /bodhi/v1/models/{alias}
const response = await fetch('http://localhost:1135/bodhi/v1/models/llama3:instruct', {
  headers: { 'Authorization': `Bearer ${powerUserToken}` }
});

const alias = await response.json();
```

#### Create Model Alias

```typescript
// POST /bodhi/v1/models
const aliasData = {
  alias: 'creative-writer',
  repo: 'microsoft/Phi-3-mini-4k-instruct-gguf',
  filename: 'Phi-3-mini-4k-instruct-q4.gguf',
  snapshot: 'main',
  request_params: {
    temperature: 1.2,
    top_p: 0.9,
    max_tokens: 1000
  },
  context_params: {
    n_ctx: 4096,
    n_threads: 8,
    n_predict: 1000
  }
};

const response = await fetch('http://localhost:1135/bodhi/v1/models', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${powerUserToken}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify(aliasData)
});

const createdAlias = await response.json();
```

#### Update Model Alias

```typescript
// PUT /bodhi/v1/models/{alias}
const updates = {
  request_params: {
    temperature: 0.8,
    max_tokens: 1500
  },
  context_params: {
    n_ctx: 8192
  }
};

const response = await fetch('http://localhost:1135/bodhi/v1/models/creative-writer', {
  method: 'PUT',
  headers: {
    'Authorization': `Bearer ${powerUserToken}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify(updates)
});
```

### Alias Configuration Examples

#### Chat Assistant Alias
```typescript
{
  alias: 'chat-assistant',
  repo: 'microsoft/Phi-3-mini-4k-instruct-gguf',
  filename: 'Phi-3-mini-4k-instruct-q4.gguf',
  request_params: {
    temperature: 0.7,
    top_p: 0.95,
    max_tokens: 2048
  },
  context_params: {
    n_ctx: 4096,
    n_threads: 8
  }
}
```

#### Creative Writer Alias
```typescript
{
  alias: 'creative-writer',
  repo: 'microsoft/Phi-3-mini-4k-instruct-gguf',
  filename: 'Phi-3-mini-4k-instruct-q4.gguf',
  request_params: {
    temperature: 1.2,
    top_p: 0.9,
    max_tokens: 1000,
    frequency_penalty: 0.1,
    presence_penalty: 0.1
  },
  context_params: {
    n_ctx: 8192,
    n_predict: 1000
  }
}
```

#### Code Assistant Alias
```typescript
{
  alias: 'code-assistant',
  repo: 'microsoft/Phi-3-mini-4k-instruct-gguf',
  filename: 'Phi-3-mini-4k-instruct-q4.gguf',
  request_params: {
    temperature: 0.1,
    top_p: 0.95,
    max_tokens: 4096,
    stop: ['```\n\n', '# End']
  },
  context_params: {
    n_ctx: 16384,
    n_predict: 4096
  }
}
```

## Model Downloads System

### Download Models from HuggingFace

BodhiApp can download GGUF models directly from HuggingFace repositories.

#### Download by Repository and Filename

```typescript
// POST /bodhi/v1/modelfiles/pull
const downloadRequest = {
  repo: 'microsoft/Phi-3-mini-4k-instruct-gguf',
  filename: 'Phi-3-mini-4k-instruct-q4.gguf'
};

const response = await fetch('http://localhost:1135/bodhi/v1/modelfiles/pull', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${powerUserToken}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify(downloadRequest)
});

const downloadInfo = await response.json();
```

**Response Format**:
```json
{
  "id": "download-123",
  "repo": "microsoft/Phi-3-mini-4k-instruct-gguf",
  "filename": "Phi-3-mini-4k-instruct-q4.gguf",
  "status": "pending",
  "created_at": "2024-01-15T10:30:00Z"
}
```

#### Download by Predefined Alias

```typescript
// POST /bodhi/v1/modelfiles/pull/{alias}
const response = await fetch('http://localhost:1135/bodhi/v1/modelfiles/pull/llama3:instruct', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${powerUserToken}`
  }
});

const downloadInfo = await response.json();
```

#### Available Predefined Aliases

| Alias | Model | Description |
|-------|-------|-------------|
| `llama3:instruct` | Meta Llama 3 8B Instruct | General purpose instruction model |
| `llama3:70b-instruct` | Meta Llama 3 70B Instruct | Large instruction model |
| `llama2:chat` | Llama 2 7B Chat | Conversational model |
| `phi3:mini` | Phi 3 Mini | Lightweight Microsoft model |
| `mistral:instruct` | Mistral 7B Instruct | Fast instruction-following model |
| `mixtral:instruct` | Mixtral 8x7B Instruct | Mixture of experts model |
| `gemma:instruct` | Gemma 7B Instruct | Google's instruction model |

### Download Status Tracking

#### List Downloads

```typescript
// GET /bodhi/v1/modelfiles/pull
const response = await fetch('http://localhost:1135/bodhi/v1/modelfiles/pull', {
  headers: { 'Authorization': `Bearer ${powerUserToken}` }
});

const downloads = await response.json();
```

#### Check Download Status

```typescript
// GET /bodhi/v1/modelfiles/pull/{id}
const response = await fetch('http://localhost:1135/bodhi/v1/modelfiles/pull/download-123', {
  headers: { 'Authorization': `Bearer ${powerUserToken}` }
});

const status = await response.json();
```

**Status Response**:
```json
{
  "id": "download-123",
  "repo": "microsoft/Phi-3-mini-4k-instruct-gguf",
  "filename": "Phi-3-mini-4k-instruct-q4.gguf",
  "status": "completed",
  "progress": 100,
  "size_downloaded": 2147483648,
  "size_total": 2147483648,
  "created_at": "2024-01-15T10:30:00Z",
  "completed_at": "2024-01-15T10:35:00Z"
}
```

### Download Status Values

| Status | Description |
|--------|-------------|
| `pending` | Download queued but not started |
| `downloading` | Download in progress |
| `completed` | Download finished successfully |
| `failed` | Download failed with error |
| `cancelled` | Download was cancelled |

## Model Files Management

### List Local Model Files

View all GGUF model files available in your HuggingFace cache:

```typescript
// GET /bodhi/v1/modelfiles
const response = await fetch('http://localhost:1135/bodhi/v1/modelfiles', {
  headers: { 'Authorization': `Bearer ${apiToken}` }
});

const modelFiles = await response.json();
```

**Response Format**:
```json
{
  "data": [
    {
      "repo": "microsoft/Phi-3-mini-4k-instruct-gguf",
      "filename": "Phi-3-mini-4k-instruct-q4.gguf",
      "snapshot_id": "main",
      "size": 2147483648
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 30
}
```

### Model File Information

Each model file entry includes:

- **Repository**: Source HuggingFace repository
- **Filename**: GGUF file name
- **Snapshot ID**: Git commit or branch reference
- **Size**: File size in bytes
- **Path**: Local file system path (computed)

### Storage Location

Model files are stored in the HuggingFace cache directory:

- **Default Location**: `~/.cache/huggingface/hub/`
- **Configurable**: Via `HF_HOME` setting
- **Structure**: `models--{org}--{repo}/snapshots/{commit}/{filename}`

## Parameter Configuration Guide

### Request Parameters (OpenAI Compatible)

These parameters control the model's response generation:

| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| `temperature` | number | 0.0-2.0 | 1.0 | Sampling randomness (0=deterministic, 2=very random) |
| `max_tokens` | integer | 1-∞ | Model-dependent | Maximum tokens to generate |
| `top_p` | number | 0.0-1.0 | 1.0 | Nucleus sampling (cumulative probability) |
| `frequency_penalty` | number | -2.0-2.0 | 0.0 | Penalty for frequent tokens |
| `presence_penalty` | number | -2.0-2.0 | 0.0 | Penalty for repeated tokens |
| `stop` | string[] | - | [] | Stop sequences to end generation |

### Context Parameters (llama.cpp Specific)

These parameters control the model's context and processing:

| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| `n_ctx` | integer | 512-32768 | 2048 | Context window size |
| `n_threads` | integer | 1-64 | 8 | CPU threads for processing |
| `n_parallel` | integer | 1-16 | 1 | Parallel sequence processing |
| `n_predict` | integer | -1-∞ | -1 | Max tokens to predict (-1=unlimited) |
| `n_keep` | integer | 0-∞ | 0 | Tokens to keep when context full |

### Parameter Optimization Tips

#### For Chat Applications
```typescript
{
  temperature: 0.7,        // Balanced creativity
  top_p: 0.95,            // Focused sampling
  max_tokens: 2048,       // Reasonable response length
  n_ctx: 4096,            // Good context window
  n_threads: 8            // Utilize CPU cores
}
```

#### For Creative Writing
```typescript
{
  temperature: 1.2,       // High creativity
  top_p: 0.9,            // Allow diverse tokens
  frequency_penalty: 0.1, // Reduce repetition
  presence_penalty: 0.1,  // Encourage variety
  n_ctx: 8192,           // Large context for stories
  n_predict: 1000        // Longer generations
}
```

#### For Code Generation
```typescript
{
  temperature: 0.1,       // Low randomness
  top_p: 0.95,           // Focused sampling
  stop: ['```\n\n'],     // Stop at code block end
  n_ctx: 16384,          // Large context for code
  n_predict: 4096        // Allow long code blocks
}
```

#### For Analysis Tasks
```typescript
{
  temperature: 0.3,       // Consistent analysis
  max_tokens: 4096,      // Detailed responses
  n_ctx: 16384,          // Large context for documents
  n_threads: 16          // Fast processing
}
```

## Advanced Usage Patterns

### Model Switching Workflow

```typescript
class ModelManager {
  private apiToken: string;
  private baseURL: string = 'http://localhost:1135';

  async createAlias(config: AliasConfig) {
    const response = await fetch(`${this.baseURL}/bodhi/v1/models`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.apiToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(config)
    });
    return response.json();
  }

  async switchModel(aliasName: string, newModelFile: string) {
    const response = await fetch(`${this.baseURL}/bodhi/v1/models/${aliasName}`, {
      method: 'PUT',
      headers: {
        'Authorization': `Bearer ${this.apiToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ filename: newModelFile })
    });
    return response.json();
  }

  async downloadModel(repo: string, filename: string) {
    const response = await fetch(`${this.baseURL}/bodhi/v1/modelfiles/pull`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.apiToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ repo, filename })
    });
    return response.json();
  }
}
```

### Bulk Alias Creation

```typescript
async function createMultipleAliases(configs: AliasConfig[]) {
  const results = [];
  
  for (const config of configs) {
    try {
      const response = await fetch('http://localhost:1135/bodhi/v1/models', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${powerUserToken}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(config)
      });
      
      const result = await response.json();
      results.push({ success: true, alias: config.alias, data: result });
    } catch (error) {
      results.push({ success: false, alias: config.alias, error });
    }
  }
  
  return results;
}

// Usage
const aliasConfigs = [
  {
    alias: 'fast-chat',
    repo: 'microsoft/Phi-3-mini-4k-instruct-gguf',
    filename: 'Phi-3-mini-4k-instruct-q4.gguf',
    request_params: { temperature: 0.7, max_tokens: 1024 }
  },
  {
    alias: 'detailed-analysis',
    repo: 'microsoft/Phi-3-mini-4k-instruct-gguf',
    filename: 'Phi-3-mini-4k-instruct-q4.gguf',
    request_params: { temperature: 0.3, max_tokens: 4096 }
  }
];

const results = await createMultipleAliases(aliasConfigs);
```

### Download Progress Monitoring

```typescript
async function monitorDownload(downloadId: string) {
  const checkStatus = async () => {
    const response = await fetch(`http://localhost:1135/bodhi/v1/modelfiles/pull/${downloadId}`, {
      headers: { 'Authorization': `Bearer ${powerUserToken}` }
    });
    return response.json();
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
    }, 2000); // Check every 2 seconds
  });
}

// Usage
const download = await fetch('http://localhost:1135/bodhi/v1/modelfiles/pull', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${powerUserToken}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    repo: 'microsoft/Phi-3-mini-4k-instruct-gguf',
    filename: 'Phi-3-mini-4k-instruct-q4.gguf'
  })
});

const downloadInfo = await download.json();
await monitorDownload(downloadInfo.id);
console.log('Download completed!');
```

## Best Practices

### Alias Naming Conventions

1. **Use Semantic Names**: `creative-writer`, `code-assistant`, `chat-bot`
2. **Include Model Context**: `llama3:instruct`, `phi3:mini-chat`
3. **Avoid Special Characters**: Use hyphens and colons only
4. **Be Descriptive**: `fast-summary` vs `model1`

### Parameter Selection

1. **Start with Defaults**: Use default parameters initially
2. **Test Incrementally**: Change one parameter at a time
3. **Document Changes**: Keep notes on parameter effects
4. **Use Case Specific**: Different parameters for different tasks

### Storage Management

1. **Monitor Disk Usage**: GGUF files can be large (2-8GB each)
2. **Clean Unused Models**: Remove models you no longer need
3. **Use Appropriate Quantization**: Balance quality vs size (Q4_K_M is often good)
4. **Batch Downloads**: Download multiple models during off-hours

### Performance Optimization

1. **Match Context to Use Case**: Don't use 16K context for simple chat
2. **Optimize Thread Count**: Match to your CPU cores
3. **Use Appropriate Model Size**: Larger isn't always better
4. **Monitor Resource Usage**: Watch CPU, memory, and disk usage

## Error Handling

### Common Error Scenarios

```typescript
async function createAliasWithErrorHandling(config: AliasConfig) {
  try {
    const response = await fetch('http://localhost:1135/bodhi/v1/models', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${powerUserToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(config)
    });

    if (!response.ok) {
      const error = await response.json();
      
      switch (response.status) {
        case 400:
          throw new Error(`Invalid alias configuration: ${error.error.message}`);
        case 401:
          throw new Error('Authentication failed - check your API token');
        case 403:
          throw new Error('Insufficient permissions - requires PowerUser access');
        case 409:
          throw new Error(`Alias '${config.alias}' already exists`);
        default:
          throw new Error(`Server error: ${error.error.message}`);
      }
    }

    return response.json();
  } catch (error) {
    console.error('Failed to create alias:', error);
    throw error;
  }
}
```

### Download Error Recovery

```typescript
async function downloadWithRetry(repo: string, filename: string, maxRetries = 3) {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      const response = await fetch('http://localhost:1135/bodhi/v1/modelfiles/pull', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${powerUserToken}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ repo, filename })
      });

      if (response.ok) {
        return response.json();
      }
      
      if (attempt === maxRetries) {
        throw new Error(`Download failed after ${maxRetries} attempts`);
      }
      
      // Wait before retry (exponential backoff)
      await new Promise(resolve => setTimeout(resolve, Math.pow(2, attempt) * 1000));
      
    } catch (error) {
      if (attempt === maxRetries) throw error;
    }
  }
}
```

## Next Steps

Now that you understand model management:

1. **[Learn BodhiApp APIs](bodhi-api.md)** - Access system information and settings
2. **[Try Ollama APIs](ollama-api.md)** - Use Ollama-compatible endpoints
3. **[Handle Errors](error-handling.md)** - Implement robust error handling
4. **[See Examples](examples.md)** - Complete integration examples

---

*Model management is the foundation of BodhiApp's flexibility, allowing you to optimize AI performance for specific use cases while maintaining consistent API interfaces.* 