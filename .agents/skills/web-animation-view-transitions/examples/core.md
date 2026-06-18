# View Transitions Core Examples

> Complete code examples for basic View Transitions patterns. See [SKILL.md](../SKILL.md) for core concepts and [reference.md](../reference.md) for decision frameworks.

**For advanced patterns**: See topic-specific files in this folder:

- [spa.md](spa.md) - Same-document SPA transitions
- [shared-elements.md](shared-elements.md) - Shared element hero animations

---

## Pattern 1: Feature Detection Utility

### Good Example - Comprehensive Feature Detection

```typescript
// view-transition-utils.ts

// Feature detection constant
export const SUPPORTS_VIEW_TRANSITIONS =
  typeof document !== "undefined" && "startViewTransition" in document;

// Type for callback functions
type TransitionCallback = () => void | Promise<void>;

/**
 * Execute a function with optional view transition.
 * Falls back gracefully if transitions are not supported.
 */
export async function withViewTransition(
  updateFn: TransitionCallback,
): Promise<void> {
  if (!SUPPORTS_VIEW_TRANSITIONS) {
    await updateFn();
    return;
  }

  const transition = document.startViewTransition(async () => {
    await updateFn();
  });

  await transition.finished;
}

/**
 * Execute a function with view transition, returning the ViewTransition object
 * for custom animation control.
 */
export function startViewTransitionSafe(
  updateFn: TransitionCallback,
): ViewTransition | null {
  if (!SUPPORTS_VIEW_TRANSITIONS) {
    updateFn();
    return null;
  }

  return document.startViewTransition(() => updateFn());
}
```

**Why good:** Named export for tree-shaking, type-safe, async support, returns transition object for advanced use cases

### Bad Example - No Feature Detection

```typescript
// Bad - will crash in unsupported browsers
function updatePage(content: string): void {
  document.startViewTransition(() => {
    document.body.innerHTML = content;
  });
}
```

**Why bad:** No feature detection causes runtime errors in Firefox < 144, Safari < 18, older browsers

---

## Pattern 2: Basic State Transition

### Good Example - Toggle with Transition

```typescript
type ViewState = "list" | "detail";

let currentView: ViewState = "list";

function setView(view: ViewState): void {
  currentView = view;
  renderView();
}

function renderView(): void {
  const container = document.getElementById("app");
  if (!container) return;

  if (currentView === "list") {
    container.innerHTML = "<ul>...</ul>";
  } else {
    container.innerHTML = "<article>...</article>";
  }
}

function navigateTo(view: ViewState): void {
  if (!document.startViewTransition) {
    setView(view);
    return;
  }

  document.startViewTransition(() => {
    setView(view);
  });
}

// Usage
document.getElementById("show-detail")?.addEventListener("click", () => {
  navigateTo("detail");
});

document.getElementById("back-to-list")?.addEventListener("click", () => {
  navigateTo("list");
});
```

**Why good:** Feature detection with fallback, clear separation of state and render

---

## Pattern 3: Async Data Loading Transition

### Good Example - Fetch with Transition

```typescript
interface User {
  id: string;
  name: string;
  email: string;
}

async function loadUserWithTransition(userId: string): Promise<void> {
  const updateFn = async () => {
    // Show loading state
    showLoadingState();

    try {
      const response = await fetch(`/api/users/${userId}`);
      const user: User = await response.json();
      renderUserProfile(user);
    } catch (error) {
      renderError("Failed to load user");
    }
  };

  if (!document.startViewTransition) {
    await updateFn();
    return;
  }

  // Start transition with async callback
  const transition = document.startViewTransition(async () => {
    await updateFn();
  });

  // Handle transition errors separately
  transition.finished.catch((error) => {
    console.warn("View transition failed:", error);
  });
}

function showLoadingState(): void {
  const container = document.getElementById("user-profile");
  if (container) {
    container.innerHTML = '<div aria-busy="true">Loading...</div>';
  }
}

function renderUserProfile(user: User): void {
  const container = document.getElementById("user-profile");
  if (container) {
    container.innerHTML = `
      <article>
        <h1>${user.name}</h1>
        <p>${user.email}</p>
      </article>
    `;
  }
}

function renderError(message: string): void {
  const container = document.getElementById("user-profile");
  if (container) {
    container.innerHTML = `<div role="alert">${message}</div>`;
  }
}
```

**Why good:** Handles async operations inside transition callback, error handling for both fetch and transition, accessible loading state

---

## Pattern 4: Transition Promise Handling

### Good Example - Using ViewTransition Promises

```typescript
async function demonstrateTransitionPromises(
  updateFn: () => void,
): Promise<void> {
  if (!document.startViewTransition) {
    updateFn();
    return;
  }

  const transition = document.startViewTransition(updateFn);

  // ready: Pseudo-element tree is created, animation about to start
  transition.ready
    .then(() => {
      console.log("Pseudo-elements ready - safe to add custom animations");
    })
    .catch((error) => {
      console.warn("Transition rejected before ready:", error);
    });

  // updateCallbackDone: DOM update callback completed
  transition.updateCallbackDone
    .then(() => {
      console.log("DOM update complete");
    })
    .catch((error) => {
      console.error("DOM update failed:", error);
    });

  // finished: Transition complete, new view visible
  await transition.finished;
  console.log("Transition finished - cleanup safe");
}
```

**Why good:** Shows all three promise stages, proper error handling for each

### Good Example - Custom Animation After Ready

```typescript
const CUSTOM_DURATION_MS = 400;
const CUSTOM_EASING = "cubic-bezier(0.4, 0, 0.2, 1)";

async function transitionWithCustomAnimation(
  updateFn: () => void,
): Promise<void> {
  if (!document.startViewTransition) {
    updateFn();
    return;
  }

  const transition = document.startViewTransition(updateFn);

  // Wait for pseudo-elements to exist
  await transition.ready;

  // Apply custom animation via Web Animations API
  document.documentElement.animate(
    {
      opacity: [0.8, 1],
      transform: ["scale(0.98)", "scale(1)"],
    },
    {
      duration: CUSTOM_DURATION_MS,
      easing: CUSTOM_EASING,
      pseudoElement: "::view-transition-new(root)",
    },
  );

  await transition.finished;
}
```

**Why good:** Waits for ready before animating, uses named constants, applies to correct pseudo-element

---

## Pattern 5: Skipping Transition

### Good Example - Conditional Transition Skip

```typescript
interface TransitionOptions {
  skipTransition?: boolean;
}

function updateWithOptions(
  updateFn: () => void,
  options: TransitionOptions = {},
): ViewTransition | null {
  // Skip if requested or not supported
  if (options.skipTransition || !document.startViewTransition) {
    updateFn();
    return null;
  }

  const transition = document.startViewTransition(updateFn);

  return transition;
}

// Skip transition programmatically
function skipTransition(transition: ViewTransition | null): void {
  if (transition) {
    transition.skipTransition();
  }
}

// Usage - skip on rapid interactions
let pendingTransition: ViewTransition | null = null;

function handleRapidNavigation(page: string): void {
  // Skip previous transition if still running
  skipTransition(pendingTransition);

  pendingTransition = updateWithOptions(() => {
    renderPage(page);
  });
}

function renderPage(page: string): void {
  // Page rendering logic
}
```

**Why good:** Allows programmatic skip, handles rapid interactions gracefully

---

## Pattern 6: Transition CSS Customization

### Good Example - Basic CSS Animation Override

```css
/* Timing constants as CSS custom properties */
:root {
  --vt-duration: 300ms;
  --vt-easing-enter: ease-out;
  --vt-easing-exit: ease-in;
}

/* Default cross-fade with custom timing */
::view-transition-old(root),
::view-transition-new(root) {
  animation-duration: var(--vt-duration);
}

/* Asymmetric timing - exit faster than enter */
::view-transition-old(root) {
  animation-timing-function: var(--vt-easing-exit);
  animation-duration: calc(var(--vt-duration) * 0.8);
}

::view-transition-new(root) {
  animation-timing-function: var(--vt-easing-enter);
}
```

**Why good:** CSS custom properties for maintainable timing values, asymmetric timing for polish

### Good Example - Disable Animation for Specific Elements

```css
/* Exclude header from transition animation */
.site-header {
  view-transition-name: header;
}

::view-transition-old(header),
::view-transition-new(header) {
  animation: none;
}

/* Header stays static while content animates */
::view-transition-group(header) {
  z-index: 100;
}
```

**Why good:** Named element excluded from animation, z-index ensures proper layering
