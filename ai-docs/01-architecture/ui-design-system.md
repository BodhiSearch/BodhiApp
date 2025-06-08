# UI Design System

This document provides guidance for UI design patterns, component usage, and visual consistency in the Bodhi App.

## Required Documentation References

**MUST READ for implementation:**
- `ai-docs/01-architecture/frontend-react.md` - React component patterns and development
- `ai-docs/01-architecture/development-conventions.md` - Styling conventions and best practices

**FOR DETAILED DESIGN SPECIFICATIONS:**
- `ai-docs/01-architecture/design-system.md` - Complete design system documentation

## Design System Foundation

### Technology Stack
- **Tailwind CSS** - Utility-first CSS framework for rapid styling
- **Shadcn/ui** - Component library built on Radix UI primitives
- **Radix UI** - Unstyled, accessible UI primitives
- **Class Variance Authority (CVA)** - Component variant management
- **Lucide React** - Consistent icon library

### Design Tokens

#### Color System
```css
/* Primary Colors */
--primary: 222.2 84% 4.9%;
--primary-foreground: 210 40% 98%;

/* Secondary Colors */
--secondary: 210 40% 96%;
--secondary-foreground: 222.2 84% 4.9%;

/* Accent Colors */
--accent: 210 40% 96%;
--accent-foreground: 222.2 84% 4.9%;

/* Semantic Colors */
--destructive: 0 84.2% 60.2%;
--destructive-foreground: 210 40% 98%;
```

#### Typography Scale
```css
/* Font Sizes */
--text-xs: 0.75rem;    /* 12px */
--text-sm: 0.875rem;   /* 14px */
--text-base: 1rem;     /* 16px */
--text-lg: 1.125rem;   /* 18px */
--text-xl: 1.25rem;    /* 20px */
--text-2xl: 1.5rem;    /* 24px */
--text-3xl: 1.875rem;  /* 30px */
--text-4xl: 2.25rem;   /* 36px */
```

#### Spacing Scale
```css
/* Spacing Units */
--spacing-1: 0.25rem;  /* 4px */
--spacing-2: 0.5rem;   /* 8px */
--spacing-3: 0.75rem;  /* 12px */
--spacing-4: 1rem;     /* 16px */
--spacing-6: 1.5rem;   /* 24px */
--spacing-8: 2rem;     /* 32px */
--spacing-12: 3rem;    /* 48px */
--spacing-16: 4rem;    /* 64px */
```

## Component Usage Patterns

### Button Components
```typescript
import { Button } from "@/components/ui/button";

// Primary button
<Button variant="default" size="default">
  Primary Action
</Button>

// Secondary button
<Button variant="secondary" size="default">
  Secondary Action
</Button>

// Destructive button
<Button variant="destructive" size="default">
  Delete
</Button>

// Ghost button
<Button variant="ghost" size="sm">
  <Plus className="h-4 w-4 mr-2" />
  Add Item
</Button>
```

### Form Components
```typescript
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

// Input field
<div className="space-y-2">
  <Label htmlFor="name">Name</Label>
  <Input id="name" placeholder="Enter name" />
</div>

// Textarea
<div className="space-y-2">
  <Label htmlFor="description">Description</Label>
  <Textarea id="description" placeholder="Enter description" />
</div>

// Select dropdown
<Select>
  <SelectTrigger>
    <SelectValue placeholder="Select option" />
  </SelectTrigger>
  <SelectContent>
    <SelectItem value="option1">Option 1</SelectItem>
    <SelectItem value="option2">Option 2</SelectItem>
  </SelectContent>
</Select>
```

### Dialog Components
```typescript
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";

<Dialog>
  <DialogTrigger asChild>
    <Button variant="outline">Open Dialog</Button>
  </DialogTrigger>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Dialog Title</DialogTitle>
    </DialogHeader>
    <div className="space-y-4">
      {/* Dialog content */}
    </div>
  </DialogContent>
</Dialog>
```

### Card Components
```typescript
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

<Card>
  <CardHeader>
    <CardTitle>Card Title</CardTitle>
  </CardHeader>
  <CardContent>
    <p>Card content goes here.</p>
  </CardContent>
</Card>
```

## Layout Patterns

### Container Layouts
```typescript
// Page container
<div className="container mx-auto px-4 py-8">
  <h1 className="text-3xl font-bold mb-6">Page Title</h1>
  {/* Page content */}
</div>

// Section container
<section className="space-y-6">
  <h2 className="text-2xl font-semibold">Section Title</h2>
  {/* Section content */}
</section>

// Grid layout
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
  {items.map(item => (
    <Card key={item.id}>
      {/* Card content */}
    </Card>
  ))}
</div>
```

### Responsive Design
```typescript
// Mobile-first responsive classes
<div className="flex flex-col md:flex-row gap-4">
  <div className="w-full md:w-1/3">Sidebar</div>
  <div className="w-full md:w-2/3">Main content</div>
</div>

// Responsive text sizes
<h1 className="text-2xl md:text-3xl lg:text-4xl font-bold">
  Responsive Heading
</h1>

// Responsive spacing
<div className="p-4 md:p-6 lg:p-8">
  Content with responsive padding
</div>
```

## Icon Usage

### Lucide React Icons
```typescript
import { Plus, Edit, Trash2, Download, Upload, Settings } from "lucide-react";

// Icon in button
<Button variant="ghost" size="sm">
  <Plus className="h-4 w-4 mr-2" />
  Add Item
</Button>

// Standalone icon
<Settings className="h-5 w-5 text-muted-foreground" />

// Icon with consistent sizing
<div className="flex items-center space-x-2">
  <Download className="h-4 w-4" />
  <span>Download</span>
</div>
```

### Icon Sizing Guidelines
- **Small icons**: `h-4 w-4` (16px) - For buttons, inline text
- **Medium icons**: `h-5 w-5` (20px) - For navigation, cards
- **Large icons**: `h-6 w-6` (24px) - For headers, prominent actions
- **Extra large icons**: `h-8 w-8` (32px) - For empty states, illustrations

## Color Usage Patterns

### Semantic Color Classes
```typescript
// Primary actions
<Button className="bg-primary text-primary-foreground">
  Primary Action
</Button>

// Success states
<div className="bg-green-50 border border-green-200 text-green-800 p-4 rounded-md">
  Success message
</div>

// Warning states
<div className="bg-yellow-50 border border-yellow-200 text-yellow-800 p-4 rounded-md">
  Warning message
</div>

// Error states
<div className="bg-red-50 border border-red-200 text-red-800 p-4 rounded-md">
  Error message
</div>

// Muted text
<p className="text-muted-foreground">
  Secondary information
</p>
```

### Background and Border Colors
```typescript
// Card backgrounds
<Card className="bg-card border border-border">
  Card content
</Card>

// Hover states
<div className="hover:bg-accent hover:text-accent-foreground transition-colors">
  Hoverable content
</div>

// Focus states
<Input className="focus:ring-2 focus:ring-primary focus:border-primary" />
```

## Animation and Transitions

### Transition Classes
```typescript
// Smooth transitions
<Button className="transition-colors duration-200 hover:bg-primary/90">
  Hover me
</Button>

// Transform transitions
<div className="transform transition-transform duration-200 hover:scale-105">
  Scalable content
</div>

// Opacity transitions
<div className="transition-opacity duration-300 hover:opacity-80">
  Fade on hover
</div>
```

### Loading States
```typescript
import { Loader2 } from "lucide-react";

// Loading button
<Button disabled={isLoading}>
  {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
  {isLoading ? "Loading..." : "Submit"}
</Button>

// Loading spinner
<div className="flex items-center justify-center p-8">
  <Loader2 className="h-8 w-8 animate-spin" />
</div>
```

## Accessibility Patterns

### Focus Management
```typescript
// Focus visible styles
<Button className="focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2">
  Accessible button
</Button>

// Skip links
<a
  href="#main-content"
  className="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 bg-primary text-primary-foreground px-4 py-2 rounded-md"
>
  Skip to main content
</a>
```

### Screen Reader Support
```typescript
// ARIA labels
<Button aria-label="Close dialog">
  <X className="h-4 w-4" />
</Button>

// Descriptive text for screen readers
<div>
  <span className="sr-only">Loading</span>
  <Loader2 className="h-4 w-4 animate-spin" aria-hidden="true" />
</div>

// Form labels
<Label htmlFor="email" className="sr-only">
  Email address
</Label>
<Input id="email" type="email" placeholder="Email address" />
```

## Dark Mode Support

### Theme-Aware Colors
```typescript
// Automatic dark mode colors
<div className="bg-background text-foreground">
  Content that adapts to theme
</div>

// Theme-specific styling
<Card className="bg-card border-border">
  <CardContent className="text-card-foreground">
    Theme-aware card
  </CardContent>
</Card>
```

## Component Composition Patterns

### Compound Components
```typescript
// Form field composition
<FormField
  control={form.control}
  name="fieldName"
  render={({ field }) => (
    <FormItem>
      <FormLabel>Field Label</FormLabel>
      <FormControl>
        <Input {...field} />
      </FormControl>
      <FormDescription>
        Optional field description
      </FormDescription>
      <FormMessage />
    </FormItem>
  )}
/>
```

### Conditional Rendering
```typescript
// Conditional UI elements
{isLoading ? (
  <div className="flex items-center justify-center p-8">
    <Loader2 className="h-8 w-8 animate-spin" />
  </div>
) : (
  <div className="space-y-4">
    {/* Content */}
  </div>
)}

// Error states
{error && (
  <div className="bg-destructive/15 border border-destructive/20 text-destructive p-4 rounded-md">
    {error.message}
  </div>
)}
```

## Performance Considerations

### Efficient Class Names
```typescript
import { cn } from "@/lib/utils";

// Conditional classes with cn utility
<div
  className={cn(
    "base-classes",
    condition && "conditional-classes",
    variant === "primary" && "variant-classes"
  )}
>
```

### Lazy Loading
```typescript
// Lazy load heavy components
const HeavyComponent = lazy(() => import("./HeavyComponent"));

<Suspense fallback={<div>Loading...</div>}>
  <HeavyComponent />
</Suspense>
```

## Chat Interface Design Specifications

### Chat UI Components

The chat interface follows a modern, minimal design with these key components:

#### Layout Structure
- **Main central chat area** for displaying messages and input
- **Left sidebar panel** for settings (collapsible on mobile)
- **Clean header** with app title and theme toggle

#### Chat Area Design
- **Message bubbles** with clear visual distinction between user and LLM messages
- **Typing/streaming animation** for LLM responses in real-time
- **Input area** with text field and send button
- **Copy button** for LLM responses (appears on hover)
- **Empty state** with prompt to start chatting
- **Context actions** (copy, redo) visible on hover for assistant responses

#### Sidebar Panels (Left Panel)
The sidebar uses a collapsible design similar to Discord channels:

1. **Model Settings Panel**
   - API key management (update/view)
   - Model selection dropdown
   - Temperature slider control
   - System prompt text area

2. **Chat Settings Panel**
   - Tool selection (web search, Python code interpreter)
   - Mode selection (AI Chat, Deep Research)
   - Settings indicators below chat input

3. **Workspace Panel**
   - File upload functionality
   - Google Drive integration
   - File Access API permissions
   - File read/write capabilities

### Chat Design Specifications

#### Color Scheme
- **Primary**: Soft, calming blue (#3B82F6)
- **Light theme**: Clean white background with dark text
- **Dark theme**: Deep blue/gray background (#1E293B) with light text
- **Success, error, warning states** with appropriate semantic colors

#### Typography
- **Sans-serif font** for clean readability
- **Clear hierarchy** with different weights and sizes
- **Good contrast** for accessibility compliance

#### Message Styling
```typescript
// User message styling
<div className="bg-primary text-primary-foreground rounded-lg p-3 ml-auto max-w-[80%]">
  User message content
</div>

// Assistant message styling
<div className="bg-muted text-muted-foreground rounded-lg p-3 mr-auto max-w-[80%]">
  Assistant response content
</div>
```

## Common Mistakes to Avoid

### Layout Mistakes
- ❌ Mixing different container widths on the same page
- ❌ Inconsistent spacing between sections
- ❌ Using custom spacing instead of design system values
- ❌ Breaking the established visual hierarchy

### Component Mistakes
- ❌ Using multiple primary buttons in the same section
- ❌ Inconsistent card padding across different screens
- ❌ Mixing different text styles for the same content type
- ❌ Custom colors instead of semantic color tokens

### Responsive Mistakes
- ❌ Forgetting mobile-first approach
- ❌ Inconsistent breakpoint usage
- ❌ Different responsive patterns for similar content
- ❌ Not testing on actual mobile devices

### Chat Interface Mistakes
- ❌ Inconsistent message bubble styling between user and assistant
- ❌ Poor contrast in dark mode for message text
- ❌ Missing loading states during streaming responses
- ❌ Inaccessible copy buttons without proper ARIA labels

## Implementation Consistency Rules

### Standard Page Structure
```typescript
<div className="container mx-auto px-4 py-8">
  <header className="mb-6">
    <h1 className="text-3xl font-bold">Page Title</h1>
  </header>
  <main className="space-y-6">
    {/* Page content */}
  </main>
</div>
```

### Standard Form Structure
```typescript
<Form {...form}>
  <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
    <FormField
      control={form.control}
      name="fieldName"
      render={({ field }) => (
        <FormItem>
          <FormLabel>Field Label</FormLabel>
          <FormControl>
            <Input {...field} />
          </FormControl>
          <FormMessage />
        </FormItem>
      )}
    />
    <Button type="submit">Submit</Button>
  </form>
</Form>
```

### Consistency Guidelines
- **Always use the same header structure** across all pages
- **Uniform button placement** (primary action on the right)
- **Consistent card padding** (p-6 for large cards, p-4 for small cards)
- **Standard spacing** between sections (space-y-6 for main content)
- **Consistent error states** with same styling and messaging patterns

## Related Documentation

- **[Frontend React](frontend-react.md)** - React component patterns and development
- **[Development Conventions](development-conventions.md)** - Styling conventions and best practices
- **[Design System](design-system.md)** - Complete design system specifications
- **[Frontend Testing](frontend-testing.md)** - UI testing patterns and accessibility testing

---

*For complete design specifications and detailed component documentation, see [Design System](design-system.md). For React implementation patterns, see [Frontend React](frontend-react.md).*
