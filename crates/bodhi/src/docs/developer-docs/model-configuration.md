---
title: "Model Configuration"
description: "Advanced configuration options for model deployment and inference"
---

# Model Configuration

## Overview

Bodhi App provides extensive configuration options for model deployment and inference settings through model aliases and runtime parameters.

## Model Alias Configuration

### Structure
```yaml
alias: "model-name"
model_file: "path/to/model"
inference:
  temperature: 0.7
  top_p: 0.9
server:
  parallel_slots: 2
  context_length: 2048
```

### Parameters

#### Inference Settings
- `temperature`: Controls randomness (0.0 - 1.0)
- `top_p`: Nucleus sampling parameter
- `max_tokens`: Maximum response length
- `frequency_penalty`: Repetition control
- `presence_penalty`: Topic diversity control

#### Server Settings
- `parallel_slots`: Concurrent inference slots
- `context_length`: Maximum context window
- `batch_size`: Processing batch size
- `thread_count`: CPU thread allocation

## Runtime Configuration

### Request-Level Parameters
- Override default settings per request
- Dynamic parameter adjustment
- Session-specific configurations
- Response format control

### Resource Management
- Memory allocation
- CPU thread management
- Batch processing settings
- Cache configuration

## Integration Examples

### API Configuration
```typescript
const config = {
  inference: {
    temperature: 0.7,
    maxTokens: 1000
  },
  server: {
    parallelSlots: 2
  }
};

const response = await client.chat.completions.create({
  model: "alias-name",
  messages: [...],
  ...config
});
```

### Environment Variables
```bash
BODHI_MODEL_THREADS=4
BODHI_CONTEXT_LENGTH=2048
BODHI_DEFAULT_TEMP=0.7
``` 