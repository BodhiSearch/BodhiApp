# Understanding Context Parameters in Bodhi

## Overview

Context parameters control how the model processes and generates text at a low level. These parameters affect performance, resource usage, and generation behavior.

## Core Parameters

### n_ctx (Context Window Size)

- **Default:** 512
- **Range:** Model dependent
- **Description:** Controls the size of the prompt context window.
- **Impact:**
  - Affects memory usage and response context length
  - Larger values allow longer conversations
  - Must be balanced with available system memory

### n_seed (Context Seed)

- **Default:** 0
- **Range:** 0-999999
- **Description:** A number that helps generate consistent responses.
- **Impact:**
  - Same seed + settings = similar responses
  - Useful for:
    - Testing and debugging
    - Reproducible results
    - Consistent behavior across runs

### n_threads (Thread Count)

- **Default:** System CPU count
- **Range:** 1-max system threads
- **Description:** Number of CPU threads to use for computation.
- **Impact:**
  - Affects processing speed
  - Higher values may improve performance
  - Should match system capabilities

### n_parallel (Parallel Sequences)

- **Default:** 1
- **Description:** Number of concurrent requests to process.
- **Impact:**
  - Controls server load
  - Higher values allow more simultaneous processing
  - Must be balanced with system resources

### n_predict (Token Prediction Limit)

- **Default:** -1 (unbounded)
- **Description:** Maximum number of tokens to generate.
- **Impact:**
  - Controls response length
  - -1 allows model to determine length
  - Can be used to limit resource usage

### n_keep (Initial Prompt Tokens)

- **Default:** 0
- **Description:** Number of tokens to keep from initial prompt.
- **Impact:**
  - Affects context preservation
  - Higher values maintain more initial context
  - Useful for maintaining specific instructions

## Best Practices

### Resource Management

1. Start with default values
2. Adjust based on:
   - Available system resources
   - Response quality requirements
   - Performance needs

### Performance Optimization

#### 1. n_threads:
- Match to available CPU cores
- Consider other system loads
- Monitor CPU usage

#### 2. n_ctx:
- Balance memory usage with context needs
- Consider model's default context size
- Monitor memory consumption

#### 3. n_parallel:
- Start with 1
- Increase based on server capacity
- Monitor system stability

### Consistency Control

#### 1. n_seed:
- Use fixed seeds for testing
- Document seeds for reproducible results
- Vary seeds for production diversity

#### 2. n_keep:
- Use for maintaining critical context
- Balance with n_ctx limits
- Consider prompt engineering needs

## Common Configurations

### Development Setup

```json
{
    "n_ctx": 512,
    "n_threads": 4,
    "n_parallel": 1,
    "n_seed": 42,
    "n_predict": 1024,
    "n_keep": 0
}
```

### Production Setup

```json
{
    "n_ctx": 2048,
    "n_threads": 8,
    "n_parallel": 4,
    "n_seed": 0,
    "n_predict": -1,
    "n_keep": 64
}
```

### Testing Setup

```json
{
    "n_ctx": 1024,
    "n_threads": 2,
    "n_parallel": 1,
    "n_seed": 12345,
    "n_predict": 512,
    "n_keep": 128
}
```

## Troubleshooting

### Common Issues

#### 1. High Memory Usage:
- Reduce n_ctx
- Lower n_parallel
- Monitor system resources

#### 2. Slow Response Times:
- Increase n_threads
- Check system load
- Optimize n_ctx

#### 3. Inconsistent Results:
- Set fixed n_seed
- Check n_keep values
- Review context window

#### 4. Resource Conflicts:
- Balance n_parallel with resources
- Adjust n_threads for system
- Monitor concurrent usage

## Technical Details

These parameters directly affect the underlying language model's behavior and resource utilization. They should be adjusted based on:

- Hardware capabilities
- Application requirements
- Performance needs
- Stability requirements

## See Also

- Model Configuration Guide
- Performance Optimization Guide
- System Requirements Documentation
