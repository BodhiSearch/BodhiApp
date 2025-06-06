# Bodhi Design System

## Overview

The Bodhi design system provides a comprehensive foundation for creating consistent, accessible, and beautiful user interfaces across the Bodhi App ecosystem. Built on Tailwind CSS and Shadcn/ui, it ensures design consistency while maintaining flexibility for future growth.

## Core Technologies

### Tailwind CSS
**Utility-first CSS framework** with the following configuration:

- Custom theme configuration with CSS variables
- Dark mode support using 'class' strategy
- Custom color scheme with HSL values
- Responsive container configuration
- Animation support via tailwindcss-animate
- Custom font configuration using Inter

### Shadcn/ui
**Component library built on Radix UI** with:

- Default style configuration
- CSS variables based theming
- Base color: slate
- Component aliases configured
- Radix UI primitives for accessibility

## Color System

### Semantic Color Tokens

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

### Theme Implementation

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

### Extended Color Palette

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

### Visual Layer Hierarchy

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

## Typography

### Font System

- **Primary Font**: Inter - Modern, readable sans-serif
- **Font Loading**: Optimized for performance
- **Font Display**: Swap for better loading experience

### Typography Scale

```css
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

### Text Hierarchy

```css
/* Text styles */
h1 { @apply text-3xl font-bold md:text-4xl }
h2 { @apply text-2xl font-bold md:text-3xl }
h3 { @apply text-xl font-bold md:text-2xl }
```

## Spacing System

### Spacing Scale

```css
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

### Common Utilities

```css
/* Spacing utilities */
.content-spacing { @apply space-y-6 }
.section-spacing { @apply py-8 md:py-12 }

/* Layout containers */
.page-container { @apply container mx-auto px-4 sm:px-6 lg:px-8 }
.card-container { @apply rounded-lg border bg-card p-6 shadow-sm }
.form-container { @apply max-w-2xl mx-auto space-y-6 }
```

## Component Architecture

### Layout Structure

Base layout hierarchy:

```
root-layout
├── app-header
│   ├── app-navigation
│   └── app-breadcrumb
└── app-main
    └── page-content
```

### Component Identification

Components use data-testid for clear identification:

```tsx
<header data-testid="app-header">
<nav data-testid="app-navigation">
<main data-testid="app-main">
```

### Reusable Component Classes

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

## Responsive Design

### Breakpoint System

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

### Container Patterns

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

## Component Patterns

### Form Components

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

### Interactive Components

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

## Theming System

### Theme Configuration

- Dark theme is the default
- Theme colors defined using HSL values
- Semantic color mapping for consistency

### Utility Classes

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

## CSS Processing

- **PostCSS** for processing Tailwind directives
- **CSS variables** for theme customization
- **Global styles** in src/app/globals.css

## Build & Optimization

### React PWA Support
- PWA configuration via vite-plugin-pwa
- Aggressive frontend navigation caching
- Service worker optimization
- Workbox configuration

### Vite Build Configuration
- Static build output
- Asset optimization
- ESLint integration
- TypeScript compilation
- Font optimization
