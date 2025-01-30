
# Setup Wizard: Welcome to Bodhi

## User Story
As a new user,
I want to understand what Bodhi App offers and how to set it up,
So that I can make informed decisions during the setup process.

## Background
- First screen of setup wizard
- Sets tone for user experience
- Introduces key features and benefits
- Prepares user for setup decisions

## Acceptance Criteria

### Content Requirements
- [x] Clear value proposition
- [x] Key features overview
- [x] Setup process outline
- [x] Progress indicator (1/5)
- [x] Next step preview

### UI/UX Requirements
- [x] Responsive layout
- [x] Loading states
- [x] Progress tracking
- [x] Smooth animations
- [x] Dark mode support

### Technical Implementation
- [x] State persistence via /app/setup endpoint
- [x] URL-based navigation (/ui/setup/*)
- [x] Tailwind responsive design
- [x] ShadcnUI components
- [x] Framer Motion animations

## Content Structure

### Layout

#### Desktop/Tablet Layout (>768px)
```
┌─────────────────────────────┐
│     Progress (1/5)          │
├─────────────────────────────┤
│        Logo                 │
├─────────────────────────────┤
│     Welcome Message         │
├─────────────────────────────┤
│     Features/Benefits       │
│     [Expandable Cards]      │
├─────────────────────────────┤
│     Setup Preview          │
├─────────────────────────────┤
│     Continue Button         │
└─────────────────────────────┘
```

#### Mobile Layout (<768px)
```
┌────────────────────┐
│    Progress (1/5)  │
├────────────────────┤
│       Logo         │
├────────────────────┤
│  Welcome Message   │
├────────────────────┤
│ Features/Benefits  │
│ [Collapsed Cards]  │
├────────────────────┤
│   Setup Preview    │
├────────────────────┤
│ Continue Button    │
└────────────────────┘
```

## Technical Details

### Component Structure
```typescript
interface SetupProgressProps {
  currentStep: number;
  totalSteps: number;
  steps: Array<{
    id: string;
    label: string;
    status: 'completed' | 'current' | 'upcoming';
  }>;
}

interface SetupRequest {
  authz: boolean;
}

interface AppInfo {
  status: string;
}
```

### State Management
- Backend state persistence via useSetupApp hook
- No local state persistence required
- URL-based navigation state

### Styling Framework
- Tailwind CSS for responsive design
- Default breakpoints:
  - sm: 640px
  - md: 768px
  - lg: 1024px
  - xl: 1280px
  - 2xl: 1536px
- Violet theme color scheme
- Dark mode support

### Assets
- Logo: /bodhi-logo/*.svg (60px, 120px, 240px, 480px)
- Icons: Lucide React
- No additional illustrations required

### Animations
- Framer Motion integration
- Page load animations
- Card hover effects
- Progress indicator transitions
- Expandable section animations

## Testing Criteria

### Functional Tests
- Progress indicator display
- Navigation flow
- State management
- Dark mode toggle
- Responsive layouts

### Visual Tests
- Animation smoothness
- Layout consistency
- Loading states
- Card interactions
- Theme compliance

### Accessibility Tests
- Screen reader compatibility
- Keyboard navigation
- Focus management
- ARIA best practices

## Out of Scope
- Back navigation
- Keyboard shortcuts
- Custom illustrations
- Reading time tracking
- Performance metrics

## Dependencies
- ShadcnUI components
- Framer Motion
- Lucide React icons
- useSetupApp hook
- AppInitializer component
- Loading component

## Success Metrics
- User proceeds to next step
- Setup state properly saved
- No navigation/routing errors
- Consistent rendering across devices

## Page Content Structure

### Layout

### Responsive Layout
```
Desktop (>768px):
┌──────────────────────────┐
│      Progress Bar        │
├──────────────────────────┤
│         Logo             │
├──────────────────────────┤
│        Headline          │
├──────────────────────────┤
│      Introduction        │
├──────────────────────────┤
│    Benefits Section      │
│    (2x2 Grid Cards)      │
├──────────────────────────┤
│     Next Button          │
└──────────────────────────┘

Mobile (<768px):
┌──────────────────────┐
│    Progress Bar      │
├──────────────────────┤
│       Logo           │
├──────────────────────┤
│      Headline        │
├──────────────────────┤
│    Introduction      │
├──────────────────────┤
│ Benefits (Stacked)   │
│ ┌────────────────┐   │
│ │    Card 1      │   │
│ └────────────────┘   │
│ ┌────────────────┐   │
│ │    Card 2      │   │
│ └────────────────┘   │
│ ┌────────────────┐   │
│ │    Card 3      │   │
│ └────────────────┘   │
│ ┌────────────────┐   │
│ │    Card 4      │   │
│ └────────────────┘   │
├──────────────────────┤
│    Next Button       │
└──────────────────────┘
```

### Mobile Considerations
- Stack benefit cards vertically
- Reduce horizontal padding (16px)
- Adjust typography sizes:
  - Headline: 24px
  - Subheadline: 18px
  - Body: 16px
- Full-width buttons
- Collapsible sections if needed
- Touch-friendly tap targets (min 44px)

### Content Sections

#### Header
```
Welcome to Bodhi App
Run AI Models Locally, Privately, and Free
```

#### Our Vision
```
The word 'Bodhi' comes from Sanskrit, meaning intelligence or enlightenment. 
We believe artificial intelligence should be accessible to everyone, not just 
those who can afford expensive subscription or have the technical expertise to 
set up complex systems.

Bodhi App is our step towards democratizing AI - bringing powerful language 
models to your device, making AI accessible to underserved communities 
worldwide. By running everything locally and freely, we're working to ensure 
no one is left behind in the AI revolution.
```

#### Introduction
```
Bodhi App brings the power of Large Language Models (LLMs) directly to your device. 
Experience AI without compromising your privacy or paying for API calls.

We'll guide you through a simple setup process to get your personal AI assistant 
up and running on your machine in minutes.
```

#### Key Benefits (Card Grid with Icons)
```
🔒 Complete Privacy
Your data stays on your device. No cloud processing, no data sharing, 
just secure local inference.

💰 Always Free
Run unlimited AI inferences without usage fees or API costs. 
Your hardware, your rules.

🚀 Full Control
Choose and run any compatible LLM model. Customize settings and 
optimize for your needs.

⚡ Local Performance
Direct access to your hardware's capabilities for optimal performance 
without internet latency.
```

#### Setup Journey Preview
```
Quick Setup Steps:
1. Choose your authentication mode
2. Configure your environment
3. Get your first AI model
4. Start chatting!

Time to complete: ~5 minutes
```

#### Call to Action
```
Primary Button: "Begin Setup →"
Secondary Text: "Just a few steps to your personal AI assistant"
```

### Visual Elements
- Bodhi logo at top (SVG)
- Progress indicator showing step 1/5
- Benefit cards with subtle hover effects
- Loading states for status checks
- Smooth transitions between sections

### Typography
- Headline: Large, bold (32px)
- Subheadline: Medium, regular (20px)
- Body text: Regular (16px)
- Benefits headings: Medium (18px)
- Progress text: Small (14px)

### Spacing
- Comfortable vertical spacing between sections (32px)
- Card grid with consistent gaps (24px)
- Content container max-width (800px)
- Responsive padding on mobile (16px)

### Animations
- Subtle fade-in on page load
- Smooth hover states on cards
- Button hover/active states
- Progress indicator emphasis 
- Simple step indicators with icons
- Subtle progress visualization
- Card hover effects with gentle elevation

---Old Story---

# Setup Wizard: Bodhi App Introduction Screen

## User Story
As a new user running Bodhi App for the first time,  
I want to understand what Bodhi App is and its key benefits  
So that I can make informed decisions during the setup process.

## Background
- Users download Bodhi App to run LLMs locally on their device
- First-time setup requires clear explanation of app's purpose and benefits
- Users need to understand privacy and cost benefits before proceeding
- This is the first screen of the multi-stage setup wizard

## Acceptance Criteria

### Content Requirements
- [x] Clear headline introducing Bodhi App
- [x] Concise explanation of local LLM inference
- [x] Key benefits highlighted:
  - Privacy: "All inference runs locally - your data never leaves your device"
  - Cost: "Completely free and unmetered usage"
  - Control: "Run AI models on your own hardware"
- [x] Setup wizard progress indicator showing first step
- [x] Clear "Next" button to proceed with setup

### UI/UX Requirements
- [x] Clean, focused layout with Bodhi logo
- [x] Benefits presented in easily scannable format
- [x] Consistent styling with app's design system
- [x] Responsive design for different screen sizes
- [x] Appropriate spacing and typography hierarchy
- [x] Loading state handled while checking app status

### Technical Implementation
- [x] Create new setup wizard container component
- [x] Implement wizard progress tracking
- [x] Add app status check using `useAppInfo` hook
- [x] Handle navigation to next setup stage
- [x] Persist setup progress in app state

### Navigation Logic
- [x] Redirect to this screen if app status is 'setup'
- [x] Enable "Next" button only after content is loaded
- [x] Save progress when proceeding to next step
- [x] Handle browser refresh/reload scenarios

## Technical Details

### Component Structure
```typescript
interface SetupWizardProps {
  currentStep: number;
  totalSteps: number;
  children: React.ReactNode;
}

interface IntroScreenProps {
  onNext: () => void;
  isLoading: boolean;
}
```

### State Management
```typescript
interface SetupState {
  currentStep: number;
  completed: string[];
  // ... other setup state
}
```

### Route Protection
- Ensure setup flow can't be bypassed
- Handle direct URL access appropriately
- Maintain setup state consistency

## Out of Scope
- Authentication mode selection (next step)
- System compatibility checks
- Model recommendations
- Detailed feature explanations
- Community/social links

## Testing Criteria

### Unit Tests
- [x] Test component rendering
- [x] Verify content accuracy
- [x] Test loading states
- [x] Validate navigation logic

### Integration Tests
- [x] Test setup flow state management
- [x] Verify routing logic
- [x] Test progress persistence
- [x] Validate app status checks

### Accessibility Tests
- [x] Proper heading hierarchy
- [x] Keyboard navigation
- [x] Screen reader compatibility
- [x] Sufficient color contrast

## Design Assets
- Bodhi logo (SVG)
- Progress indicator component
- Typography styles from design system
- Spacing/layout constants

## Success Metrics
- User proceeds to next step
- Setup state properly saved
- No navigation/routing errors
- Consistent rendering across devices

## Dependencies
- App status management system
- Setup wizard container component
- Design system components
- State persistence mechanism

## Page Content Structure

### Layout

### Responsive Layout
```
Desktop (>768px):
┌──────────────────────────┐
│      Progress Bar        │
├──────────────────────────┤
│         Logo             │
├──────────────────────────┤
│        Headline          │
├──────────────────────────┤
│      Introduction        │
├──────────────────────────┤
│    Benefits Section      │
│    (2x2 Grid Cards)      │
├──────────────────────────┤
│     Next Button          │
└──────────────────────────┘

Mobile (<768px):
┌──────────────────────┐
│    Progress Bar      │
├──────────────────────┤
│       Logo           │
├──────────────────────┤
│      Headline        │
├──────────────────────┤
│    Introduction      │
├──────────────────────┤
│ Benefits (Stacked)   │
│ ┌────────────────┐   │
│ │    Card 1      │   │
│ └────────────────┘   │
│ ┌────────────────┐   │
│ │    Card 2      │   │
│ └────────────────┘   │
│ ┌────────────────┐   │
│ │    Card 3      │   │
│ └────────────────┘   │
│ ┌────────────────┐   │
│ │    Card 4      │   │
│ └────────────────┘   │
├──────────────────────┤
│    Next Button       │
└──────────────────────┘
```

### Mobile Considerations
- Stack benefit cards vertically
- Reduce horizontal padding (16px)
- Adjust typography sizes:
  - Headline: 24px
  - Subheadline: 18px
  - Body: 16px
- Full-width buttons
- Collapsible sections if needed
- Touch-friendly tap targets (min 44px)

### Content Sections

#### Header
```
Welcome to Bodhi App
Run AI Models Locally, Privately, and Free
```

#### Our Vision
```
The word 'Bodhi' comes from Sanskrit, meaning intelligence or enlightenment. 
We believe artificial intelligence should be accessible to everyone, not just 
those who can afford expensive subscription or have the technical expertise to 
set up complex systems.

Bodhi App is our step towards democratizing AI - bringing powerful language 
models to your device, making AI accessible to underserved communities 
worldwide. By running everything locally and freely, we're working to ensure 
no one is left behind in the AI revolution.
```

#### Introduction
```
Bodhi App brings the power of Large Language Models (LLMs) directly to your device. 
Experience AI without compromising your privacy or paying for API calls.

We'll guide you through a simple setup process to get your personal AI assistant 
up and running on your machine in minutes.
```

#### Key Benefits (Card Grid with Icons)
```
🔒 Complete Privacy
Your data stays on your device. No cloud processing, no data sharing, 
just secure local inference.

💰 Always Free
Run unlimited AI inferences without usage fees or API costs. 
Your hardware, your rules.

🚀 Full Control
Choose and run any compatible LLM model. Customize settings and 
optimize for your needs.

⚡ Local Performance
Direct access to your hardware's capabilities for optimal performance 
without internet latency.
```

#### Setup Journey Preview
```
Quick Setup Steps:
1. Choose your authentication mode
2. Configure your environment
3. Get your first AI model
4. Start chatting!

Time to complete: ~5 minutes
```

#### Call to Action
```
Primary Button: "Begin Setup →"
Secondary Text: "Just a few steps to your personal AI assistant"
```

### Visual Elements
- Bodhi logo at top (SVG)
- Progress indicator showing step 1/5
- Benefit cards with subtle hover effects
- Loading states for status checks
- Smooth transitions between sections

### Typography
- Headline: Large, bold (32px)
- Subheadline: Medium, regular (20px)
- Body text: Regular (16px)
- Benefits headings: Medium (18px)
- Progress text: Small (14px)

### Spacing
- Comfortable vertical spacing between sections (32px)
- Card grid with consistent gaps (24px)
- Content container max-width (800px)
- Responsive padding on mobile (16px)

### Animations
- Subtle fade-in on page load
- Smooth hover states on cards
- Button hover/active states
- Progress indicator emphasis 
- Simple step indicators with icons
- Subtle progress visualization
- Card hover effects with gentle elevation
