=============================
Model Alias Page UI/UX Revamp
=============================

User Story
---------
As a Bodhi App User
I want an intuitive interface for managing and testing model configurations
So that I can efficiently create, modify, and validate model aliases for my use cases

Background
---------
- Current interface is basic with limited user guidance
- Parameter configuration requires deep technical knowledge
- No built-in way to test configurations
- Limited organization and discovery features
- No usage analytics or metrics

Acceptance Criteria
-----------------

Phase 1: Enhanced List View
~~~~~~~~~~~~~~~~~~~~~~~~~

Backend Changes
^^^^^^^^^^^^^
1. Usage Metrics API:
   .. code-block:: typescript

    interface AliasMetrics {
      alias: string;
      usage_count: number;
      last_used: string;
      success_rate: number;
    }

2. Enhanced Alias List API:
   .. code-block:: typescript

    interface EnhancedAlias {
      alias: string;
      source: 'user' | 'model';
      repo: string;
      filename: string;
      metrics: AliasMetrics;
      model_family?: string;
      quantization?: string;
    }

Frontend Changes
^^^^^^^^^^^^^^
1. Grouped List View:
   - Group by model family
   - Visual distinction between user/model aliases
   - Quick actions based on alias type
   - Usage statistics display

2. Enhanced Search & Filter:
   - Filter by model family
   - Filter by source type
   - Search by alias name
   - Sort by usage/last used

3. List Item Components:
   - Status indicators
   - Quick action buttons
   - Usage metrics
   - Expandable details

Phase 2: Configuration Management
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Backend Changes
^^^^^^^^^^^^^
1. Parameter Validation API:
   - Model-specific range validation
   - Parameter conflict detection
   - Detailed error messages

2. Temporary Alias Support:
   - Create temporary configurations
   - Auto-cleanup unused temps
   - Convert temp to permanent

Frontend Changes
^^^^^^^^^^^^^^
1. Enhanced Form Interface:
   - Grouped parameter sections
   - Parameter descriptions
   - Visual range indicators
   - Conflict warnings

2. Quick Actions:
   - Clone existing alias
   - Create from model
   - Import/Export configs
   - Delete with confirmation

3. Validation Feedback:
   - Real-time field validation
   - Error message display
   - Range visualization
   - Conflict indicators

Phase 3: Configuration Playground
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Backend Changes
^^^^^^^^^^^^^
1. Test Configuration API:
   - Use existing chat endpoints
   - Support temporary aliases
   - Parameter validation

Frontend Changes
^^^^^^^^^^^^^^
1. Playground Interface:
   - Split view design
   - Parameter controls
   - Test input area
   - Response preview

2. Parameter Testing:
   - Real-time adjustments
   - Save/reset options
   - Compare configurations
   - Performance feedback

3. Template Testing:
   - Template input area
   - Variable substitution
   - Format validation
   - Preview rendering

Technical Implementation
----------------------

API Structure
~~~~~~~~~~~
.. code-block:: typescript

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

Component Structure
~~~~~~~~~~~~~~~~
1. List View Components:
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

2. Configuration Components:
   ```
   AliasForm
   ├── ParameterGroups
   │   ├── GenerationControl
   │   └── PerformanceSettings
   ├── ValidationDisplay
   └── ActionButtons
   ```

3. Playground Components:
   ```
   ConfigPlayground
   ├── ParameterPanel
   ├── TestInput
   ├── ResponsePreview
   └── ActionButtons
   ```

Testing Requirements
------------------
1. Component Testing:
   - Filter functionality
   - Group collapsing
   - Action handlers
   - Form validation

2. Integration Testing:
   - API integration
   - Metric updates
   - Configuration flow
   - Playground interaction

3. Validation Testing:
   - Parameter ranges
   - Conflict detection
   - Error handling
   - Form submission

Mobile Considerations
------------------
1. List View:
   - Collapsible groups
   - Touch-friendly actions
   - Simplified metrics
   - Swipe actions

2. Configuration:
   - Full-screen editors
   - Stepped form flow
   - Touch-optimized controls
   - Keyboard handling

3. Playground:
   - Tab-based navigation
   - Responsive split view
   - Touch-friendly controls
   - Mobile-first design

Not In Scope
-----------
- Backup/restore functionality
- Historical configuration tracking
- Advanced analytics
- Batch operations
- Chat interface integration

Dependencies
-----------
- Backend validation API
- Metrics tracking system
- Template processing
- Parameter validation

Migration Strategy
---------------
- Progressive enhancement
- No data migration needed
- Feature flag for new UI
- Parallel old/new views

Future Considerations
------------------
1. Enhanced Features:
   - Configuration templates
   - Preset libraries
   - Advanced analytics
   - Batch operations

2. Integration:
   - Chat interface connection
   - Model performance metrics
   - Usage recommendations
   - Community sharing

3. Analytics:
   - Usage patterns
   - Performance tracking
   - Error analysis
   - Optimization suggestions

@@ Phase 1 Detailed Design @@

Phase 1: Detailed Design
----------------------

UI Layout - Desktop
~~~~~~~~~~~~~~~~~
.. code-block::

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

UI Layout - Mobile
~~~~~~~~~~~~~~~~
.. code-block::

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

@@ Phase 2 Detailed Design @@

Phase 2: Detailed Design
----------------------

Configuration Form - Desktop
~~~~~~~~~~~~~~~~~~~~~~~~~
.. code-block::

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

@@ Phase 3 Detailed Design @@

Phase 3: Detailed Design
----------------------

Playground Interface - Desktop
~~~~~~~~~~~~~~~~~~~~~~~~~~
.. code-block::

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
    │               │ │ <system>You are a helpful...</system>│ │
    │ [Send]        │ │ <user>{{message}}</user>           │ │
    └───────────────┴──────────────────────────────────────────┘

Mobile Playground
~~~~~~~~~~~~~~
.. code-block::

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

Parameter Control - Mobile
~~~~~~~~~~~~~~~~~~~~~~~
.. code-block::

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

@@ Add Onboarding Section @@

User Onboarding Design
--------------------

First Visit Experience
~~~~~~~~~~~~~~~~~~~
.. code-block::

    ┌──────────────────────────────────────────────────────────┐
    │ Welcome to Model Configurations!     [Dismiss] [Tour]    │
    ├──────────────────────────────────────────────────────────┤
    │ Customize and optimize your AI models with configuration │
    │ profiles. Start with model defaults or create your own   │
    │ configurations for specific use cases.                   │
    └──────────────────────────────────────────────────────────┘

Feature Spotlights
~~~~~~~~~~~~~~~~
Sequential tooltips highlighting key features:

1. Model Alias Overview
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

2. Configuration Types
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

3. Parameter Groups
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

4. Configuration Testing
   ```
   ┌─ Tooltip ────────────────┐
   │ Test Your Settings      │
   │ Try configurations      │
   │ before saving them      │
   │ [4/5] [Skip] [Next →]   │
   └────────────────────────┘
      ↓
   [Test Button]
   ```

5. Quick Actions
   ```
   ┌─ Tooltip ────────────────┐
   │ Quick Actions           │
   │ Clone, edit, or create  │
   │ new configurations      │
   │ [5/5] [Finish]         │
   └────────────────────────┘
      ↓
   [Action Buttons]
   ```

Implementation Details
~~~~~~~~~~~~~~~~~~~
.. code-block:: typescript

    interface OnboardingState {
      hasSeenTour: boolean;
      currentStep: number;
      isDismissed: boolean;
      lastSeenVersion: string;
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

Parameter Help System
~~~~~~~~~~~~~~~~~~
1. Inline Parameter Help
   ```
   ┌─ Parameter Help ──────────────────────┐
   │ Temperature                           │
   │                                       │
   │ What it does:                         │
   │ Controls response randomness          │
   │                                       │
   │ Recommended settings:                 │
   │ • Creative: 0.7-0.9                  │
   │ • Factual: 0.1-0.3                   │
   │                                       │
   │ Tips:                                │
   │ • Higher = more creative             │
   │ • Lower = more focused               │
   │                                       │
   │ [Learn More] [See Examples]          │
   └───────────────────────────────────────┘
   ```

2. Quick Tips
   ```
   ┌─ Quick Tip ─────────────┐
   │ 💡 Try lowering the     │
   │ temperature for more    │
   │ consistent responses    │
   └─────────────────────────┘
   ```

User Preferences
~~~~~~~~~~~~~~
- Store onboarding progress in localStorage
- Remember dismissed help topics
- Track feature usage for personalized tips
- Allow tour reset in settings

Mobile Considerations
~~~~~~~~~~~~~~~~~~
1. Tour Adaptations
   - Full-screen welcome
   - Bottom sheet tooltips
   - Swipeable tour steps
   - Progress indicator

2. Help System
   - Collapsible help panels
   - Touch-friendly tooltips
   - Quick access help button
   - Context-sensitive hints

Implementation Tasks
~~~~~~~~~~~~~~~~~
1. Core Components:
   - [ ] Welcome modal component
   - [ ] Feature spotlight system
   - [ ] Parameter help panels
   - [ ] Quick tips display

2. State Management:
   - [ ] Onboarding progress tracking
   - [ ] Help topic preferences
   - [ ] Usage analytics
   - [ ] Tour interruption handling

3. Help Content:
   - [ ] Parameter descriptions
   - [ ] Usage examples
   - [ ] Best practices
   - [ ] Common pitfalls

4. Mobile Support:
   - [ ] Responsive layouts
   - [ ] Touch interactions
   - [ ] Gesture navigation
   - [ ] Compact help display

5. Testing:
   - [ ] Tour progression
   - [ ] Help system usability
   - [ ] Mobile interactions
   - [ ] State persistence

@@ Add Missing Requirements @@

Additional Requirements
--------------------

Chat Template Integration
~~~~~~~~~~~~~~~~~~~~~~~
1. Template Selection:
   - Built-in template list
   - Custom template support
   - Template preview
   - Format validation

2. Template Management:
   ```
   ┌─────────────────────────────────────┐
   │ Chat Template                    [↓] │
   ├─────────────────────────────────────┤
   │ • Built-in                          │
   │   ├── Llama2                        │
   │   ├── Phi3                          │
   │   └── Gemma                         │
   │ • Custom                            │
   │   └── [Import from HuggingFace]     │
   └─────────────────────────────────────┘
   ```

Parameter Validation
~~~~~~~~~~~~~~~~~
1. Model-Specific Validation:
   - Range validation from backend
   - Conflict detection
   - Default value handling
   - Error message display

2. Validation UI:
   ```
   ┌─────────────────────────────────────┐
   │ ⚠️ Parameter Conflicts              │
   ├─────────────────────────────────────┤
   │ • Temperature and Top-p both set    │
   │ • Context size exceeds model limit  │
   │                                     │
   │ [Show Details] [Quick Fix]          │
   └─────────────────────────────────────┘
   ```

Error Handling
~~~~~~~~~~~~
1. API Errors:
   - Connection issues
   - Validation failures
   - Missing model files
   - Permission errors

2. User Feedback:
   - Error message display
   - Recovery suggestions
   - Fallback options
   - Auto-retry logic

Implementation Tasks
------------------

1. Template Integration:
   - [ ] Template selector component
   - [ ] Template preview system
   - [ ] Custom template import
   - [ ] Format validation

2. Enhanced Validation:
   - [ ] Model-specific validation
   - [ ] Real-time parameter checking
   - [ ] Conflict detection
   - [ ] Error message system

3. Error Management:
   - [ ] Error boundary components
   - [ ] Recovery mechanisms
   - [ ] Retry logic
   - [ ] User feedback system

4. Analytics Integration:
   - [ ] Usage tracking
   - [ ] Error tracking
   - [ ] Performance monitoring
   - [ ] User behavior analysis

Performance Considerations
-----------------------

1. Data Loading:
   - Lazy load parameter details
   - Cache template data
   - Progressive loading
   - Background validation

2. State Management:
   - Efficient form updates
   - Minimal re-renders
   - Optimized validation
   - Smart caching

3. Mobile Optimization:
   - Reduced network calls
   - Optimized animations
   - Touch interaction delay
   - Memory management

Accessibility Requirements
-----------------------

1. Keyboard Navigation:
   - Focus management
   - Shortcut keys
   - Tab order
   - ARIA labels

2. Screen Readers:
   - Meaningful descriptions
   - Status announcements
   - Error notifications
   - Help text access

3. Visual Accessibility:
   - Color contrast
   - Text scaling
   - Focus indicators
   - Icon labels

Documentation Requirements
-----------------------

1. User Documentation:
   - Parameter descriptions
   - Configuration guides
   - Best practices
   - Troubleshooting

2. Technical Documentation:
   - API integration
   - Component usage
   - State management
   - Error handling

3. Maintenance Guides:
   - Update procedures
   - Migration steps
   - Backup processes
   - Recovery procedures

Success Metrics
-------------

1. Usage Metrics:
   - Configuration creation rate
   - Parameter adjustment frequency
   - Template usage patterns
   - Error rates

2. User Experience:
   - Time to create config
   - Error recovery time
   - Help usage patterns
   - Tour completion rate

3. Performance Metrics:
   - Load times
   - Response times
   - Validation speed
   - Error resolution time


Mobile Optimization:
Hide less important columns on mobile (Source)
Show Source under the alias name for mobile
Add truncation for long alias names
Right-align any numeric/size columns
Optimize table header text and spacing
UI Improvements:
1. Add welcome banner with localStorage persistence
Add HuggingFace link for models from HF
Improve pagination layout and mobile view
Optimize button sizes and spacing
Add tooltips for actions
Questions before proceeding:
Should we keep the same banner text or customize it for Model Aliases?
Are there any model-specific actions we should add besides HuggingFace link?
Should we show any additional metadata in the mobile view?
Do we want to keep the same page size (30) as ModelFiles?
Are there any specific sorting preferences we should set as default?
Should we add any filtering capabilities in this phase?