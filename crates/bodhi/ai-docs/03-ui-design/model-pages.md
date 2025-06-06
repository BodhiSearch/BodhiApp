# Model Pages UI Design

This document consolidates the UI/UX design specifications for all model-related pages in the Bodhi App, including Model Aliases, ModelFiles, and related interfaces.

## Model Alias Page Design

### Context & Purpose

The Model Alias page serves as a configuration management interface for AI model inference in the Bodhi application. It allows users to create and manage custom configurations for model inference, enabling fine-tuned control over model behavior.

### User Context

#### Primary Users
- AI Engineers (primary)
- End users (secondary)
- Technical users with ML/AI background

#### Key Characteristics
- AI Engineers build applications using LLM responses
- Need predictable model behavior through configuration
- Require experimentation capabilities for parameter tuning

#### Usage Patterns
- Frequent during initial model setup/tuning phase
- Infrequent once optimal configuration is found
- Reference existing configurations for new setups
- Copy/modify existing configurations for variants

### Enhanced List View Design

```text
┌──────────────────────────────────────────────────────────┐
│ Model Aliases                          [+ Create Alias]  │
├──────────────────────────────────────────────────────────┤
│ Quick Stats                                              │
│ ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐  │
│ │ User Aliases │ │Model Aliases │ │ Most Used        │  │
│ │     12      │ │     24       │ │ llama2:chat      │  │
│ └──────────────┘ └──────────────┘ └──────────────────┘  │
├──────────────────────────────────────────────────────────┤
│ [Family ▾] [Source ▾] [Sort ▾] 🔍___________________    │
├──────────────────────────────────────────────────────────┤
│ Llama2 Family                                            │
│ ├── 📝 llama2:chat (User)                               │
│ │   ├── Model: Llama-2-7B-Chat-GGUF                     │
│ │   ├── Usage: 156/hr | Success: 99.2%                  │
│ │   └── [Try] [Edit] [Clone] [Delete]                   │
│ │                                                        │
│ ├── 🔒 llama2/7b-chat (Model)                          │
│ │   ├── Model: Llama-2-7B-Chat-GGUF                     │
│ │   └── [Try] [Create Custom]                           │
│ │                                                        │
│ Phi Family                                               │
│ ├── 📝 phi2:creative                                    │
│     ├── Model: Phi-2-GGUF                               │
│     ├── Last used: 2h ago                               │
│     └── [Try] [Edit] [Clone] [Delete]                   │
└──────────────────────────────────────────────────────────┘
```

### Configuration Form Design

```text
┌──────────────────────────────────────────────────────────┐
│ Edit Alias: llama2:chat                    [Save] [Test] │
├──────────────────────────────────────────────────────────┤
│ Basic Settings                                           │
│ ┌─────────────────┐ ┌─────────────────┐ ┌────────────┐  │
│ │ Alias           │ │ Model           │ │ Template  ▾│  │
│ │ llama2:chat     │ │ Llama2 7B Chat  │ │ llama2     │  │
│ └─────────────────┘ └─────────────────┘ └────────────┘  │
├──────────────────────────────────────────────────────────┤
│ Generation Control ▾                                     │
│ ┌─────────────────────────────────────────────┐         │
│ │ Temperature                                 │         │
│ │ 0 ─────[|||]────── 2.0                     │         │
│ │         0.7                                 │         │
│ │ ℹ️ Higher values increase randomness        │         │
│ │                                             │         │
│ │ Top-p                                       │         │
│ │ 0 ─────[|||]────── 1.0                     │         │
│ │         0.9                                 │         │
│ │ ⚠️ Conflicts with Temperature if both set   │         │
│ └─────────────────────────────────────────────┘         │
├──────────────────────────────────────────────────────────┤
│ Performance Settings ▾                                   │
│ ┌─────────────────────────────────────────────┐         │
│ │ Context Size                                │         │
│ │ 512 ───[|||]────── 8192                    │         │
│ │        4096                                 │         │
│ └─────────────────────────────────────────────┘         │
└──────────────────────────────────────────────────────────┘
```

### Configuration Playground

```text
┌──────────────────────────────────────────────────────────┐
│ Configuration Playground           [Save] [Reset] [Share] │
├───────────────┬──────────────────────────────────────────┤
│ Test Input    │ Parameters                               │
│               │ ┌────────────────────────────────────┐   │
│ [Messages     │ │ Generation                     ▾   │   │
│  Thread]      │ │ • Temperature: 0.7                 │   │
│               │ │ • Top-p: 0.9                      │   │
│               │ │                                    │   │
│               │ │ Performance                    ▾   │   │
│               │ │ • Threads: 4                      │   │
│               │ │ • Context: 4096                   │   │
│               │ └────────────────────────────────────┘   │
│               │                                          │
│ [Type to test │ Template Preview                        │
│  config...]   │ ┌────────────────────────────────────┐  │
│               │ │ <s>You are a helpful...</s>        │  │
│ [Send]        │ │ <user>{{message}}</user>           │  │
└───────────────┴──────────────────────────────────────────┘
```

## ModelFiles Page Design

### Context & Purpose

The ModelFiles page serves as an inventory management interface for AI model files in the Bodhi application. This documentation outlines the user context, requirements, and design considerations for this critical system component.

### User Context

#### Primary Users
- System administrators (server mode)
- End users (individual mode)
- Technical users with ML/AI background

#### Usage Patterns
- Infrequent access (not a daily-use page)
- Primary use case: Storage management and model inventory
- Secondary use case: Model information reference

#### Key User Tasks

1. **Storage Management**
   - Identify large models for potential deletion
   - Review model inventory
   - Free up disk space (models typically 10GB+)

2. **Model Information**
   - View model metadata
   - Access model repository information
   - Compare model variants (quantization levels)

### Enhanced ModelFiles Interface

```text
┌──────────────────────────────────────────────────────────┐
│ Storage Dashboard                                        │
├──────────────────────────────────────────────────────────┤
│ Used: 127.4 GB of 500 GB  │  Models: 12  │ Available: ↑ │
├──────────────────────────────────────────────────────────┤
│ [Family ▾] [Size ▾] [Status ▾] 🔍___________________    │
├──────────────────────────────────────────────────────────┤
│ TheBloke/Llama-2-7B-Chat-GGUF                    ...    │
│ ↗ HF  │  ❤️ 2.3k  │  apache-2.0  │ #llama #chat        │
├──────────────────────────────────────────────────────────┤
│ ┌─────────┬──────┬─────────┬──────────┬───────────────┐ │
│ │ Variant │ Size │ Quality │ Status   │    Actions    │ │
│ ├─────────┼──────┼─────────┼──────────┼───────────────┤ │
│ │ Q4_K_M  │ 4 GB │ ⭐️⭐️⭐️⭐️ │ Active   │ Delete Info  │ │
│ │ Q5_K_M  │ 5 GB │ ⭐️⭐️⭐️⭐️ │ 45% ▰▰▱▱ │ Cancel Info  │ │
│ └─────────┴──────┴─────────┴──────────┴───────────────┘ │
├──────────────────────────────────────────────────────────┤
│ google/gemma-7b                                   ...    │
│ ↗ HF  │  ❤️ 5.1k  │  apache-2.0  │ #gemma              │
└──────────────────────────────────────────────────────────┘
```

### Unified Model Management Interface

```text
[Storage Dashboard]
Used: 127.4 GB of 500 GB
Downloaded Models: 12
Available for Download: 35+

[Quick Actions]
Browse Models | Manage Storage | Clean Up

[Search & Filters]
🔍 Search models...
[Family ▾] [Size ▾] [Status ▾] [Sort ▾]

Status Options:
- Downloaded
- Available
- Downloading
- Failed

Repository: TheBloke/Llama-2-7B-Chat-GGUF
Tags: Text Generation, PyTorch, 7B
[View on HuggingFace] [Show All Variants]

┌────────────────┬──────┬─────────┬────────────┬───────────┐
│ Variant        │ Size │ Quality │ Status     │ Actions   │
├────────────────┼──────┼─────────┼────────────┼───────────┤
│ Q4_K_M         │ 4.1G │ ⭐⭐⭐⭐  │ Downloaded │ [Delete]  │
│ Q5_K_M         │ 4.8G │ ⭐⭐⭐⭐⭐ │ Available  │ [Download]│
│ Q8_0           │ 7.2G │ ⭐⭐⭐⭐⭐ │ Available  │ [Download]│
└────────────────┴──────┴─────────┴────────────┴───────────┘
```

## Mobile Optimizations

### Model Alias Mobile Layout

```text
┌────────────────────────┐
│ Model Aliases     [+]  │
├────────────────────────┤
│ [Filters ▾] 🔍        │
├────────────────────────┤
│ Llama2 Family      >   │
├────────────────────────┤
│ 📝 llama2:chat        │
│ User Config           >│
│ 156/hr                 │
├────────────────────────┤
│ 🔒 llama2/7b-chat     │
│ Model Config         > │
└────────────────────────┘
```

### ModelFiles Mobile Layout

```text
┌────────────────────────┐
│ Storage: 127.4/500 GB  │
│ Models: 12             │
├────────────────────────┤
│ [Filters ▾] 🔍        │
├────────────────────────┤
│ TheBloke/Llama-2...   >│
│ ❤️ 2.3k #llama         │
├────────────────────────┤
│ Q4_K_M                 │
│ 4 GB │ Active         >│
├────────────────────────┤
│ Q5_K_M                 │
│ 5 GB │ 45% ▰▰▱▱       >│
└────────────────────────┘
```

## Design Principles

### Information Architecture

1. **Primary Information (Always Visible)**
   - Model Family/Repository
   - Size & Status
   - Quick Actions

2. **Secondary Information (Expandable)**
   - Technical Details
   - Performance Metrics
   - Related Models

3. **Tertiary Information (On Demand)**
   - Full Metadata
   - Usage History
   - Technical Hashes

### Visual Hierarchy

- Use icons to represent model families
- Color-code or badge different quantization levels
- Add visual indicators for active/inactive models
- Show storage usage trends

### Interaction Patterns

- Progressive disclosure for complex information
- Contextual actions based on model status
- Consistent navigation patterns
- Touch-friendly mobile interactions

## Accessibility Considerations

- Keyboard navigation support
- Screen reader compatibility
- High contrast mode support
- Focus management
- ARIA labels for complex interactions

## Performance Considerations

- Lazy loading of detailed information
- Efficient parameter validation
- Smooth transitions
- Responsive updates
- Optimized for large model lists

---

*This consolidated design specification ensures consistent and intuitive model management experiences across all Bodhi App interfaces.*
