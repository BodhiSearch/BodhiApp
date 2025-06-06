# ModelFiles Page UI/UX Revamp

## User Story

As a Bodhi App User
I want a unified interface for discovering, downloading, and managing AI models
So that I can efficiently manage my local models and discover new ones

## Background

- Currently have separate pages for model files and downloads
- Manual model download process requiring HuggingFace navigation
- Limited model metadata and discovery features
- No storage management visibility
- No trending/featured models section

## Acceptance Criteria

### Phase 1: Core Refactoring

#### Backend Changes

1. **New Storage API endpoint:**

```typescript
interface StorageInfo {
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
  model_count: number;
}
```

2. **Enhanced Featured Models API:**

```typescript
interface FeaturedModel {
  id: string;
  title: string;
  description: string;
  repo: string;
  recommended_variant: {
    filename: string;
    size: number;
  };
  metadata: {
    downloads_24h?: number;
    stars?: number;
    likes?: number;
    tags: string[];
  };
  links: {
    huggingface?: string;
    announcement?: string;
    paper?: string;
  };
  published_at: string;
}
```

3. **Enhanced Model Metadata API:**

```typescript
interface ModelFileMetadata {
  repo: string;
  model_family?: string;
  architecture?: string;
  parameters?: number;
  quantization?: {
    bits: number;
    method: string;
  };
  license?: string;
  tags: string[];
  performance_metrics?: {
    speed_rating: number;
    memory_rating: number;
    quality_rating: number;
  };
}
```

#### Frontend Changes

1. **Storage Dashboard Component:**
   - Display total/used/available storage
   - Show model count
   - Quick actions for storage management

2. **Enhanced Table View:**
   - Unified view of downloaded and available models
   - Status indicators (downloaded, downloading, available)
   - Quick actions based on model status
   - Responsive design for mobile

3. **Model Details Overlay:**
   - Comprehensive model information
   - Performance metrics visualization
   - Download variant selection
   - Links to documentation/resources

### Phase 2: Download Integration

1. **Download Progress Tracking:**
   - Real-time progress updates
   - Multiple concurrent downloads
   - Download queue management
   - Error handling and retry

2. **Smart Download Dialog:**
   - Storage impact preview
   - Variant recommendations
   - Quick download option
   - Space availability check

### Phase 3: Discovery Features

1. **Trending Models Section:**
   - Featured model highlight
   - Trending models carousel
   - One-click download
   - Learn more overlay

2. **Search and Filters:**
   - Model family filter
   - Size category filter
   - Status filter
   - Sort options

## Testing Requirements

1. **Component Testing:**
   - Storage dashboard functionality
   - Table view interactions
   - Download progress tracking
   - Model details overlay

2. **Integration Testing:**
   - Download workflow
   - Storage updates
   - Filter interactions
   - Search functionality

3. **Responsive Testing:**
   - Mobile layout verification
   - Touch interactions
   - Overlay behavior on mobile

## Technical Implementation

### API Endpoints

1. `GET /api/v1/storage` - Storage information
2. `GET /api/v1/featured-models` - Enhanced featured models
3. `GET /api/v1/modelfiles/{repo}/metadata` - Enhanced metadata

### Component Structure

1. **StorageDashboard**
   - Storage metrics
   - Quick actions

2. **ModelFilesTable**
   - Enhanced table view
   - Status indicators
   - Action buttons

3. **ModelDetailsOverlay**
   - Metadata display
   - Performance metrics
   - Download options

4. **TrendingModels**
   - Featured section
   - Model carousel
   - Quick download

## Not In Scope

- Historical storage tracking
- Model comparison features
- User preferences/settings
- Batch operations
- Performance benchmarking

## Dependencies

- HuggingFace API for model metadata
- Storage monitoring system
- Download manager service
- Real-time progress tracking

## Migration Strategy

- Direct replacement of existing pages
- No backward compatibility required
- No user preference migration needed

## Future Considerations

1. **Enhanced Features:**
   - Model comparison
   - Usage analytics
   - Performance benchmarks

2. **Storage Management:**
   - Cleanup recommendations
   - Storage optimization
   - Usage trends

3. **Discovery:**
   - Personalized recommendations
   - Usage-based suggestions
   - Community ratings

## Phase 1: Detailed Design

### UI Layout - Desktop

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

### UI Layout - Mobile

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

### API Structure

```typescript
interface ModelRepository {
  repo: string;
  metadata: {
    likes: number;
    stars: number;
    license: string;
    tags: string[];
    description: string;
    links: {
      huggingface: string;
      homepage?: string;
      paper?: string;
    }
  };
  variants: ModelVariant[];
}

interface ModelVariant {
  filename: string;
  size: number;
  status: 'downloaded' | 'downloading' | 'available';
  progress?: number;
  metadata: {
    architecture: string;
    parameters: number;
    quantization: {
      bits: number;
      method: string;
    };
    performance_metrics: {
      speed: number;
      memory: number;
      quality: number;
    }
  }
}

// Single API endpoint returns complete data
GET /api/v1/modelfiles
Response: {
  storage: {
    total_bytes: number;
    used_bytes: number;
    available_bytes: number;
    model_count: number;
  };
  repositories: ModelRepository[];
}
```

### Component Hierarchy

```text
ModelFilesPage
├── StorageDashboard
│   └── StorageMetrics
├── FilterBar
│   ├── FamilyFilter
│   ├── SizeFilter
│   ├── StatusFilter
│   └── SearchInput
└── RepositoryList
    └── RepositoryCard
        ├── RepositoryHeader
        │   └── MetadataBadges
        └── VariantsTable
            └── VariantRow
                ├── StatusIndicator
                ├── ProgressBar
                └── ActionButtons
```

### Mobile Interactions

1. **Touch Targets:**
   - Minimum 44x44px touch areas
   - Swipeable repository cards
   - Bottom sheet for filters
   - Full-screen overlays for details

2. **Progressive Disclosure:**
   - Collapse repository metadata
   - Show essential variant info
   - Expand to full details on tap
   - Quick actions in swipe menu

### Data Flow

1. **Page Load:**
   - Fetch complete model data
   - Initialize storage dashboard
   - Apply default filters
   - Render repository list

2. **User Interactions:**
   - Filter updates -> client-side filtering
   - Repository expand -> show variants
   - Variant actions -> API calls
   - Search -> client-side search

3. **Real-time Updates:**
   - WebSocket for download progress
   - Storage metrics refresh
   - Status indicators update

### Implementation Tasks

1. **API Development:**
   - [ ] Design unified ModelRepository schema
   - [ ] Implement combined modelfiles endpoint
   - [ ] Add WebSocket for progress updates
   - [ ] Create storage metrics endpoint

2. **Component Development:**
   - [ ] Build StorageDashboard component
   - [ ] Create FilterBar with responsive design
   - [ ] Implement RepositoryCard component
   - [ ] Develop VariantsTable component
   - [ ] Add mobile-specific interactions

3. **State Management:**
   - [ ] Set up client-side filtering
   - [ ] Implement search functionality
   - [ ] Handle download progress updates
   - [ ] Manage expanded/collapsed states

4. **Testing:**
   - [ ] Unit tests for components
   - [ ] Integration tests for data flow
   - [ ] Mobile interaction testing
   - [ ] Accessibility testing

## Phase 2: Download Integration

### Component Integration

```text
┌─ ModelFiles Page ───────────────────────────────────────┐
│ [Storage Dashboard]                                     │
│ Used: 127.4 GB │ Models: 12 │ [Download Model] [Queue] │
├──────────────────────────────────────────────────────────┤
│ 🔍 Filter downloaded models...                          │
│ [Main Table Content...]                                 │
└──────────────────────────────────────────────────────────┘

┌─ Download Model Overlay ─────────────────────────────────┐
│ Download New Model                                       │
├──────────────────────────────────────────────────────────┤
│ Enter Repository URL or Name:                            │
│ ┌────────────────────────────────────────┐              │
│ │ https://huggingface.co/org/repo        │              │
│ └────────────────────────────────────────┘              │
│                                                          │
│ [Repository Preview]                                     │
│ Same layout as main table repository card                │
│                                                          │
│ [Cancel] [Download Selected]                            │
└──────────────────────────────────────────────────────────┘

┌─ Download Queue Overlay ─────────────────────────────────┐
│ Active Downloads                                         │
├──────────────────────────────────────────────────────────┤
│ Llama-2 Q8_0                                            │
│ 7.2 GB │ 45% ▰▰▰▱▱▱▱ │ [Cancel]                        │
│                                                          │
│ Phi-2 Q4_K_M                                            │
│ 3.1 GB │ Queued │ [Cancel]                             │
└──────────────────────────────────────────────────────────┘
```

### Mobile Layout

```text
┌─ Download Model ─────┐
│ ← Back              │
├─────────────────────┤
│ Enter URL/Name:     │
│ [________________] │
│                     │
│ [Preview Card]      │
│ Tap to expand       │
│                     │
│ [Download] [Cancel] │
└─────────────────────┘

┌─ Queue ─────────────┐
│ ← Downloads         │
├─────────────────────┤
│ Llama-2 Q8_0       │
│ 45% ▰▰▰▱▱▱▱        │
│ [Cancel Download]   │
├─────────────────────┤
│ Phi-2 Q4_K_M       │
│ Queued             │
│ [Cancel]           │
└─────────────────────┘
```

### User Flows

#### 1. Download New Model

```
User clicks "Download Model"
-> Opens overlay
-> Enters repo URL/name
-> System fetches & displays repo info
-> User selects variant
-> System validates storage
-> Adds to download queue
-> Shows in queue overlay
```

#### 2. Monitor Downloads

```
User clicks "Queue" button
-> Opens queue overlay
-> Shows active downloads
-> Progress updates real-time
-> Complete downloads appear in main table
```

#### 3. Handle Failures

```
Download fails
-> Shows error in queue
-> Provides retry option
-> Logs error details
-> Offers troubleshooting
```

### Download States

1. **Available** - Not downloaded, can be downloaded
2. **Queued** - Added to download queue, waiting
3. **Downloading** - Currently downloading with progress
4. **Downloaded** - Successfully downloaded and available
5. **Failed** - Download failed, retry available
6. **Cancelled** - User cancelled download

### Storage Validation

Before starting downloads:

1. **Check available space**
2. **Warn if insufficient space**
3. **Suggest cleanup options**
4. **Allow user to proceed or cancel**

### Error Handling

1. **Network errors** - Retry with exponential backoff
2. **Storage errors** - Clear error messages and suggestions
3. **Authentication errors** - Guide to API key setup
4. **Validation errors** - Clear feedback on invalid URLs

### Progress Tracking

1. **Real-time updates** via WebSocket
2. **Persistent across page refreshes**
3. **Background downloads continue**
4. **Notification on completion**

### Queue Management

1. **Multiple concurrent downloads** (configurable limit)
2. **Priority ordering** (user can reorder)
3. **Pause/resume capability**
4. **Automatic retry on failure**

### Integration Points

1. **Storage dashboard** updates in real-time
2. **Main table** shows download status
3. **Filter system** includes download states
4. **Search** works across all models
