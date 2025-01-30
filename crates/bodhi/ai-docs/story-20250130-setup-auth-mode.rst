Setup Wizard: Authentication Mode Selection
=========================================

User Story
---------
As a new Bodhi App user,
I want to understand and choose between authentication modes,
So that I can set up the app according to my needs.

Background
----------
- Users need to choose between authenticated and non-authenticated modes
- This decision is permanent and affects feature availability
- Authenticated mode is recommended for most users
- Choice impacts multi-user capabilities and security features

Acceptance Criteria
-----------------

Content Requirements
~~~~~~~~~~~~~~~~~~
- Clear presentation of both authentication modes
- Recommended mode (authenticated) highlighted
- Feature availability comparison
- Permanent decision warning
- Internet requirement notice
- Admin role explanation
- Link to learn more

UI/UX Requirements
~~~~~~~~~~~~~~~~
- Primary styling for authenticated mode
- Muted styling for non-authenticated mode
- Clear visual hierarchy
- Responsive design
- Loading states for selection
- Smooth transitions
- Progress indicator (step 2/5)

Technical Implementation
~~~~~~~~~~~~~~~~~~~~~~
- Update setup wizard container
- Implement mode selection handling
- Add status persistence
- Handle navigation flow
- Error state management
- Loading state handling

Navigation Logic
~~~~~~~~~~~~~~
- Forward to resource admin setup (auth mode)
- Forward to environment setup (non-auth mode)
- Handle back navigation
- Prevent direct URL access

Content Structure
---------------

Layout Options
~~~~~~~~~~~~~

Responsive Layout
~~~~~~~~~~~~~~~

Desktop Layout (>768px)
^^^^^^^^^^^^^^^^^^^^^
.. code-block:: text

    ┌─────────────────────────────┐
    │     Setup Progress (2/5)    │
    ├─────────────────────────────┤
    │    Choose Setup Mode        │
    ├─────────────────────────────┤
    │ ┌─────────────┐ ┌────────┐ │
    │ │ Recommended │ │        │ │
    │ │   Auth      │ │ Basic  │ │
    │ │   Mode      │ │ Mode   │ │
    │ └─────────────┘ └────────┘ │
    ├─────────────────────────────┤
    │      Decision Warning       │
    └─────────────────────────────┘

Mobile Layout (<768px)
^^^^^^^^^^^^^^^^^^^^
.. code-block:: text

    ┌────────────────────┐
    │  Progress (2/5)    │
    ├────────────────────┤
    │  Choose Mode       │
    ├────────────────────┤
    │ ┌────────────────┐ │
    │ │  Recommended   │ │
    │ │   Auth Mode    │ │
    │ └────────────────┘ │
    │ ┌────────────────┐ │
    │ │  Basic Mode    │ │
    │ └────────────────┘ │
    ├────────────────────┤
    │ Warning Message    │
    ├────────────────────┤
    │ [Auth Button]      │
    │ [Basic Button]     │
    └────────────────────┘

Mobile Considerations
^^^^^^^^^^^^^^^^^^
- Stack mode cards vertically
- Full-width buttons
- Simplified comparison table view
- Touch-friendly tap targets
- Collapsible feature details
- Reduced padding (16px)
- Adjusted typography scale
- Swipe gestures support


Option 2: Comparison Table
~~~~~~~~~~~~~~~~~~~~~~~~~
.. code-block:: text

    Feature          │ Auth Mode │ Basic Mode
    ────────────────┼───────────┼───────────
    Multi-User      │    ✓      │    ✗
    API Tokens      │    ✓      │    ✗
    User Mgmt       │    ✓      │    ✗
    Monitoring      │    ✓      │    ✗
    Local Access    │    ✓      │    ✓
    No Login Req.   │    ✗      │    ✓


Content Sections
~~~~~~~~~~~~~~

Header
^^^^^^
.. code-block:: text

    Choose Your Setup Mode
    Select how you want to use Bodhi App

Introduction
^^^^^^^^^^^
.. code-block:: text

    This choice determines how you'll access and manage your Bodhi App.
    This is a permanent setting and cannot be changed later without
    losing your data and settings.

Recommended Mode
^^^^^^^^^^^^^^
.. code-block:: text

    🔐 Authenticated Mode (Recommended)
    
    - Secure access with email login
    - Multiple users with controlled access
    - API tokens for external apps
    - Usage monitoring and management
    - Future features automatically available
    
    Perfect for teams and security-conscious users
    Requires internet connection for login

Basic Mode
^^^^^^^^^
.. code-block:: text

    🚪 Non-Authenticated Mode
    
    - Direct access without login
    - Single user setup
    - Basic features only
    - Limited to local access
    
    Suitable for personal use and testing

Warning Message
^^^^^^^^^^^^^
.. code-block:: text

    ⚠️ Important: This choice is permanent
    Changing modes later will require a fresh setup and data loss

Call to Action
^^^^^^^^^^^^
.. code-block:: text

    [Set up with Authentication] (Primary Button)
    [Continue without Authentication] (Muted Button)
    
    Learn more about setup modes →

Technical Details
---------------

Component Structure
~~~~~~~~~~~~~~~~~
.. code-block:: typescript

    interface AuthModeSelectionProps {
      onSelect: (authMode: boolean) => void;
      isLoading: boolean;
    }

State Management
~~~~~~~~~~~~~~
.. code-block:: typescript

    interface SetupState {
      authMode: boolean | null;
      step: number;
    }

Testing Criteria
--------------

Functional Tests
~~~~~~~~~~~~~~
- Mode selection handling
- Navigation flow
- State persistence
- Loading states
- Error handling

Visual Tests
~~~~~~~~~~
- Responsive design
- Style consistency
- Animation smoothness
- Loading indicators

Accessibility Tests
~~~~~~~~~~~~~~~~~
- Keyboard navigation
- Screen reader compatibility
- Focus management
- Color contrast

Out of Scope
-----------
- OAuth2 implementation details
- Resource admin setup flow
- User management interface
- Token management system
- Future feature specifics

Dependencies
----------
- Setup wizard container
- App state management
- Navigation system
- UI component library 