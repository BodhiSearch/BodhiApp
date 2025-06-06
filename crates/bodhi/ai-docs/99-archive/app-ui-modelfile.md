# ModelFiles Page Design

## Context & Purpose

The ModelFiles page serves as an inventory management interface for AI model files in the Bodhi application. This documentation outlines the user context, requirements, and design considerations for this critical system component.

## User Context

### Primary Users

- System administrators (server mode)
- End users (individual mode)
- Technical users with ML/AI background

### Usage Patterns

- Infrequent access (not a daily-use page)
- Primary use case: Storage management and model inventory
- Secondary use case: Model information reference

### Key User Tasks

1. **Storage Management**
   - Identify large models for potential deletion
   - Review model inventory
   - Free up disk space (models typically 10GB+)

2. **Model Information**
   - View model metadata
   - Access model repository information
   - Compare model variants (quantization levels)

## Information Architecture

### Data Hierarchy

**Primary Information (Table Columns)**

1. **Repository (Highest Priority)**
   - Indicates model family (Gemma, Llama, Phi, etc.)
   - Critical for model identification

2. **Filename (High Priority)**
   - Shows quantization level
   - Indicates model variant

3. **Size (High Priority)**
   - Critical for storage management
   - Proposed categorization:
     - XXS (Extra Extra Small Language Model)
     - XS (Extra Small Language Model)
     - S (Small Language Model)
     - M (Medium Language Model)
     - L (Large Language Model)
     - 2XL (Double Extra Large Language Model)
     - 3XL (Triple Extra Large Language Model)

4. **Updated At (Secondary)**
   - General information
   - Timestamp for reference

5. **Snapshot (Technical Reference)**
   - Hash identifier
   - Primarily for technical users

## Detailed Information Display

### File-Level Information Overlay

- Trigger via info/details button at file level
- Modal overlay displays comprehensive file information
- Sections in overlay:
  1. **Model Metadata**
     - Architecture
     - Parameters
     - Quantization details
     - Format specifications
  2. **Performance Metrics**
     - Memory requirements
     - Speed indicators
     - Quality metrics
  3. **Technical Details**
     - Full snapshot hash
     - Exact size
     - Configuration details

### Repository Information Overlay

- Accessible via repo name link
- Displays HuggingFace repository homepage in overlay
- Provides context about:
  - Original model
  - Documentation
  - Usage examples
  - Community information

## Interaction Design

### Current Actions

- View model list
- Sort by columns
- Expand rows for details
- Paginate through results

### Proposed Enhancements

1. **Model Management**
   - Delete models
   - Direct link to Hugging Face repository
   - Model download integration

2. **Visualization**
   - Size category indicators
   - Last used indicators
   - Repository grouping

3. **Search & Filter**
   - Repository name search
   - Size category filter
   - Quick filters for common queries

## Layout Considerations

### Page Structure

- Responsive design (mobile/desktop)
- Pagination:
  - Desktop: 20-30 items
  - Mobile: 10 items
- Replace with overlay-based detailed views
- Table remains flat with alternating row colors
- Quick actions remain visible at row level
- Info button triggers detailed overlay

### Visual Hierarchy

1. **Primary Level**
   - Repository name
   - Size category
   - Quantization level
   - Action buttons

2. **Secondary Level**
   - Filename details
   - Updated timestamp
   - Technical metadata

3. **Tertiary Level**
   - Expanded details
   - Technical specifications
   - Full hashes

### Table Layout

- Repository name as header with HuggingFace link
- Repository metadata (likes, license, key tags)
- Model files with alternating row colors
- No collapsible sections
- Quick actions at model level only

## Performance Considerations

### Data Loading

- Initial load: 10 items (current)
- Proposed: Adaptive loading
  - Desktop: 20-30 items
  - Mobile: 10 items
- Progressive loading for expanded details

### User Base Scale

- Typical users: 5-6 model files
- Power users: ~20 model files
- Page design should accommodate both scenarios

## Integration Points

### Application Flow

- Part of model management workflow
- Connected to model download system
- Potential integration with model performance comparison

### Related Features

- Model download progress tracking
- Storage management system
- Model inference system

## Technical Requirements

### Data Display

```typescript
interface ModelFile {
  repo: string;
  filename: string;
  size: number;
  updated_at: string;
  snapshot: string;
  metadata: {
    architecture: string;
    parameters: number;
    quantization: string;
    // Additional metadata fields
  };
}
```

## Future Considerations

1. **Enhanced Features**
   - Model comparison tools
   - Performance metrics
   - Usage statistics
   - Storage optimization recommendations

2. **UX Improvements**
   - Saved searches
   - Custom views
   - Batch operations
   - Advanced filtering

3. **Integration Opportunities**
   - Model download integration
   - Performance monitoring
   - Storage analytics

## UI/UX Expert Analysis & Recommendations

Based on the analysis and the provided context, here are key findings and recommendations:

### Key Findings

1. **Purpose Mismatch**
   - The current interface treats this as a generic data table, but it's primarily a storage management tool
   - Users mainly visit this page to manage disk space and find model information
   - The technical information (like snapshot hashes) is given equal visual weight to critical information

2. **Information Overload**
   - All metadata is treated equally in the expanded view
   - File sizes are shown in raw GB format without context
   - Technical details (snapshots) take up valuable screen space

3. **Limited User Actions**
   - No direct actions for the primary use case (storage management)
   - Missing quick links to related actions (download, delete)
   - Expandable rows don't provide actionable information

### Recommendations

#### 1. Redesign Primary View

```
[Storage Usage Overview]
----------------------
Total Space Used: 127.4 GB
Available Space: 234.6 GB
Models: 12

[Quick Actions]
Download New Model | Clean Up Space
```

#### 2. Improve Table Layout

```
[Model Family Icon] Llama-2
├── 7B Chat (Q4_K_M)  4.08 GB  ⚡️Active
├── 7B Chat (Q3_K_M)  3.30 GB  Last used 2 weeks ago
└── 7B Chat (Q2_K)    2.53 GB  Unused

[Model Family Icon] Phi
├── 3.5 Mini (Q8_0)   3.78 GB  ⚡️Active
└── 3.5 Mini (Q4_K_M) 2.95 GB  Last used 1 month ago
```

#### 3. Enhanced Row Expansion

```
[Model Details]        [Quick Actions]
Architecture: Llama    Delete Model
Parameters: 7B        View on HuggingFace
Format: GGUF          Download Alternative

[Performance]         [Storage Impact]
Speed: ⭐️⭐️⭐️⭐️      Size: 4.08 GB
Memory: ⭐️⭐️⭐️        Category: Medium
Quality: ⭐️⭐️⭐️⭐️     Compression: Q4
```

#### 4. Contextual Actions

- Add a "Storage Management" mode that highlights size-related information
- Implement batch selection for multi-model cleanup
- Show size impact before deletion

#### 5. Visual Improvements

- Use icons to represent model families
- Color-code or badge different quantization levels
- Add visual indicators for active/inactive models
- Show storage usage trends

#### 6. Mobile Optimizations

- Collapse model families by default
- Show critical information first (size, status)
- Optimize expanded view for touch interfaces
- Add pull-to-refresh for updates

#### 7. Progressive Enhancement

- Add search with common filters
- Implement "recently used" section
- Show related models when viewing details
- Add quick comparisons between quantization levels

#### 8. Information Architecture

```
Primary (Always Visible)
- Model Family
- Size & Status
- Quick Actions

Secondary (Expandable)
- Technical Details
- Performance Metrics
- Related Models

Tertiary (On Demand)
- Full Metadata
- Usage History
- Technical Hashes
```

These recommendations focus on:

- Making storage management more intuitive
- Providing context for technical information
- Streamlining common actions
- Improving information hierarchy
- Enhancing mobile usability

## Repository Display

### Grouping & Visual Design

- Group models by repository
- Use alternating row colors for visual separation
- No collapsible groups to minimize clicks
- Repository name links to HuggingFace page

### Repository Metadata

Display key HuggingFace repository information:

- Repository likes count
- License information
- Key tags (limited selection):
  - Model type (Text Generation, etc.)
  - Framework tags (PyTorch, etc.)
  - Language tags
  - Relevant paper references
  - Special capabilities

## Filtering System

### 1. Model Family Filter

- Multi-select toggle buttons
- Options include all detected families plus "Unknown"
- No inference from filenames
- Shows model count per family
- Blank/Unknown for missing metadata

### 2. Size Category Filter

- Multi-select toggle buttons
- Categories:
  - XXS (Extra Extra Small)
  - XS (Extra Small)
  - S (Small)
  - M (Medium)
  - L (Large)
  - 2XL (Double Extra Large)
  - 3XL (Triple Extra Large)
- Shows model count per category

### Filter Behavior

- AND relationship between family and size filters
- Filters persist in browser local storage
- Real-time update of matching model counts
- Clear all filters option

## Actions & Controls

### Model-Level Actions

- Delete model
- View on HuggingFace
- Additional model-specific actions

### Storage Display

- Total storage used
- Available storage
- Number of models

### Interface Updates

```typescript
interface RepositoryMetadata {
  likes: number;
  license: string;
  tags: string[];
  paperReference?: string;
}

interface FilterState {
  modelFamilies: string[];
  sizeCategories: string[];
}

// Local storage schema
interface StoredPreferences {
  filters: FilterState;
  // Other user preferences
}
```

### Modal Overlay Behavior

- Full-screen on mobile devices
- Large centered modal on desktop
- Dismissible via:
  - Close button
  - Click outside
  - Escape key
- Maintains scroll position in main table
- Supports keyboard navigation

## Unified Model Management Interface

### Analysis of Current Pain Points

1. Disconnected workflows - Users have to switch between HuggingFace, download page, and modelfiles page
2. Manual input prone to errors
3. No visibility of available files before download
4. No indication of already downloaded files
5. Context switching between management and acquisition of models

### Enhanced Storage Overview

```text
[Storage Dashboard]
Used: 127.4 GB of 500 GB
Downloaded Models: 12
Available for Download: 35+

[Quick Actions]
Browse Models | Manage Storage | Clean Up
```

### Unified Search & Browse Interface

```text
[Search & Filters]
🔍 Search models...
[Family ▾] [Size ▾] [Status ▾] [Sort ▾]

Status Options:
- Downloaded
- Available
- Downloading
- Failed
```

### Integrated Table View

```text
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
