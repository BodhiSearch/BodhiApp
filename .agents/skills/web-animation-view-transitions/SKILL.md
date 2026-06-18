---
name: web-animation-view-transitions
description: View Transitions API patterns - same-document transitions, cross-document MPA transitions, shared element animations, pseudo-element styling, accessibility
---

# View Transitions API Patterns

> **Quick Guide:** Use the View Transitions API for native page/state transitions. `document.startViewTransition()` for same-document, `@view-transition { navigation: auto }` for cross-document MPA. Always feature-detect before use and respect `prefers-reduced-motion`. Use the options form `startViewTransition({ update, types })` when you need typed transitions.

---

<critical_requirements>

## CRITICAL: Before Using This Skill

> **All code must follow project conventions in CLAUDE.md** (kebab-case, named exports, import ordering, `import type`, named constants)

**(You MUST feature-detect before using startViewTransition - it is NOT available in all browsers)**

**(You MUST respect prefers-reduced-motion by providing reduced or disabled animations)**

**(You MUST ensure view-transition-name values are unique - duplicate names break transitions)**

**(You MUST clean up dynamically assigned view-transition-name values after transitions complete)**

**(You MUST use named constants for all animation timing values - NO magic numbers)**

</critical_requirements>

---

**Auto-detection:** View Transitions API, startViewTransition, view-transition-name, @view-transition, ::view-transition, pageswap, pagereveal, ViewTransition, view-transition-class, match-element, active-view-transition-type

**When to use:**

- Animating state changes in single-page applications
- Creating smooth page-to-page transitions in multi-page applications
- Implementing shared element (hero) animations between views
- Providing visual continuity during navigation
- Creating custom transition effects (slide, scale, circular reveal)

**Key patterns covered:**

- Same-document transitions with startViewTransition()
- Cross-document MPA transitions with @view-transition CSS
- view-transition-name for shared element animations
- Pseudo-element styling (::view-transition-old, ::view-transition-new)
- Direction-aware transitions with :active-view-transition-type()
- Feature detection and graceful fallbacks
- prefers-reduced-motion accessibility patterns

**When NOT to use:**

- Complex physics-based animations (use animation libraries)
- Animations requiring precise timeline control
- Simple hover/focus effects (use CSS transitions)

**Detailed Resources:**

- [examples/core.md](examples/core.md) - Feature detection, state transitions, promise handling, CSS customization
- [examples/spa.md](examples/spa.md) - Theme switcher, form steps, tab panels, list reordering
- [examples/shared-elements.md](examples/shared-elements.md) - Hero animations, multiple shared elements, MPA shared elements, modals
- [reference.md](reference.md) - Decision frameworks, pseudo-element reference, browser support, anti-patterns

---

<philosophy>

## Philosophy

The View Transitions API provides a native browser mechanism for creating animated transitions between DOM states or pages. It captures "before" and "after" snapshots, overlays them as pseudo-elements, and animates between them.

**Core principles:**

1. **Native over library** - Browser-native transitions are more performant and require less JavaScript
2. **Progressive enhancement** - Always feature-detect and provide functional fallback
3. **Snapshot-based** - Old state is captured as a screenshot, new state as a live representation
4. **CSS-driven** - Customize animations through pseudo-element CSS, not JavaScript
5. **Accessibility-first** - Always respect prefers-reduced-motion user preferences

</philosophy>

---

<patterns>

## Core Patterns

### Pattern 1: Feature Detection with Fallback

Always check for API support before using View Transitions. See [examples/core.md](examples/core.md) Pattern 1 for full utility.

```typescript
const SUPPORTS_VIEW_TRANSITIONS =
  typeof document !== "undefined" && "startViewTransition" in document;

function updateWithTransition(updateFn: () => void | Promise<void>): void {
  if (!SUPPORTS_VIEW_TRANSITIONS) {
    updateFn();
    return;
  }
  document.startViewTransition(() => updateFn());
}
```

**Why good:** Prevents runtime errors in unsupported browsers, provides seamless fallback

---

### Pattern 2: Same-Document (SPA) Transitions

Animate DOM state changes within a single page. See [examples/core.md](examples/core.md) Patterns 2-5 for state transitions, async loading, promise handling, and skip logic.

```typescript
// startViewTransition accepts a callback or an options object
const transition = document.startViewTransition(async () => {
  await updateFn();
});

// Options form - set types for CSS targeting
const transition = document.startViewTransition({
  update: () => updateDOM(),
  types: ["slide-forward"],
});

await transition.finished;
```

**ViewTransition object provides three promises:**

| Promise                         | Resolves when                 |
| ------------------------------- | ----------------------------- |
| `transition.ready`              | Pseudo-element tree created   |
| `transition.updateCallbackDone` | DOM update callback completed |
| `transition.finished`           | Animation complete            |

---

### Pattern 3: Cross-Document (MPA) Transitions

Enable transitions between separate pages without JavaScript. Both pages must opt in.

```css
/* Include on BOTH source and destination pages */
@view-transition {
  navigation: auto;
}
```

**Why good:** No JavaScript required, works for traverse/push/replace navigations

**Obsolete syntax:** `<meta name="view-transition" content="same-origin">` - use the CSS at-rule instead.

---

### Pattern 4: Shared Element Transitions

Create hero animations by giving matching elements the same `view-transition-name`. See [examples/shared-elements.md](examples/shared-elements.md) for full product list-to-detail, multi-element, and MPA examples.

```css
:root {
  --hero-duration: 300ms;
  --hero-easing: cubic-bezier(0.4, 0, 0.2, 1);
}

/* Same name on both pages/states creates shared element animation */
.product-thumbnail {
  view-transition-name: product-hero;
}
.product-image {
  view-transition-name: product-hero;
}

::view-transition-group(product-hero) {
  animation-duration: var(--hero-duration);
  animation-timing-function: var(--hero-easing);
}
```

**Key rules:** Names must be unique across the document. Clean up dynamically assigned names after `transition.finished`.

---

### Pattern 5: Custom CSS Animations

Override default cross-fade with custom animations via pseudo-elements. See [examples/core.md](examples/core.md) Pattern 6 for full examples.

```css
:root {
  --transition-duration: 300ms;
  --transition-easing: ease-in-out;
}

::view-transition-old(root) {
  animation: slide-out-left var(--transition-duration) var(--transition-easing);
}
::view-transition-new(root) {
  animation: slide-in-right var(--transition-duration) var(--transition-easing);
}
```

**Why good:** CSS custom properties for timing constants, GPU-accelerated transforms

---

### Pattern 6: Direction-Aware Transitions

Use different animations for forward vs backward navigation. Use the `types` parameter or `ViewTransition.types` set.

```css
html:active-view-transition-type(forwards) {
  &::view-transition-old(content) {
    animation-name: slide-out-left;
  }
  &::view-transition-new(content) {
    animation-name: slide-in-right;
  }
}

html:active-view-transition-type(backwards) {
  &::view-transition-old(content) {
    animation-name: slide-out-right;
  }
  &::view-transition-new(content) {
    animation-name: slide-in-left;
  }
}
```

```typescript
// Preferred: set types via options parameter
document.startViewTransition({
  update: () => navigateForward(),
  types: ["forwards"],
});

// Alternative: mutate types set on existing transition
const transition = document.startViewTransition(updateFn);
transition.types.add("forwards");
```

See [examples/spa.md](examples/spa.md) for form step and tab panel examples.

---

### Pattern 7: Accessibility - Reduced Motion

Always respect user preferences for reduced motion.

```css
@media (prefers-reduced-motion: reduce) {
  ::view-transition-group(*),
  ::view-transition-old(*),
  ::view-transition-new(*) {
    animation-duration: 0.01ms !important;
  }
}
```

```typescript
const REDUCED_MOTION_QUERY = "(prefers-reduced-motion: reduce)";

function shouldEnableTransitions(): boolean {
  if (window.matchMedia(REDUCED_MOTION_QUERY).matches) return false;
  return "startViewTransition" in document;
}
```

See [examples/spa.md](examples/spa.md) for a full accessible transition wrapper with preference change monitoring.

---

### Pattern 8: Circular Reveal Effect

Advanced custom animation using Web Animations API. Must `await transition.ready` before animating pseudo-elements.

```typescript
const REVEAL_DURATION_MS = 400;
const REVEAL_EASING = "ease-in-out";

const transition = document.startViewTransition(updateFn);
await transition.ready;

document.documentElement.animate(
  {
    clipPath: [`circle(0 at ${x}px ${y}px)`, `circle(${r}px at ${x}px ${y}px)`],
  },
  {
    duration: REVEAL_DURATION_MS,
    easing: REVEAL_EASING,
    pseudoElement: "::view-transition-new(root)",
  },
);
```

See [examples/spa.md](examples/spa.md) for a complete theme-switcher circular reveal implementation.

</patterns>

---

<red_flags>

## RED FLAGS

**High Priority Issues:**

- **Missing feature detection** - Calling `startViewTransition()` without checking support crashes in older browsers
- **Duplicate view-transition-name values** - Two visible elements with the same name breaks the transition entirely
- **Not cleaning up dynamic names** - Leftover names cause conflicts in subsequent transitions
- **Ignoring prefers-reduced-motion** - Mandatory for accessibility; always provide reduced or no animation
- **Magic numbers for timing** - All duration/delay values must be named constants or CSS custom properties

**Medium Priority Issues:**

- **Using obsolete meta tag syntax** - `<meta name="view-transition">` is deprecated; use `@view-transition` CSS
- **Not awaiting transition.ready for custom animations** - Web Animations API must wait for pseudo-elements to exist
- **Missing @view-transition on both MPA pages** - Cross-document transitions require opt-in on source AND destination
- **Setting view-transition-name in CSS for dynamic lists** - Causes name conflicts; use JavaScript assignment or `match-element`

**Gotchas & Edge Cases:**

- **Old state is a screenshot** - Videos, animations, GIFs freeze in the old snapshot
- **New state is "live"** - Interactive content continues playing in the new snapshot
- **Transition names are global** - Same name on different page sections will conflict
- **Animations block interaction** - User cannot interact until transition completes; keep animations under 300ms
- **Cross-document needs same-origin** - Different origins cannot share transitions
- **`match-element` requires Chrome 137+/Safari 18.4+** - Not yet available in Firefox
- **`pagereveal` must be registered early** - Put handler in `<head>` or use `blocking="render"`
- **Reserved names** (`auto`, `inherit`, `none`, `unset`) are CSS keywords, not valid custom identifiers

</red_flags>

---

<critical_reminders>

## CRITICAL REMINDERS

> **All code must follow project conventions in CLAUDE.md**

**(You MUST feature-detect before using startViewTransition - it is NOT available in all browsers)**

**(You MUST respect prefers-reduced-motion by providing reduced or disabled animations)**

**(You MUST ensure view-transition-name values are unique - duplicate names break transitions)**

**(You MUST clean up dynamically assigned view-transition-name values after transitions complete)**

**(You MUST use named constants for all animation timing values - NO magic numbers)**

**Failure to follow these rules will break transitions in unsupported browsers and create inaccessible experiences.**

</critical_reminders>
