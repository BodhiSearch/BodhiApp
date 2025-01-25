UI Components & Styling
~~~~~~~~~~~~~~~~~~~~~
- **Tailwind CSS**: Utility-first CSS framework
  - Custom theme configuration with CSS variables
  - Dark mode support using 'class' strategy
  - Custom color scheme with HSL values
  - Responsive container configuration
  - Animation support via tailwindcss-animate
  - Custom font configuration using Inter

- **Shadcn/ui**: Component library built on Radix UI
  - Default style configuration
  - CSS variables based theming
  - Base color: slate
  - Component aliases configured
  - Radix UI primitives:
    - Dialog
    - Dropdown Menu
    - Label
    - Popover
    - Scroll Area
    - Select
    - Separator
    - Slider
    - Slot
    - Switch
    - Toast
    - Tooltip

- **CSS Processing**:
  - PostCSS for processing Tailwind directives
  - CSS variables for theme customization
  - Global styles in src/app/globals.css

- **Theme System**:
  - Dark/Light mode support
  - System preference detection
  - Theme persistence in localStorage
  - CSS variables for color schemes
  - Customizable sidebar theming

Development Tools
~~~~~~~~~~~~~~~
- **ESLint**: Code linting
- **Prettier**: Code formatting
  - End of line: LF
  - Configured via prettier config
- **PostCSS**: CSS processing
- **Tailwind Config**: Extended configuration
  - Custom container queries
  - Extended color palette
  - Custom animations
  - Font family configuration
  - Border radius system
  - Custom keyframes

Build & Optimization
~~~~~~~~~~~~~~~~~~
- **Next.js PWA Support**:
  - PWA configuration via @ducanh2912/next-pwa
  - Aggressive frontend navigation caching
  - Service worker optimization
  - Workbox configuration

- **Next.js Build Configuration**:
  - Static export enabled
  - Trailing slash configuration
  - Image optimization disabled
  - ESLint integration
  - Custom webpack configuration
  - Geist font transpilation

Styling Conventions
~~~~~~~~~~~~~~~~
- Use Tailwind CSS classes for all styling
- Follow utility-first CSS approach
- Leverage CSS variables for theming
- Use clsx/tailwind-merge for conditional classes
- Maintain dark mode compatibility
- Follow HSL color system
- Use semantic color variables
- Implement responsive design patterns

Theme Configuration
~~~~~~~~~~~~~~~~
- Root theme variables in :root selector
- Dark theme variables in .dark selector
- Custom color schemes for:
  - Background/Foreground
  - Primary/Secondary colors
  - Accent colors
  - Destructive states
  - Muted variants
  - Card styles
  - Popover elements
  - Sidebar components
  - Chart colors

Component Architecture
~~~~~~~~~~~~~~~~~~~
- Modular component structure
- Composition using Radix primitives
- Variant support via cva
- Responsive design patterns
- Accessibility considerations
- Dark mode compatibility
- Animation integration
- State management patterns

These conventions ensure:
- Consistent styling across components
- Maintainable theme system
- Accessible user interface
- Responsive design implementation
- Performance optimization
- Developer experience