=============================
ModelFiles Page UI/UX Revamp
=============================

User Story
---------
As a Bodhi App User
I want a unified interface for discovering, downloading, and managing AI models
So that I can efficiently manage my local models and discover new ones

Background
---------
- Currently have separate pages for model files and downloads
- Manual model download process requiring HuggingFace navigation
- Limited model metadata and discovery features
- No storage management visibility
- No trending/featured models section

Acceptance Criteria
-----------------

Phase 1: Core Refactoring
~~~~~~~~~~~~~~~~~~~~~~~~

Backend Changes
^^^^^^^^^^^^^
1. New Storage API endpoint:

.. code-block:: typescript

    interface StorageInfo {
      total_bytes: number;
      used_bytes: number;
      available_bytes: number;
      model_count: number;
    }

2. Enhanced Featured Models API:

.. code-block:: typescript

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

3. Enhanced Model Metadata API:

.. code-block:: typescript

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

Frontend Changes
^^^^^^^^^^^^^^
1. Storage Dashboard Component:
   - Display total/used/available storage
   - Show model count
   - Quick actions for storage management

2. Enhanced Table View:
   - Unified view of downloaded and available models
   - Status indicators (downloaded, downloading, available)
   - Quick actions based on model status
   - Responsive design for mobile

3. Model Details Overlay:
   - Comprehensive model information
   - Performance metrics visualization
   - Download variant selection
   - Links to documentation/resources

Phase 2: Download Integration
~~~~~~~~~~~~~~~~~~~~~~~~~~~
1. Download Progress Tracking:
   - Real-time progress updates
   - Multiple concurrent downloads
   - Download queue management
   - Error handling and retry

2. Smart Download Dialog:
   - Storage impact preview
   - Variant recommendations
   - Quick download option
   - Space availability check

Phase 3: Discovery Features
~~~~~~~~~~~~~~~~~~~~~~~~~
1. Trending Models Section:
   - Featured model highlight
   - Trending models carousel
   - One-click download
   - Learn more overlay

2. Search and Filters:
   - Model family filter
   - Size category filter
   - Status filter
   - Sort options

Testing Requirements
------------------
1. Component Testing:
   - Storage dashboard functionality
   - Table view interactions
   - Download progress tracking
   - Model details overlay

2. Integration Testing:
   - Download workflow
   - Storage updates
   - Filter interactions
   - Search functionality

3. Responsive Testing:
   - Mobile layout verification
   - Touch interactions
   - Overlay behavior on mobile

Technical Implementation
----------------------

API Endpoints
~~~~~~~~~~~
1. ``GET /api/v1/storage`` - Storage information
2. ``GET /api/v1/featured-models`` - Enhanced featured models
3. ``GET /api/v1/modelfiles/{repo}/metadata`` - Enhanced metadata

Component Structure
~~~~~~~~~~~~~~~~
1. StorageDashboard
   - Storage metrics
   - Quick actions

2. ModelFilesTable
   - Enhanced table view
   - Status indicators
   - Action buttons

3. ModelDetailsOverlay
   - Metadata display
   - Performance metrics
   - Download options

4. TrendingModels
   - Featured section
   - Model carousel
   - Quick download

Not In Scope
-----------
- Historical storage tracking
- Model comparison features
- User preferences/settings
- Batch operations
- Performance benchmarking

Dependencies
-----------
- HuggingFace API for model metadata
- Storage monitoring system
- Download manager service
- Real-time progress tracking

Migration Strategy
---------------
- Direct replacement of existing pages
- No backward compatibility required
- No user preference migration needed

Future Considerations
------------------
1. Enhanced Features:
   - Model comparison
   - Usage analytics
   - Performance benchmarks

2. Storage Management:
   - Cleanup recommendations
   - Storage optimization
   - Usage trends

3. Discovery:
   - Personalized recommendations
   - Usage-based suggestions
   - Community ratings

@@ Phase 1 Detailed Requirements @@

Phase 1: Detailed Design
----------------------

UI Layout - Desktop
~~~~~~~~~~~~~~~~~
.. code-block::

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

UI Layout - Mobile
~~~~~~~~~~~~~~~~
.. code-block::

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

API Structure
~~~~~~~~~~~
.. code-block:: typescript

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

Component Hierarchy
~~~~~~~~~~~~~~~~
.. code-block::

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

Mobile Interactions
~~~~~~~~~~~~~~~~
1. Touch Targets:
   - Minimum 44x44px touch areas
   - Swipeable repository cards
   - Bottom sheet for filters
   - Full-screen overlays for details

2. Progressive Disclosure:
   - Collapse repository metadata
   - Show essential variant info
   - Expand to full details on tap
   - Quick actions in swipe menu

Data Flow
~~~~~~~~
1. Page Load:
   - Fetch complete model data
   - Initialize storage dashboard
   - Apply default filters
   - Render repository list

2. User Interactions:
   - Filter updates -> client-side filtering
   - Repository expand -> show variants
   - Variant actions -> API calls
   - Search -> client-side search

3. Real-time Updates:
   - WebSocket for download progress
   - Storage metrics refresh
   - Status indicators update

Implementation Tasks
~~~~~~~~~~~~~~~~~
1. API Development:
   - [ ] Design unified ModelRepository schema
   - [ ] Implement combined modelfiles endpoint
   - [ ] Add WebSocket for progress updates
   - [ ] Create storage metrics endpoint

2. Component Development:
   - [ ] Build StorageDashboard component
   - [ ] Create FilterBar with responsive design
   - [ ] Implement RepositoryCard component
   - [ ] Develop VariantsTable component
   - [ ] Add mobile-specific interactions

3. State Management:
   - [ ] Set up client-side filtering
   - [ ] Implement search functionality
   - [ ] Handle download progress updates
   - [ ] Manage expanded/collapsed states

4. Testing:
   - [ ] Unit tests for components
   - [ ] Integration tests for data flow
   - [ ] Mobile interaction testing
   - [ ] Accessibility testing 

@@ Phase 2: Download Integration @@

Phase 2: Detailed Design
----------------------

Component Integration
~~~~~~~~~~~~~~~~~~
.. code-block::

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

Mobile Layout
~~~~~~~~~~~
.. code-block::

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

User Flows
~~~~~~~~~

1. Download New Model
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

2. Monitor Downloads
   ```
   User clicks "Queue" button
   -> Opens queue overlay
   -> Shows active downloads
   -> Progress updates real-time
   -> Complete downloads appear in main table
   ```

3. Handle Failures
   ```
   Download fails
   -> Status shows as "Failed"
   -> Error message on hover
   -> Option to retry
   -> Clear from queue
   ```

Implementation Tasks
~~~~~~~~~~~~~~~~~

1. Download Dialog:
   - [ ] Create DownloadModelOverlay component
   - [ ] Implement URL/repo name input with validation
   - [ ] Add repository preview component
   - [ ] Create download confirmation flow
   - [ ] Add storage validation

2. Download Queue:
   - [ ] Create DownloadQueueOverlay component
   - [ ] Implement real-time progress updates
   - [ ] Add download management actions
   - [ ] Handle failed downloads

3. Integration:
   - [ ] Add download triggers to main UI
   - [ ] Connect WebSocket for progress
   - [ ] Update storage dashboard
   - [ ] Handle download completion

4. Mobile Support:
   - [ ] Optimize overlays for mobile
   - [ ] Add touch-friendly controls
   - [ ] Implement mobile progress view

5. Error Handling:
   - [ ] Validate storage requirements
   - [ ] Handle network failures
   - [ ] Show error messages
   - [ ] Implement retry logic

API Updates
~~~~~~~~~~
.. code-block:: typescript

    interface DownloadQueueItem {
      id: string;
      repo: string;
      filename: string;
      size: number;
      status: 'queued' | 'downloading' | 'failed';
      progress?: number;
      error?: string;
      started_at: string;
    }

    // WebSocket Events
    interface DownloadProgressEvent {
      id: string;
      progress: number;
      status: string;
      error?: string;
    }

    // API Endpoints
    GET /api/v1/downloads/queue
    Response: {
      active: DownloadQueueItem[];
      completed: DownloadQueueItem[];
    }

    POST /api/v1/downloads
    Request: {
      repo: string;
      filename: string;
    }

    DELETE /api/v1/downloads/{id}
    Response: 204

Testing Requirements
~~~~~~~~~~~~~~~~~
1. Functional Testing:
   - URL/repo name validation
   - Storage space validation
   - Download progress tracking
   - Error handling

2. Integration Testing:
   - WebSocket connections
   - Queue management
   - Storage updates
   - Table updates

3. Mobile Testing:
   - Touch interactions
   - Progress visibility
   - Overlay behavior

@@ After existing Phase 1 and 2 details @@

Additional Implementation Details
------------------------------

Model Details Overlay
~~~~~~~~~~~~~~~~~~~
.. code-block::

    ┌─ Main Table ─────────┐ ┌─ Info Overlay ──────────────┐
    │ [Table Content...]   │ │ Model: Llama-2-7B           │
    │                      │ ├───────────────────────────────
    │                      │ │ Metadata:                    │
    │                      │ │ Architecture: Llama          │
    │                      │ │ Base Model: Llama 2         │
    │                      │ │ ... (HF-style metadata)     │
    └──────────────────────┘ └───────────────────────────────

- Overlay slides in from right on desktop
- Full-screen overlay on mobile
- Dismissible by clicking outside
- Uses HuggingFace-style metadata display
- Direct API response rendering
- No metadata transformation needed

Filter Implementation
~~~~~~~~~~~~~~~~~~
.. code-block::

    Desktop:
    [Family: Llama, Phi ✕] [Size: S, M ✕] [Status: Active ✕]

    Mobile:
    ┌─ Filters ─────────────┐
    │ Family ▾             │
    │ ☐ Llama              │
    │ ☐ Phi                │
    │ [Apply]              │
    └──────────────────────┘

- Uses shadcn/ui components
- Multi-select chips on desktop
- Dropdown selects on mobile
- OR logic within categories
- AND logic between categories
- Persisted in localStorage
- State management via React state

Table Interactions
~~~~~~~~~~~~~~~~
1. Sorting:
   - Single column sort only
   - Sort by repo name
   - No multi-column sorting needed

2. Filtering:
   - Client-side filtering
   - Immediate updates
   - No server round-trips

3. Caching:
   - Uses React Query caching
   - Default stale time: 10 minutes
   - Background updates

Download Queue Updates
~~~~~~~~~~~~~~~~~~~
1. Progress Tracking:
   - 10-second polling when queue overlay open
   - No WebSocket needed
   - Simple GET request to queue endpoint

2. Error Handling:
   - Retry button resubmits download request
   - Delete button removes from queue
   - Error messages shown inline
   - No global notification system

Form Validation
~~~~~~~~~~~~~
1. URL/Repo Input:
   - React Hook Form validation
   - Immediate feedback
   - Error states shown inline
   - Support for:
     - Full HF URLs
     - Repo names (org/repo)
     - Direct file links

2. Storage Validation:
   - Simple available space check
   - No version conflict checking
   - Basic error messaging

Component Props
~~~~~~~~~~~~~
.. code-block:: typescript

    interface FilterState {
      families: string[];
      sizes: string[];
      statuses: string[];
    }

    interface FilterProps {
      state: FilterState;
      onChange: (newState: FilterState) => void;
      isMobile: boolean;
    }

    interface QueueOverlayProps {
      isOpen: boolean;
      onClose: () => void;
      pollingInterval: number;
    }

    interface InfoOverlayProps {
      repo: string;
      filename: string;
      isOpen: boolean;
      onClose: () => void;
      metadata: Record<string, unknown>;
    }

@@ Phase 3: Discovery Features @@

Phase 3: Detailed Design
----------------------

UI Layout - Featured Models
~~~~~~~~~~~~~~~~~~~~~~~
.. code-block::

    ┌──────────────────────────────────────────────────────────┐
    │ Featured Models                          [Dismiss] [→]   │
    ├──────────────────────────────────────────────────────────┤
    │ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐        │
    │ │ Gemma   │ │ Phi-3   │ │ Mixtral │ │ More... │        │
    │ │ #chat   │ │ #code   │ │ #chat   │ │         │        │
    │ │ 7B      │ │ 3B      │ │ 8x7B    │ │         │        │
    │ │         │ │         │ │         │ │         │        │
    │ │[Try Now]│ │[Try Now]│ │[Try Now]│ │         │        │
    │ └─────────┘ └─────────┘ └─────────┘ └─────────┘        │
    └──────────────────────────────────────────────────────────┘

Mobile Layout
~~~~~~~~~~~
.. code-block::

    ┌────────────────────────┐
    │ Featured Models     ⨯  │
    ├────────────────────────┤
    │ ┌──────────────────┐   │
    │ │ Gemma            │   │
    │ │ #chat #7B        │   │
    │ │ [Download Now]   │   │
    │ └──────────────────┘   │
    │ Swipe for more →       │
    └────────────────────────┘

API Structure
~~~~~~~~~~~
.. code-block:: typescript

    interface FeaturedModel {
      id: string;
      name: string;
      description: string;
      tags: string[];      // #chat, #code, etc
      size_category: string; // 7B, 3B etc
      repo: string;
      filename: string;    // Recommended variant
      metadata: {
        downloads: number;
        likes: number;
        model_type: string;
        family: string;
      };
      links: {
        huggingface: string;
        paper?: string;
      };
    }

    // API Endpoints
    GET /api/v1/featured-models
    Response: {
      models: FeaturedModel[];
      last_updated: string;
    }

    // Settings
    interface UserSettings {
      show_featured_models: boolean;
      // other settings...
    }

Component Hierarchy
~~~~~~~~~~~~~~~~
.. code-block::

    ModelFilesPage
    ├── FeaturedModelsSection (dismissible)
    │   ├── FeaturedModelCard
    │   │   ├── ModelBadges
    │   │   ├── QuickActions
    │   │   └── DownloadButton
    │   └── HorizontalScroller
    └── ExistingComponents...

User Flows
~~~~~~~~~

1. Quick Download
   ```
   User sees featured model
   -> Clicks "Try Now"
   -> System checks storage
   -> Initiates download
   -> Shows in download queue
   -> Updates status when complete
   ```

2. Learn More
   ```
   User interested in model
   -> Clicks model name/link
   -> Opens HuggingFace in new tab
   -> Explores detailed information
   ```

3. Dismiss Featured
   ```
   User wants to hide featured
   -> Clicks dismiss
   -> Section hides
   -> State persists until new models
   -> Can re-enable in settings
   ```

Implementation Tasks
~~~~~~~~~~~~~~~~~

1. Featured Models Component:
   - [ ] Create horizontal scrolling container
   - [ ] Implement featured model card
   - [ ] Add dismiss functionality
   - [ ] Handle mobile swipe gestures

2. Download Integration:
   - [ ] Add one-click download handler
   - [ ] Integrate with download queue
   - [ ] Show download status
   - [ ] Handle errors gracefully

3. Settings Integration:
   - [ ] Add featured models toggle
   - [ ] Persist user preference
   - [ ] Handle new models notification

4. Mobile Optimization:
   - [ ] Implement touch-friendly scrolling
   - [ ] Optimize card layout
   - [ ] Add swipe indicators

Testing Requirements
~~~~~~~~~~~~~~~~~
1. Functional Testing:
   - Featured models display
   - Horizontal scroll behavior
   - Download integration
   - Dismiss functionality

2. Mobile Testing:
   - Touch scrolling
   - Swipe gestures
   - Card layout
   - Download interaction

3. Integration Testing:
   - Settings persistence
   - Download queue integration
   - Status updates

Component Props
~~~~~~~~~~~~
.. code-block:: typescript

    interface FeaturedModelsProps {
      models: FeaturedModel[];
      onDismiss: () => void;
      isDismissed: boolean;
    }

    interface FeaturedModelCardProps {
      model: FeaturedModel;
      isDownloaded: boolean;
      onDownload: () => void;
      onLearnMore: () => void;
    }

    interface HorizontalScrollerProps {
      children: React.ReactNode;
      showScrollButtons: boolean;
    }

@@ Add Onboarding Section @@

User Onboarding Design
--------------------

First Visit Experience
~~~~~~~~~~~~~~~~~~~
.. code-block::

    ┌──────────────────────────────────────────────────────────┐
    │ Welcome to Model Management! [Dismiss] [Take a Tour]     │
    ├──────────────────────────────────────────────────────────┤
    │ Discover, download, and manage your AI models in one     │
    │ place. Get started with trending models or manage your   │
    │ existing collection.                                     │
    └──────────────────────────────────────────────────────────┘

Feature Spotlights
~~~~~~~~~~~~~~~~
Sequential tooltips that highlight key features:

1. Featured Models Spotlight
   ```
   ┌─ Tooltip ──────────────┐
   │ Trending Models        │
   │ Discover and try new   │
   │ models with one click  │
   │ [1/4] [Skip] [Next →] │
   └──────────────────────┘
      ↓
   [Featured Models Section]
   ```

2. Download Spotlight
   ```
   ┌─ Tooltip ────────────┐
   │ Download Models      │
   │ Get any model from   │
   │ HuggingFace easily   │
   │ [2/4] [Skip] [Next] │
   └────────────────────┘
      ↓
   [Download Model Button]
   ```

3. Storage Dashboard
   ```
   ┌─ Tooltip ──────────┐
   │ Storage Overview   │
   │ Monitor your space │
   │ and model count    │
   │ [3/4] [Skip][Next]│
   └──────────────────┘
      ↓
   [Storage Dashboard]
   ```

4. Filter & Search
   ```
   ┌─ Tooltip ──────────┐
   │ Find Models        │
   │ Filter and search  │
   │ your collection    │
   │ [4/4] [Finish]    │
   └──────────────────┘
      ↓
   [Filter Bar]
   ```

Implementation Details
~~~~~~~~~~~~~~~~~~~
.. code-block:: typescript

    interface OnboardingState {
      hasSeenTour: boolean;
      currentStep: number;
      isDismissed: boolean;
    }

    interface SpotlightProps {
      step: number;
      title: string;
      description: string;
      position: 'top' | 'bottom' | 'left' | 'right';
      onNext: () => void;
      onSkip: () => void;
      totalSteps: number;
    }

User Preferences
~~~~~~~~~~~~~~
- Store onboarding state in localStorage
- Allow reset via Settings page
- Persist dismissal state

Mobile Considerations
~~~~~~~~~~~~~~~~~~
- Full-width tooltips
- Swipeable tour steps
- Tap anywhere to dismiss
- Automatic positioning

Implementation Tasks
~~~~~~~~~~~~~~~~~
1. Onboarding Components:
   - [ ] Create Welcome banner component
   - [ ] Implement Spotlight component
   - [ ] Add step navigation
   - [ ] Handle dismissal state

2. State Management:
   - [ ] Add onboarding state storage
   - [ ] Implement tour progression
   - [ ] Handle interruptions

3. Mobile Support:
   - [ ] Add touch interactions
   - [ ] Optimize tooltip placement
   - [ ] Implement swipe navigation

4. Testing:
   - [ ] Test tour flow
   - [ ] Verify state persistence
   - [ ] Check mobile interactions

@@ Add Implementation Progress @@

Task Completion Summary
---------------------

UI Improvements
~~~~~~~~~~~~~
1. ✓ Enhanced Table Layout
   - Implemented responsive column visibility
   - Added truncation for long repo names
   - Right-aligned size column
   - Optimized mobile view with stacked info

2. ✓ Mobile Optimization
   - Hidden filename column on mobile
   - Added filename under repo for mobile
   - Adjusted button sizes for touch
   - Centered pagination controls

3. ✓ User Onboarding
   - Added dismissable welcome banner
   - Stored banner state in localStorage
   - Added clear onboarding message
   - Implemented dismiss functionality

4. ✓ Navigation Elements
   - Added HuggingFace repository links
   - Implemented external link icons
   - Added tooltips for actions
   - Optimized button spacing

5. ✓ Pagination Improvements
   - Simplified page number display
   - Centered controls on mobile
   - Added responsive spacing
   - Optimized button sizes

Technical Implementation
~~~~~~~~~~~~~~~~~~~~~
1. ✓ Table Component
   ```typescript
   const columns = [
     {
       id: 'repo',
       name: 'Repo',
       sorted: true,
       className: 'max-w-[180px] truncate',
     },
     // ... other columns
   ];
   ```

2. ✓ Mobile Layout
   ```typescript
   <TableCell className="max-w-[180px]">
     <div className="truncate">{modelFile.repo}</div>
     <div className="text-xs text-muted-foreground truncate sm:hidden mt-1">
       {modelFile.filename}
     </div>
   </TableCell>
   ```

3. ✓ Banner Storage
   ```typescript
   const [hasDismissedBanner, setHasDismissedBanner] = useLocalStorage(
     'modelfiles-banner-dismissed',
     false
   );
   ```

4. ✓ External Links
   ```typescript
   const getHuggingFaceUrl = (repo: string) => {
     return `https://huggingface.co/${repo}`;
   };
   ```

5. ✓ Pagination Component
   ```typescript
   <div className="flex justify-center gap-4">
     <Button size="sm" className="px-6">Previous</Button>
     <span className="flex items-center">{page}/{totalPages}</span>
     <Button size="sm" className="px-6">Next</Button>
   </div>
   ```

Code Organization
~~~~~~~~~~~~~~
1. ✓ Component Structure
   - Separated table configuration
   - Isolated pagination logic
   - Modular banner component
   - Reusable helper functions

2. ✓ Style Management
   - Consistent class naming
   - Responsive design classes
   - Mobile-first approach
   - Utility class optimization

3. ✓ State Management
   - Local storage integration
   - Pagination state
   - Sort state handling
   - Banner visibility control

Testing Considerations
~~~~~~~~~~~~~~~~~~~
1. ✓ Component Testing
   - Table rendering
   - Mobile responsiveness
   - Banner persistence
   - Link functionality

2. ✓ User Interactions
   - Sort functionality
   - Pagination controls
   - Banner dismissal
   - External links

3. ✓ Mobile Testing
   - Touch targets
   - Responsive layout
   - Content visibility
   - Navigation usability

Future Improvements
~~~~~~~~~~~~~~~~
1. Performance Optimization
   - Virtual scrolling for large lists
   - Optimized sorting
   - Cached external links
   - Lazy loading improvements

2. Enhanced Features
   - Batch actions
   - Advanced filtering
   - Search functionality
   - More metadata display

3. Accessibility
   - Keyboard navigation
   - Screen reader support
   - Focus management
   - ARIA attributes
