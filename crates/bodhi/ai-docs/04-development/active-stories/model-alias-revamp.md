# Model Alias Page UI/UX Revamp

## User Story

As a Bodhi App User
I want an intuitive interface for managing and testing model configurations
So that I can efficiently create, modify, and validate model aliases for my use cases

## Background

- Current interface is basic with limited user guidance
- Parameter configuration requires deep technical knowledge
- No built-in way to test configurations
- Limited organization and discovery features
- No usage analytics or metrics

## Acceptance Criteria

### Phase 1: Enhanced List View

#### Backend Changes

1. **Usage Metrics API:**

```typescript
interface AliasMetrics {
  alias: string;
  usage_count: number;
  last_used: string;
  success_rate: number;
}
```

2. **Enhanced Alias List API:**

```typescript
interface EnhancedAlias {
  alias: string;
  source: 'user' | 'model';
  repo: string;
  filename: string;
  metrics: AliasMetrics;
  model_family?: string;
  quantization?: string;
}
```

#### Frontend Changes

1. **Grouped List View:**
   - Group by model family
   - Visual distinction between user/model aliases
   - Quick actions based on alias type
   - Usage statistics display

2. **Enhanced Search & Filter:**
   - Filter by model family
   - Filter by source type
   - Search by alias name
   - Sort by usage/last used

3. **List Item Components:**
   - Status indicators
   - Quick action buttons
   - Usage metrics
   - Expandable details

### Phase 2: Configuration Management

#### Backend Changes

1. **Parameter Validation API:**
   - Model-specific range validation
   - Parameter conflict detection
   - Detailed error messages

2. **Temporary Alias Support:**
   - Create temporary configurations
   - Auto-cleanup unused temps
   - Convert temp to permanent

#### Frontend Changes

1. **Enhanced Form Interface:**
   - Grouped parameter sections
   - Parameter descriptions
   - Visual range indicators
   - Conflict warnings

2. **Quick Actions:**
   - Clone existing alias
   - Create from model
   - Import/Export configs
   - Delete with confirmation

3. **Validation Feedback:**
   - Real-time field validation
   - Error message display
   - Range visualization
   - Conflict indicators

### Phase 3: Configuration Playground

#### Backend Changes

1. **Test Configuration API:**
   - Use existing chat endpoints
   - Support temporary aliases
   - Parameter validation

#### Frontend Changes

1. **Playground Interface:**
   - Split view design
   - Parameter controls
   - Test input area
   - Response preview

2. **Parameter Testing:**
   - Real-time adjustments
   - Save/reset options
   - Compare configurations
   - Performance feedback

3. **Template Testing:**
   - Template input area
   - Variable substitution
   - Format validation
   - Preview rendering

## Technical Implementation

### API Structure

```typescript
// Metrics API
GET /api/v1/alias/metrics
Response: {
  aliases: AliasMetrics[];
}

// Enhanced List API
GET /api/v1/alias/list
Response: {
  aliases: EnhancedAlias[];
  total: number;
  page_size: number;
}

// Temporary Alias
POST /api/v1/alias/temp
Body: AliasConfig
Response: {
  temp_id: string;
  config: AliasConfig;
}
```

### Component Structure

1. **List View Components:**

```
ModelAliasPage
├── FilterBar
│   ├── ModelFamilyFilter
│   ├── SourceTypeFilter
│   └── SearchInput
├── GroupedAliasList
│   └── AliasCard
│       ├── QuickActions
│       ├── MetricsDisplay
│       └── ExpandedDetails
└── ActionPanel
    └── QuickActionButtons
```

2. **Configuration Components:**

```
AliasForm
├── ParameterGroups
│   ├── GenerationControl
│   └── PerformanceSettings
├── ValidationDisplay
└── ActionButtons
```

3. **Playground Components:**

```
ConfigPlayground
├── ParameterPanel
├── TestInput
├── ResponsePreview
└── ActionButtons
```

## Testing Requirements

1. **Component Testing:**
   - Filter functionality
   - Group collapsing
   - Action handlers
   - Form validation

2. **Integration Testing:**
   - API integration
   - Metric updates
   - Configuration flow
   - Playground interaction

3. **Validation Testing:**
   - Parameter ranges
   - Conflict detection
   - Error handling
   - Form submission

## Mobile Considerations

1. **List View:**
   - Collapsible groups
   - Touch-friendly actions
   - Simplified metrics
   - Swipe actions

2. **Configuration:**
   - Full-screen editors
   - Stepped form flow
   - Touch-optimized controls
   - Keyboard handling

3. **Playground:**
   - Tab-based navigation
   - Responsive split view
   - Touch-friendly controls
   - Mobile-first design

## Not In Scope

- Backup/restore functionality
- Historical configuration tracking
- Advanced analytics
- Batch operations
- Chat interface integration

## Dependencies

- Backend validation API
- Metrics tracking system
- Template processing
- Parameter validation

## Migration Strategy

- Progressive enhancement
- No data migration needed
- Feature flag for new UI
- Parallel old/new views

## Future Considerations

1. **Enhanced Features:**
   - Configuration templates
   - Preset libraries
   - Advanced analytics
   - Batch operations

2. **Integration:**
   - Chat interface connection
   - Model performance metrics
   - Usage recommendations
   - Community sharing

3. **Analytics:**
   - Usage patterns
   - Performance tracking
   - Error analysis
   - Optimization suggestions

## Phase 1: Detailed Design

### UI Layout - Desktop

```
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

### UI Layout - Mobile

```
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

## Phase 2: Detailed Design

### Configuration Form - Desktop

```
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

## Phase 3: Detailed Design

### Playground Interface - Desktop

```
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

### Mobile Playground

```
┌────────────────────────┐
│ Playground        [⚙️] │
├────────────────────────┤
│ [Test] [Template] [Cfg]│
├────────────────────────┤
│ Test Input            │
│                       │
│ [Message Thread]      │
│                       │
│ [Type message...]     │
│                       │
│ [Send]               >│
└────────────────────────┘
```

### Parameter Control - Mobile

```
┌────────────────────────┐
│ Temperature        [×] │
├────────────────────────┤
│ 0 ────[|||]───── 2.0  │
│        0.7            │
│                       │
│ ℹ️ Controls randomness │
│ 💡 Try 0.7 for chat   │
│                       │
│ [Cancel] [Apply]      │
└────────────────────────┘
```

## User Onboarding Design

### First Visit Experience

```
┌──────────────────────────────────────────────────────────┐
│ Welcome to Model Configurations!     [Dismiss] [Tour]    │
├──────────────────────────────────────────────────────────┤
│ Customize and optimize your AI models with configuration │
│ profiles. Start with model defaults or create your own   │
│ configurations for specific use cases.                   │
└──────────────────────────────────────────────────────────┘
```

### Feature Spotlights

Sequential tooltips highlighting key features:

#### 1. Model Alias Overview

```
┌─ Tooltip ──────────────────┐
│ Model Configurations       │
│ View and manage different  │
│ configurations for your    │
│ AI models                  │
│ [1/5] [Skip] [Next →]     │
└──────────────────────────┘
   ↓
[Model Alias List]
```

#### 2. Configuration Types

```
┌─ Tooltip ────────────────┐
│ User vs Model Configs   │
│ 🔒 Model: Built-in      │
│ 📝 User: Customizable   │
│ [2/5] [Skip] [Next →]   │
└────────────────────────┘
   ↓
[Configuration Types]
```

#### 3. Parameter Groups

```
┌─ Tooltip ──────────────────┐
│ Parameter Categories       │
│ Generation: Output control │
│ Performance: System tuning │
│ [3/5] [Skip] [Next →]     │
└──────────────────────────┘
   ↓
[Parameter Groups]
```

#### 4. Configuration Testing

```
┌─ Tooltip ──────────────────┐
│ Test Your Configurations   │
│ Try different settings     │
│ before saving changes      │
│ [4/5] [Skip] [Next →]     │
└──────────────────────────┘
   ↓
[Test Button]
```

#### 5. Quick Actions

```
┌─ Tooltip ──────────────────┐
│ Quick Actions              │
│ Clone, edit, or delete     │
│ configurations easily      │
│ [5/5] [Skip] [Finish]     │
└──────────────────────────┘
   ↓
[Action Buttons]
```

### Progressive Disclosure

1. **Basic View (Default)**
   - Essential parameters only
   - Simplified interface
   - Common use cases

2. **Advanced View (Optional)**
   - All parameters visible
   - Technical details
   - Expert configurations

3. **Expert Mode (Power Users)**
   - Raw parameter editing
   - Validation bypass
   - Advanced features

### Help Integration

1. **Contextual Help**
   - Parameter tooltips
   - Range explanations
   - Conflict warnings

2. **Documentation Links**
   - Parameter guides
   - Best practices
   - Example configurations

3. **Community Resources**
   - Shared configurations
   - User examples
   - Discussion forums

### Accessibility Features

1. **Keyboard Navigation**
   - Tab order optimization
   - Keyboard shortcuts
   - Focus indicators

2. **Screen Reader Support**
   - ARIA labels
   - Descriptive text
   - Status announcements

3. **Visual Accessibility**
   - High contrast mode
   - Font size scaling
   - Color blind support

### Performance Optimization

1. **Lazy Loading**
   - Load configurations on demand
   - Progressive enhancement
   - Minimal initial payload

2. **Caching Strategy**
   - Local storage for preferences
   - API response caching
   - Optimistic updates

3. **Error Recovery**
   - Graceful degradation
   - Retry mechanisms
   - Offline support
