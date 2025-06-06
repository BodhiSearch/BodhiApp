# Model Alias Page Design

## Context & Purpose

The Model Alias page serves as a configuration management interface for AI model inference in the Bodhi application. It allows users to create and manage custom configurations for model inference, enabling fine-tuned control over model behavior.

## User Context

### Primary Users

- AI Engineers (primary)
- End users (secondary)
- Technical users with ML/AI background

### Key Characteristics

- AI Engineers build applications using LLM responses
- Need predictable model behavior through configuration
- Require experimentation capabilities for parameter tuning

### Usage Patterns

- Frequent during initial model setup/tuning phase
- Infrequent once optimal configuration is found
- Reference existing configurations for new setups
- Copy/modify existing configurations for variants

### Key User Tasks

1. **Configuration Management**
   - Create new model aliases
   - Modify existing configurations
   - Reference successful configurations
   - Clone and adapt configurations

2. **Model Information**
   - View configuration details
   - Compare different aliases
   - Understand parameter impacts

## Information Architecture

### Data Hierarchy

**Primary Information (Table Columns)**

1. **Name/Alias (Highest Priority)**
   - Unique identifier for configuration
   - Indicates purpose/use case

2. **Source (High Priority)**
   - Distinguishes between user and model aliases
   - Indicates editability

3. **Repository (High Priority)**
   - Links to model source
   - Indicates model family

4. **Filename (High Priority)**
   - Shows specific model variant
   - Indicates quantization level

## Parameter Details

### Parameter Categories

#### 1. Generation Control

- **temperature**: Controls randomness (0.0-2.0)
- **top_p**: Nucleus sampling control (0.0-1.0)
- **frequency_penalty**: Penalizes token repetition (-2.0-2.0)
- **presence_penalty**: Encourages topic diversity (-2.0-2.0)
- **max_tokens**: Limits response length
- **stop**: Sequence triggers to stop generation
- **seed**: Controls generation determinism
- **user**: End-user identifier

#### 2. Performance & Resources

- **n_threads**: Computation thread count
- **n_ctx**: Context window size
- **n_parallel**: Concurrent request handling
- **n_predict**: Token prediction limit
- **n_keep**: Initial prompt token retention
- **n_seed**: Context initialization seed

### Parameter Constraints

- Parameters may have model-specific valid ranges
- Certain parameters can conflict (e.g., temperature vs top_p)
- Default values are provided when not specified
- Some parameters affect performance while others affect output quality

## Chat Template System

### Built-in Templates

```rust
pub enum ChatTemplateId {
  Llama3,
  Llama2,
  Llama2Legacy,
  Phi3,
  Gemma,
  Deepseek,
  CommandR,
  Openchat,
  Tinyllama,
}
```

### Template Sources

1. **Built-in Templates**
   - Pre-defined templates for common models
   - Optimized for specific model families

2. **Model-Embedded Templates**
   - Templates included in GGUF model files
   - Default if no other template specified

3. **Custom Repository Templates**
   - Downloaded from HuggingFace repositories
   - Uses tokenizer_config.json chat_template field
   - Jinja template format

## Configuration Storage

### File Organization

- **Location**: $BODHI_HOME/aliases/
- **Format**: YAML files
- **Naming**: Matches alias name (non-path chars replaced with '--')
- **Multiple configs per model supported**

### Example Configuration

```yaml
alias: llama3:instruct
repo: QuantFactory/Meta-Llama-3-8B-Instruct-GGUF
filename: Meta-Llama-3-8B-Instruct.Q8_0.gguf
chat_template: llama3
snapshot: 5007652f7a641fe7170e0bad4f63839419bd9213
context_params:
  n_keep: 24
request_params:
  stop:
    - <|start_header_id|>
    - <|end_header_id|>
    - <|eot_id|>
```

## Alias Resolution

### Source Types

```rust
pub enum AliasSource {
  User,    // User-created configurations
  Model,   // Configurations from model files
}
```

### Resolution Process

1. Request contains 'model' field matching alias
2. Search order:
   - Check downloaded models first
   - Fall back to user aliases
3. Model path construction:
   - Based on repo and filename
   - Example: ~/.cache/huggingface/hub/models--QuantFactory--Meta-Llama-3-8B-Instruct-GGUF/snapshots/main/

### Error Handling

- Missing model files raise errors
- Invalid configurations are validated before saving
- Model download not triggered automatically

## UI/UX Requirements

### Enhanced Features

1. **Parameter Organization**
   - Group by impact category
   - Show help text descriptions
   - Indicate parameter relationships
   - Display valid ranges

2. **Configuration Testing**
   - Test configuration before saving
   - Show sample outputs
   - Validate parameter combinations
   - Performance impact indicators

3. **Navigation Improvements**
   - Direct links between list and create views
   - Quick actions for common tasks
   - Clear visual hierarchy
   - Mobile-friendly layout

4. **Visual Feedback**
   - Parameter conflict warnings
   - Validation status indicators
   - Save/update confirmations
   - Error explanations

## Implementation Guidelines

### Form Design

- Group related parameters logically
- Show parameter descriptions as tooltips
- Indicate required vs optional fields
- Provide default value indicators

### Validation Rules

- Check parameter ranges
- Validate parameter combinations
- Ensure required fields
- Verify file existence

### Mobile Considerations

- Collapse parameter groups
- Touch-friendly controls
- Progressive disclosure
- Simplified validation feedback

## Interaction Design

### Current Actions

- View alias list
- Create new alias
- Edit existing alias (user aliases only)
- Sort by columns
- Expand rows for details

### Proposed Enhancements

1. **Configuration Management**
   - Clone existing alias
   - Test configuration before saving
   - Save configuration from chat playground
   - Quick parameter adjustment

2. **Visualization**
   - Clear distinction between user/model aliases
   - Parameter grouping and organization
   - Visual feedback for parameter impacts

3. **Integration with Chat Interface**
   - "Save as Alias" from chat settings
   - Visual indicator for modified settings
   - Quick switching between aliases
   - Parameter experimentation

## Layout Considerations

### Page Structure

- List view with key information
- Detailed form for creation/editing
- Integration with chat playground
- Parameter organization in collapsible sections

### Form Organization

- Required fields prominently displayed
- Parameter groups in collapsible sections
- Tooltips for parameter explanation
- Visual feedback for validation

## Technical Requirements

### Parameter Validation

```typescript
interface RequestParams {
  frequency_penalty?: number;
  max_tokens?: number;
  presence_penalty?: number;
  seed?: number;
  stop?: string[];
  temperature?: number;
  top_p?: number;
  user?: string;
}

interface ContextParams {
  n_seed?: number;
  n_threads?: number;
  n_ctx?: number;
  n_parallel?: number;
  n_predict?: number;
  n_keep?: number;
}
```

## Integration Points

### Chat Interface Integration

- Parameter experimentation in chat
- Configuration saving from chat
- Real-time parameter testing
- Quick configuration switching

### Model Management Integration

- Create alias from model file
- Link to model documentation
- Access to model metrics
- Configuration recommendations

## Future Considerations

1. **Enhanced Features**
   - Parameter impact visualization
   - Configuration templates
   - Batch configuration updates
   - Configuration comparison tools

2. **UX Improvements**
   - Guided configuration creation
   - Parameter recommendation system
   - Usage analytics integration
   - Advanced filtering and search

3. **Integration Opportunities**
   - Model performance metrics
   - Configuration sharing
   - Template marketplace
   - Automated optimization

## Implementation Notes

### Form Design

- Use 2-space indentation
- Maintain consistent spacing
- Group related parameters
- Clear validation feedback

### Mobile Considerations

- Simplified parameter groups
- Touch-friendly controls
- Progressive disclosure
- Optimized layout

### Performance Optimization

- Lazy loading of detailed information
- Efficient parameter validation
- Smooth transitions
- Responsive updates

## Additional Technical Details

### Command Line Parameters

The model alias parameters correspond to underlying LLM engine command line options, including:

#### 1. Common Parameters

- **threads**: Thread count control
- **ctx-size**: Context window size
- **predict**: Token prediction control
- **batch-size**: Processing batch size
- **flash-attn**: Flash attention support
- **rope-scaling**: RoPE frequency scaling
- **cache-type-k**: KV cache data type
- **parallel**: Parallel sequence decoding

#### 2. GPU-Related Parameters

- **device**: GPU device selection
- **gpu-layers**: VRAM layer storage
- **split-mode**: Multi-GPU splitting

#### 3. System Parameters

- **mlock**: RAM retention control
- **no-mmap**: Memory mapping control
- **timeout**: Server timeout settings
- **cache-reuse**: Cache chunk reuse

### Parameter Help Text

Each parameter includes detailed help text that should be exposed in the UI:

```text
temperature: "Number between 0.0 and 2.0. Higher values like will make the output
             more random, while lower values like 0.2 will make it more focused
             and deterministic."

top_p: "An alternative to sampling with temperature, called nucleus sampling.
       The model considers the results of the tokens with top_p probability mass."

frequency_penalty: "Number between -2.0 and 2.0. Positive values penalize new tokens
                  based on their existing frequency in the text so far."
```

### Model Resolution

Example model resolution from request to file path:

#### 1. Request format

```json
{
  "model": "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF:Q8_0",
  "stream": true,
  "messages": [{"role": "user", "content": "hello"}]
}
```

#### 2. Resolution path

```
~/.cache/huggingface/hub/models--QuantFactory--Meta-Llama-3-8B-Instruct-GGUF/snapshots/main/Meta-Llama-3-8B-Instruct.Q8_0.gguf
```

### Validation Considerations

#### 1. Parameter Ranges

- Model-specific valid ranges
- Runtime validation needed
- Default fallbacks

#### 2. File System

- Alias file naming restrictions
- Path-safe character replacement
- File existence checks

#### 3. Configuration

- Required fields validation
- Parameter compatibility
- Template availability

### Usage Metrics

Key success metrics for the feature:

- Number of requests served by user aliases vs model aliases
- Configuration experimentation frequency
- Parameter adjustment patterns
- Template usage statistics

## Interface Design Patterns

Based on the knowledge base and taking inspiration from the ModelFiles page design, here's a comprehensive redesign of the Model Alias interface.

### 1. Dashboard Overview

```text
┌─────────────────────────────────────────────────────────────┐
│ Model Aliases                          [+ Create New Alias] │
├─────────────────────────────────────────────────────────────┤
│ Quick Stats                                                 │
│ ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐ │
│ │ Total Aliases │ │ User Aliases │ │ Most Active Alias    │ │
│ │     24       │ │     8        │ │ llama2:chat (156/hr) │ │
│ └──────────────┘ └──────────────┘ └──────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 2. Enhanced List View with Grouping

```text
┌─────────────────────────────────────────────────────────────┐
│ 🔍 Search aliases...   │ Group by: [Model Family ▾]        │
├─────────────────────────────────────────────────────────────┤
│ Llama2 Family                                              │
│ ├── 📝 llama2:chat (User)                                  │
│ │   └── 7B Chat Q4_K_M  │ Temperature: 0.7 │ Active Now    │
│ │       [Try] [Edit] [Clone] [Delete]                      │
│ │                                                          │
│ ├── 🔒 llama2/7b-chat (Model)                             │
│ │   └── Default Config  │ [Try] [Create Custom]            │
│ │                                                          │
│ Phi Family                                                 │
│ ├── 📝 phi2:creative                                       │
│     └── Custom settings │ Last used: 2h ago │ [Details ▾]  │
└─────────────────────────────────────────────────────────────┘
```
