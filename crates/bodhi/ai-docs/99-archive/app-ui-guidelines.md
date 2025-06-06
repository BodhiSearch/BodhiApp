# UI/UX Guidelines

## Core Design System

### Color System

#### 1. Semantic Color Tokens

Base colors are defined using semantic naming that describes their purpose:

```css
/* Background Levels */
--background-base      /* Main app background */
--background-elevated  /* Header, elevated surfaces */
--background-overlay   /* Cards, dialogs */

/* Border Levels */
--border-subtle       /* Subtle separators */
--border-strong       /* Prominent borders */

/* Text Levels */
--text-primary       /* Primary content */
--text-secondary     /* Secondary content */
--text-muted        /* De-emphasized text */
```

#### 2. Theme Implementation

Colors are implemented through Tailwind's color system:

```typescript
colors: {
  'background-base': 'hsl(var(--background-base))',
  'background-elevated': 'hsl(var(--background-elevated))',
  'background-overlay': 'hsl(var(--background-overlay))',
  'border-subtle': 'hsl(var(--border-subtle))',
  'border-strong': 'hsl(var(--border-strong))',
  'text-primary': 'hsl(var(--text-primary))',
  'text-secondary': 'hsl(var(--text-secondary))',
  'text-muted': 'hsl(var(--text-muted))',
}
```

## Component Architecture

### 1. Layout Structure

Base layout hierarchy:

```
root-layout
├── app-header
│   ├── app-navigation
│   └── app-breadcrumb
└── app-main
    └── page-content
```

### 2. Component Identification

Components use data-testid for clear identification:

```tsx
<header data-testid="app-header">
<nav data-testid="app-navigation">
<main data-testid="app-main">
```

### 3. Reusable Component Classes

Common component patterns defined in components.css:

```css
.card-elevated {
  @apply bg-overlay border-subtle rounded-lg shadow-md;
}

.header-section {
  @apply border-b border-subtle bg-elevated/50;
}

.content-section {
  @apply bg-overlay p-4 space-y-3;
}

.text-description {
  @apply text-sm text-muted text-center;
}
```

## Theming System

### 1. Theme Configuration

- Dark theme is the default
- Theme colors defined using HSL values
- Semantic color mapping for consistency

### 2. Utility Classes

Semantic utility classes for common patterns:

```css
.bg-base      /* Main background */
.bg-elevated  /* Elevated surfaces */
.bg-overlay   /* Overlay surfaces */
.border-subtle
.border-strong
.text-primary
.text-secondary
.text-muted
```

## Best Practices

### 1. Color Usage
- Use semantic color tokens instead of direct color values
- Maintain consistent contrast ratios
- Follow color hierarchy for visual importance

### 2. Component Structure
- Use data-testid for component identification
- Follow consistent naming conventions
- Implement proper component hierarchy

### 3. Responsive Design
- Mobile-first approach
- Use semantic breakpoints
- Consistent spacing scale

### 4. Accessibility
- Proper color contrast
- Semantic HTML structure
- Clear visual hierarchy

## Development Guidelines

### 1. File Organization
- Component styles in components.css
- Theme configuration in globals.css
- Tailwind config in tailwind.config.ts

### 2. Naming Conventions
- Use semantic names for colors and utilities
- Consistent component naming
- Clear test ID structure

### 3. Code Style
- Use Tailwind utility classes
- Extract common patterns to components
- Maintain consistent spacing

## Migration Guide

### 1. Updating Existing Components
- Replace hardcoded colors with semantic tokens
- Add proper data-testid attributes
- Implement consistent spacing

### 2. Theme Changes
- Update color variables in globals.css
- Modify semantic color mapping
- Test in both light and dark modes

### 3. Quality Assurance
- Verify color contrast
- Test responsive behavior
- Validate component hierarchy

## Core Principles

### 1. Mobile-First Design
- Design for mobile screens first
- Progressive enhancement for larger screens
- Touch-friendly interface elements

### 2. Consistent Theming
- Light/Dark mode consistency using Tailwind's dark mode strategy
- Color hierarchy through CSS variables
- Component-level theming with shadcn/ui integration

### 3. Responsive Patterns
- Fluid layouts using Tailwind's responsive modifiers
- Adaptive components with mobile-first approach
- Content prioritization for different screen sizes

## Tailwind Best Practices

### 1. Utility-First Approach
- Use utility classes instead of custom CSS
- Leverage Tailwind's built-in utilities
- Extract components for repeated patterns

### 2. Responsive Design

```tsx
// Mobile-first approach
<div className="
  w-full          // Default for mobile
  md:w-1/2       // Medium screens (768px+)
  lg:w-1/3       // Large screens (1024px+)
">
```

### 3. State Management

```tsx
// Interactive states
<button className="
  bg-primary
  hover:bg-primary/90
  active:bg-primary/80
  disabled:opacity-50
">
```

### 4. Dark Mode

```tsx
// Dark mode variants
<div className="
  bg-white
  dark:bg-slate-900
  text-slate-900
  dark:text-white
">
```

## Component Guidelines

### Layout Structure

#### Page Layout

```tsx
<div className="min-h-screen flex flex-col">
  <header className="sticky top-0 z-50 h-16 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
    {/* Navigation */}
  </header>
  <main className="flex-1 container mx-auto px-4 py-6">
    {/* Content */}
  </main>
</div>
```

#### Container Widths

```tsx
// Default container with responsive padding
<div className="container mx-auto px-4 sm:px-6 lg:px-8">
  {/* Content */}
</div>

// Narrow container for forms
<div className="max-w-2xl mx-auto px-4">
  {/* Form content */}
</div>
```

### Color System

#### Extended Color Palette

```css
:root {
  /* Primary Colors */
  --primary: 250 95% 64%;      /* #6366f1 Indigo */
  --primary-foreground: 0 0% 100%;
  
  /* Background Colors */
  --background: 0 0% 100%;     /* White */
  --background-subtle: 240 10% 97%; /* Subtle gray */
  
  /* Text Colors */
  --text-primary: 240 10% 4%;  /* Near black */
  --text-secondary: 240 5% 34%; /* Gray */
  
  /* Component Colors */
  --component-bg: 0 0% 100%;   /* White */
  --component-border: 240 6% 90%;
  
  /* Status Colors */
  --success: 142 72% 29%;      /* Green */
  --warning: 38 92% 50%;       /* Orange */
  --error: 0 84% 60%;          /* Red */
  --info: 199 89% 48%;         /* Blue */
}
```

### Spacing & Typography

#### Consistent Spacing

```css
/* Spacing Scale */
spacing: {
  0: '0',
  1: '0.25rem',    /* 4px */
  2: '0.5rem',     /* 8px */
  3: '0.75rem',    /* 12px */
  4: '1rem',       /* 16px */
  5: '1.25rem',    /* 20px */
  6: '1.5rem',     /* 24px */
  8: '2rem',       /* 32px */
  10: '2.5rem',    /* 40px */
  12: '3rem',      /* 48px */
}
```

#### Typography Scale

```css
/* Font Sizes */
fontSize: {
  xs: ['0.75rem', { lineHeight: '1rem' }],
  sm: ['0.875rem', { lineHeight: '1.25rem' }],
  base: ['1rem', { lineHeight: '1.5rem' }],
  lg: ['1.125rem', { lineHeight: '1.75rem' }],
  xl: ['1.25rem', { lineHeight: '1.75rem' }],
  '2xl': ['1.5rem', { lineHeight: '2rem' }],
}
```

### Component Patterns

#### Form Components

```tsx
<Card className="max-w-2xl mx-auto shadow-sm">
  <CardHeader className="space-y-1">
    <CardTitle className="text-2xl font-bold">Form Title</CardTitle>
    <CardDescription className="text-sm text-muted-foreground">
      Form description here
    </CardDescription>
  </CardHeader>
  <CardContent className="space-y-4">
    <FormField>
      <FormLabel className="text-sm font-medium">Label</FormLabel>
      <FormControl>
        <Input className="w-full" />
      </FormControl>
      <FormMessage className="text-sm text-destructive" />
    </FormField>
  </CardContent>
</Card>
```

#### Interactive Components

```tsx
// Buttons with states
<Button
  className="
    bg-primary
    hover:bg-primary/90
    active:bg-primary/80
    disabled:opacity-50
    transition-colors
    duration-200
  "
>
  Click Me
</Button>

// Loading states
<div className="animate-pulse space-y-4">
  <div className="h-4 bg-slate-200 rounded dark:bg-slate-700" />
  <div className="h-4 bg-slate-200 rounded dark:bg-slate-700 w-3/4" />
</div>
```

## Mobile Optimization

### 1. Touch Targets
- Minimum touch target size of 44px
- Adequate spacing between interactive elements
- Clear visual feedback on touch

### 2. Performance
- Use responsive images
- Implement lazy loading
- Optimize animations for mobile

### 3. Navigation
- Implement hamburger menu for mobile
- Use bottom navigation when appropriate
- Ensure easy thumb reach for common actions

## Accessibility Guidelines

### 1. Color Contrast
- Maintain WCAG 2.1 AA standard (4.5:1 for normal text)
- Use Tailwind's built-in contrast checking
- Provide sufficient contrast in dark mode

### 2. Keyboard Navigation
- Implement focus styles
- Ensure logical tab order
- Support keyboard shortcuts

### 3. Screen Readers
- Use semantic HTML
- Implement ARIA labels
- Provide text alternatives for images

### 4. Reduced Motion
- Support prefers-reduced-motion
- Provide alternatives to animations
- Ensure critical functionality works without animation

## Best Practices

### 1. Component Creation
- Use semantic naming
- Follow mobile-first approach
- Implement proper error states
- Include loading states

### 2. Theme Consistency
- Use CSS variables for colors
- Follow spacing system
- Maintain dark mode support
- Test contrast ratios

### 3. Performance
- Lazy load components when possible
- Optimize images
- Minimize layout shifts
- Use proper loading states

### 4. Accessibility
- Include ARIA labels
- Ensure keyboard navigation
- Maintain proper contrast
- Support screen readers

## Base Configuration Changes

### Tailwind Configuration

#### 1. Container Breakpoints

```typescript
screens: {
  'sm': '640px',    // Small devices
  'md': '768px',    // Medium devices
  'lg': '1024px',   // Large devices
  'xl': '1280px',   // Extra large devices
  '2xl': '1400px',  // 2X Extra large devices
}
```

These breakpoints provide consistent responsive behavior:
- Mobile-first approach starting below 640px
- Tablet portrait mode at 768px
- Tablet landscape/small desktop at 1024px
- Standard desktop at 1280px
- Large desktop at 1400px

#### 2. Extended Color System

Added status colors for better UI feedback:
- `success`: Green shade for positive actions/states
- `warning`: Orange shade for cautionary states
- `info`: Blue shade for informational states

Each color includes a foreground variant for text/icon contrast.

#### 3. Typography Scale

Implemented consistent font sizing with proper line heights:

```typescript
fontSize: {
  xs: ['0.75rem', { lineHeight: '1rem' }],     // 12px
  sm: ['0.875rem', { lineHeight: '1.25rem' }], // 14px
  base: ['1rem', { lineHeight: '1.5rem' }],    // 16px
  lg: ['1.125rem', { lineHeight: '1.75rem' }], // 18px
  xl: ['1.25rem', { lineHeight: '1.75rem' }],  // 20px
  '2xl': ['1.5rem', { lineHeight: '2rem' }],   // 24px
  '3xl': ['1.875rem', { lineHeight: '2.25rem' }], // 30px
}
```

Benefits:
- Consistent vertical rhythm
- Improved readability
- Proper line height ratios

#### 4. Spacing System

Added semantic spacing values:

```typescript
spacing: {
  '4xs': '0.125rem', // 2px - Minimal spacing
  '3xs': '0.25rem',  // 4px - Tight spacing
  '2xs': '0.375rem', // 6px - Very compact
  'xs': '0.5rem',    // 8px - Compact
  'sm': '0.75rem',   // 12px - Small
  'md': '1rem',      // 16px - Medium
  'lg': '1.5rem',    // 24px - Large
  'xl': '2rem',      // 32px - Extra large
  '2xl': '2.5rem',   // 40px - 2X Extra large
  '3xl': '3rem',     // 48px - 3X Extra large
}
```

Benefits:
- Semantic naming for better understanding
- Consistent spacing increments
- Scale suitable for both mobile and desktop

### Global CSS Changes

#### 1. Color System Organization

Colors are now organized by purpose:
- Base colors (background, foreground)
- Component colors (card, popover)
- Primary/Secondary colors
- Status colors
- UI element colors

#### 2. Dark Mode Optimization

Improved dark mode colors for:
- Better contrast ratios
- Reduced eye strain
- Consistent component appearance
- Proper hierarchy preservation

#### 3. Common Utilities

Added base utilities:

```css
/* Text styles */
h1 { @apply text-3xl font-bold md:text-4xl }
h2 { @apply text-2xl font-bold md:text-3xl }
h3 { @apply text-xl font-bold md:text-2xl }

/* Spacing utilities */
.content-spacing { @apply space-y-6 }
.section-spacing { @apply py-8 md:py-12 }

/* Layout containers */
.page-container { @apply container mx-auto px-4 sm:px-6 lg:px-8 }
.card-container { @apply rounded-lg border bg-card p-6 shadow-sm }
.form-container { @apply max-w-2xl mx-auto space-y-6 }
```

Benefits:
- Consistent component spacing
- Responsive text sizing
- Reusable layout patterns

### Implementation Impact

#### 1. Component Consistency
- All components now share consistent spacing
- Typography follows clear hierarchy
- Color usage is more predictable

#### 2. Responsive Design
- Mobile-first approach enforced
- Consistent breakpoints across app
- Proper spacing at all screen sizes

#### 3. Maintenance
- Centralized configuration
- Semantic naming for better understanding
- Reduced custom CSS needs

#### 4. Accessibility
- Improved color contrast
- Consistent text sizing
- Better dark mode support

### Migration Steps

#### 1. Immediate Updates
- Update existing components to use new spacing
- Audit color usage for consistency
- Apply responsive text classes

#### 2. Gradual Adoption
- Replace custom spacing with new scale
- Update color variables usage
- Implement new container classes

#### 3. Quality Checks
- Verify dark mode appearance
- Test responsive behavior
- Validate color contrast
- Check component spacing

## File Organization

### 1. Required Style Files

```
src/
├── styles/
│   ├── components.css    # Reusable component styles
│   └── syntax-highlighter.css
└── app/
    └── globals.css       # Global styles and theme variables
```

### 2. Style File Integration

Add to app/layout.tsx:

```typescript
import '@/app/globals.css';
import '@/styles/components.css';
import '@/styles/syntax-highlighter.css';
```

## Visual Hierarchy & Component Architecture

This section outlines learnings and best practices for visual hierarchy and component architecture.

### Color System & Visual Layers

#### Color Token Organization

Define colors in `globals.css` using a layered approach:

```css
:root {
  /* Base Layer */
  --background: 240 10% 98%    /* Slightly off-white (#F9F9FB) */

  /* Content Layer */
  --card: 0 0% 100%           /* Pure white (#FFFFFF) */

  /* Top Layer */
  --header-elevated: 240 10% 94% /* Darker for distinction */
}

.dark {
  /* Base Layer */
  --background: 240 10% 1%     /* Almost black (#020203) */

  /* Content Layer */
  --card: 240 10% 4.5%        /* Lighter (#0B0B0D) */

  /* Top Layer */
  --header-elevated: 240 10% 10% /* Lightest for distinction */
}
```

#### Visual Layer Hierarchy

1. **Base Layer (Background)**
   - Light theme: Slightly off-white for subtle foundation
   - Dark theme: Deep dark for reduced eye strain
   - Purpose: Creates foundation for content

2. **Content Layer (Cards)**
   - Light theme: Pure white for readability
   - Dark theme: Slightly elevated from background
   - Purpose: Main content areas and interactive elements

3. **Top Layer (Header)**
   - Light theme: Darker than background for distinction
   - Dark theme: Lighter than content for hierarchy
   - Uses blur and opacity for depth
   - Purpose: Navigation and key UI elements

### Component Architecture Patterns

#### Layout Structure

Organize layouts with clear containment:

```tsx
// layout.tsx
<div className="min-h-screen flex flex-col">
  <header className="sticky top-0 z-50 bg-header-elevated/90 backdrop-blur-sm">
    <Navigation />
    <Breadcrumb />
  </header>
  <main className="flex-1">
    {children}
  </main>
</div>
```

#### Tailwind Class Organization

1. **Layout Properties**

```tsx
className={cn(
  // Layout
  'flex-1 flex',
  // Positioning
  'sticky top-0 z-50',
  // Dimensions
  'h-16 px-4',
  // Visual
  'bg-header-elevated/90 backdrop-blur-sm'
)}
```

2. **Conditional Styling**

```tsx
className={cn(
  'base-styles',
  {
    'w-64': !isMobile,
    'w-10 px-0': isMobile
  }
)}
```

#### Component Best Practices

1. **Container vs Component**
   - Containers handle layout and positioning
   - Components focus on internal structure
   - Keep styling concerns separated

2. **Visual Consistency**
   - Use consistent elevation patterns
   - Maintain color hierarchy
   - Apply consistent spacing
   - Consider both themes

3. **Code Organization**
   - Group related styles
   - Use semantic class names
   - Include descriptive test IDs
   - Follow consistent patterns

These patterns ensure maintainable, scalable UI development while providing clear visual hierarchy across light and dark themes.
