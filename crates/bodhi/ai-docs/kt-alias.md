# Understanding Aliases in Bodhi

## Overview
An alias in Bodhi is a powerful way to reference either a pre-configured model or a custom set of configurations. It provides a simplified interface for users to work with complex model configurations.

## Types of Aliases

### 1. Built-in Model Aliases
- Direct references to GGUF model files from the Hugging Face repository
- Uses embedded chat templates
- Comes with default settings (e.g., context window of 2048 tokens)
- Automatically configured with fallback values

### 2. Custom Aliases
- User-defined configurations created through the Models page
- Allows full customization of model parameters
- Can override default settings
- Provides a way to save and reuse specific configurations

## Configuration Parameters

### Core Parameters

#### 1. System Prompt
- Initial instructions that set the AI's role and behavior
- Defines the context and personality for the entire conversation
- Applied at the start of each chat session

#### 2. Stream Response
- Controls real-time response generation
- When enabled: Shows AI responses as they're being generated
- When disabled: Shows complete response only after full generation

#### 3. API Token
- Personal authentication token for API access
- Enables use of custom API credentials
- Alternative to default configuration

### Generation Parameters

#### 1. Temperature (0-2)
- Controls response randomness and creativity
- Lower values (0.2): More focused, deterministic responses
- Higher values (0.8): More creative, varied outputs
- Default: 1.0
- Best used alone without Top P

#### 2. Top P (0-1)
- Alternative to temperature for controlling randomness
- Uses nucleus sampling
- Lower values: Consider only most likely tokens
- Higher values: Consider more possibilities
- Default: 1.0
- Avoid using with Temperature

#### 3. Max Tokens
- Maximum length of AI's response
- Higher values allow longer responses
- Must fit within model's context length
- Default: 2048

#### 4. Presence Penalty (-2 to +2)
- Influences topic diversity
- Positive values: Encourages new topics
- Negative values: Allows topic repetition
- Default: 0 (disabled)
- Helps prevent repetitive responses

#### 5. Frequency Penalty (-2 to +2)
- Controls word and phrase repetition
- Positive values: Reduces repetition
- Negative values: Allows more repetition
- Default: 0 (disabled)
- Useful for maintaining natural language flow

#### 6. Stop Words
- Up to 4 sequences that stop response generation
- Useful for controlling response format
- Response won't contain the stop sequence
- Helps maintain specific output structures

#### 7. Seed
- Controls response determinism
- Same seed + settings = similar responses
- Useful for:
  - Testing
  - Reproducible results
  - Consistent behavior

### Server Parameters

#### 1. Context Window Size (n_ctx)
- Size of the prompt context
- Default: 512
- Affects memory usage and response context

#### 2. Number of Threads (n_threads)
- Computation threads used
- Default: System CPU count
- Affects processing speed

#### 3. Parallel Sequences (n_parallel)
- Number of concurrent requests
- Default: 1
- Impacts server load

#### 4. Token Prediction Limit (n_predict)
- New tokens to predict
- Default: -1 (unbounded)
- Controls response length

#### 5. Initial Prompt Tokens (n_keep)
- Tokens to keep from initial prompt
- Default: 0
- Affects context preservation

## Best Practices

### Parameter Combinations
1. **Creative Writing**
   - Higher temperature (0.8)
   - Lower presence penalty
   - Moderate max tokens

2. **Technical Responses**
   - Lower temperature (0.2)
   - Higher frequency penalty
   - Specific stop words

3. **Balanced Conversation**
   - Moderate temperature (0.5)
   - Moderate presence/frequency penalties
   - Default max tokens

### When to Use Built-in Aliases
- Quick start with standard configurations
- Testing different models with default settings
- Simple use cases without specific requirements

### When to Create Custom Aliases
- Need specific parameter configurations
- Optimizing for particular use cases
- Maintaining consistent settings across sessions
- Fine-tuning model behavior

## Technical Details

The alias system is built on several key components:
- `Alias` struct containing configuration details
- `OAIRequestParams` for API-related settings
- `GptContextParams` for server-side parameters
- Chat template specifications
- Model repository connections

This knowledge transfer document serves as a comprehensive guide to understanding and working with aliases in the Bodhi application.
