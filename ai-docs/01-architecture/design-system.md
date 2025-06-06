# Bodhi Design System

## Overview

This document defines the design patterns and component usage guidelines for implementing consistent UI across Bodhi App screens. Focus is on implementation decisions that ensure visual and functional consistency.

## Layout Patterns

### Screen Structure
All screens should follow this consistent hierarchy:

```
app-header (fixed)
├── app-navigation
└── app-breadcrumb
app-main (scrollable)
└── page-content
    ├── page-header
    ├── content-sections
    └── page-actions
```

### Container Patterns
- **Full-width pages**: Use `container mx-auto px-4 sm:px-6 lg:px-8`
- **Form pages**: Use `max-w-2xl mx-auto px-4`
- **Content cards**: Use `card-container` class for consistent elevation
- **Section spacing**: Use `section-spacing` for consistent vertical rhythm

## Visual Hierarchy

### Background Layers
Use these semantic classes for consistent layering:

- `bg-base` - Main app background
- `bg-elevated` - Header, navigation surfaces
- `bg-overlay` - Cards, dialogs, modals

### Text Hierarchy
- `text-primary` - Main content, headings
- `text-secondary` - Supporting content, labels
- `text-muted` - Placeholder text, descriptions

### Border Usage
- `border-subtle` - Section separators, card borders
- `border-strong` - Focus states, active elements

### Status Colors
- `text-destructive` / `bg-destructive` - Errors, warnings
- `text-success` / `bg-success` - Success states
- `text-muted-foreground` - Disabled states

## Typography Patterns

### Heading Hierarchy
- **Page titles**: `text-3xl font-bold md:text-4xl`
- **Section titles**: `text-2xl font-bold md:text-3xl`
- **Subsection titles**: `text-xl font-bold md:text-2xl`
- **Card titles**: `text-lg font-semibold`

### Content Text
- **Body text**: `text-base` (default)
- **Small text**: `text-sm` for labels, captions
- **Tiny text**: `text-xs` for metadata, timestamps

### Text Styling
- **Emphasis**: `font-semibold` for important content
- **De-emphasis**: `text-muted-foreground` for secondary info
- **Interactive text**: `hover:text-primary` for links

## Spacing Patterns

### Content Spacing
- **Between sections**: `space-y-6` or `space-y-8`
- **Within cards**: `space-y-4` for form fields, `space-y-3` for content
- **Page sections**: `py-8 md:py-12` for consistent vertical rhythm

### Component Spacing
- **Card padding**: `p-6` for standard cards, `p-4` for compact cards
- **Form spacing**: `space-y-4` between fields, `space-y-6` between sections
- **Button groups**: `space-x-2` for related actions, `space-x-4` for separate actions

### Layout Spacing
- **Page margins**: `px-4 sm:px-6 lg:px-8` for responsive edge spacing
- **Content width**: `max-w-2xl` for forms, `max-w-4xl` for content pages

## Component Patterns

### Standard Card Pattern
```tsx
<Card className="max-w-2xl mx-auto shadow-sm">
  <CardHeader className="space-y-1">
    <CardTitle className="text-2xl font-bold">Title</CardTitle>
    <CardDescription className="text-sm text-muted-foreground">
      Description text
    </CardDescription>
  </CardHeader>
  <CardContent className="space-y-4">
    {/* Content */}
  </CardContent>
</Card>
```

### Form Pattern
```tsx
<FormField>
  <FormLabel className="text-sm font-medium">Label</FormLabel>
  <FormControl>
    <Input className="w-full" />
  </FormControl>
  <FormMessage className="text-sm text-destructive" />
</FormField>
```

### Page Header Pattern
```tsx
<div className="border-b border-subtle bg-elevated/50 px-6 py-4">
  <h1 className="text-2xl font-bold">Page Title</h1>
  <p className="text-sm text-muted-foreground">Page description</p>
</div>
```

### Content Section Pattern
```tsx
<div className="bg-overlay p-6 space-y-4 rounded-lg border border-subtle">
  <h2 className="text-lg font-semibold">Section Title</h2>
  <div className="space-y-3">
    {/* Section content */}
  </div>
</div>
```

## Responsive Patterns

### Mobile-First Approach
- Start with mobile layout, enhance for larger screens
- Use `sm:`, `md:`, `lg:` prefixes for responsive variants
- Test on mobile devices regularly

### Common Responsive Patterns
- **Text sizes**: `text-2xl md:text-3xl` for scalable headings
- **Spacing**: `py-4 md:py-6` for responsive vertical spacing
- **Grid layouts**: `grid-cols-1 md:grid-cols-2 lg:grid-cols-3`
- **Padding**: `px-4 sm:px-6 lg:px-8` for edge spacing

### Navigation Responsiveness
- Mobile: Hamburger menu, full-screen overlay
- Desktop: Horizontal navigation, sidebar navigation
- Use `hidden md:block` and `md:hidden` for responsive visibility

## Interactive States

### Button States
- **Primary actions**: `Button` with default styling
- **Secondary actions**: `Button variant="outline"`
- **Destructive actions**: `Button variant="destructive"`
- **Loading state**: Add `disabled` prop and loading spinner

### Form States
- **Default**: Standard input styling
- **Focus**: Automatic focus ring via Tailwind
- **Error**: `FormMessage` with `text-destructive`
- **Disabled**: `disabled` prop with reduced opacity

### Loading States
- **Skeleton loading**: `animate-pulse` with placeholder shapes
- **Spinner loading**: Use `Loader2` icon with `animate-spin`
- **Progressive loading**: Show partial content while loading

### Hover and Focus
- **Interactive elements**: `hover:bg-accent` for subtle hover
- **Links**: `hover:text-primary hover:underline`
- **Cards**: `hover:shadow-md transition-shadow`

## Component Usage Guidelines

### When to Use Cards
- **Form containers**: Always wrap forms in cards
- **Content sections**: Group related content
- **Data display**: Tables, lists, detailed information
- **NOT for**: Page-level layouts, navigation elements

### When to Use Buttons
- **Primary**: Main action on a page (only one per section)
- **Secondary**: Supporting actions, cancel buttons
- **Outline**: Less important actions, filters
- **Ghost**: Minimal actions, icon buttons

### When to Use Different Text Styles
- **Headings**: Clear hierarchy with consistent sizing
- **Body text**: Default for readable content
- **Muted text**: Supporting information, metadata
- **Small text**: Labels, captions, fine print

## Implementation Consistency Rules

### Screen Layout Consistency
1. **Always use the same header structure** across all pages
2. **Consistent page containers** - don't mix container patterns
3. **Uniform spacing** between sections using defined spacing classes
4. **Standard card patterns** for content grouping

### Form Consistency
1. **Same form field spacing** (`space-y-4`) across all forms
2. **Consistent label styling** (`text-sm font-medium`)
3. **Standard error message styling** (`text-sm text-destructive`)
4. **Uniform button placement** (right-aligned for primary actions)

### Navigation Consistency
1. **Same breadcrumb pattern** across all pages
2. **Consistent active states** for navigation items
3. **Standard mobile menu behavior**
4. **Uniform page transitions**

### Content Consistency
1. **Same heading hierarchy** across all content
2. **Consistent table styling** for data display
3. **Standard loading states** for all async content
4. **Uniform empty states** when no data is available

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

## Quick Reference

### Standard Page Structure
```tsx
// Every page should follow this structure
<div className="container mx-auto px-4 sm:px-6 lg:px-8">
  {/* Page header */}
  <div className="border-b border-subtle bg-elevated/50 px-6 py-4">
    <h1 className="text-2xl font-bold">Page Title</h1>
    <p className="text-sm text-muted-foreground">Description</p>
  </div>

  {/* Page content */}
  <div className="py-8 space-y-6">
    {/* Content sections */}
  </div>
</div>
```

### Standard Form Structure
```tsx
<Card className="max-w-2xl mx-auto">
  <CardHeader>
    <CardTitle>Form Title</CardTitle>
    <CardDescription>Form description</CardDescription>
  </CardHeader>
  <CardContent className="space-y-4">
    {/* Form fields with space-y-4 */}
  </CardContent>
  <CardFooter className="flex justify-end space-x-2">
    <Button variant="outline">Cancel</Button>
    <Button type="submit">Submit</Button>
  </CardFooter>
</Card>
```

### Standard Data Display
```tsx
<Card>
  <CardHeader>
    <CardTitle className="text-lg">Data Section</CardTitle>
  </CardHeader>
  <CardContent>
    {/* Use DataTable for tabular data */}
    {/* Use space-y-3 for list items */}
    {/* Use text-muted-foreground for metadata */}
  </CardContent>
</Card>
```

### Component Selection Guide

**For forms**: Always use `Card` wrapper with `CardHeader`, `CardContent`, `CardFooter`

**For data display**: Use `DataTable` for tabular data, `Card` for grouped information

**For actions**:
- Primary action: `Button` (default)
- Secondary action: `Button variant="outline"`
- Destructive action: `Button variant="destructive"`

**For navigation**: Use consistent breadcrumb and navigation patterns

**For feedback**: Use `toast` for notifications, `FormMessage` for form errors

## Related Documentation

- **[Frontend Architecture](frontend-architecture.md)** - Component organization and structure
- **[App Overview](app-overview.md)** - High-level application architecture
