# SPA View Transitions

> Same-document transition patterns for single-page applications. See [core.md](core.md) for basic patterns.

**Prerequisites**: Understand feature detection from [core.md](core.md) first.

---

## Theme Switcher with Circular Reveal

### Good Example - Click-Origin Circular Reveal

```typescript
// theme-transition.ts

interface ClickPosition {
  x: number;
  y: number;
}

// Track last click position for reveal origin
let lastClickPosition: ClickPosition = { x: 0, y: 0 };

document.addEventListener("click", (e: MouseEvent) => {
  lastClickPosition = { x: e.clientX, y: e.clientY };
});

// Animation constants
const REVEAL_DURATION_MS = 400;
const REVEAL_EASING = "ease-in-out";

export async function toggleThemeWithReveal(): Promise<void> {
  if (!document.startViewTransition) {
    toggleThemeClass();
    return;
  }

  const { x, y } = lastClickPosition;

  // Calculate radius to cover entire viewport
  const endRadius = Math.hypot(
    Math.max(x, window.innerWidth - x),
    Math.max(y, window.innerHeight - y),
  );

  const transition = document.startViewTransition(() => {
    toggleThemeClass();
  });

  await transition.ready;

  // Determine animation direction based on theme
  const isDark = document.documentElement.classList.contains("dark");
  const clipPath = [
    `circle(0 at ${x}px ${y}px)`,
    `circle(${endRadius}px at ${x}px ${y}px)`,
  ];

  document.documentElement.animate(
    { clipPath: isDark ? clipPath : [...clipPath].reverse() },
    {
      duration: REVEAL_DURATION_MS,
      easing: REVEAL_EASING,
      pseudoElement: "::view-transition-new(root)",
    },
  );
}

function toggleThemeClass(): void {
  document.documentElement.classList.toggle("dark");
}
```

**Supporting CSS:**

```css
/* Required for clip-path animation */
::view-transition-image-pair(root) {
  isolation: auto;
}

::view-transition-old(root),
::view-transition-new(root) {
  animation: none;
  mix-blend-mode: normal;
  display: block;
}
```

**Why good:** Calculates reveal radius from click position, reverses animation based on theme direction, disables default animations for custom control

---

## Multi-Step Form Transitions

### Good Example - Form Step Navigation

```typescript
// form-transitions.ts

const TOTAL_STEPS = 4;

type FormStep = 1 | 2 | 3 | 4;

interface FormState {
  currentStep: FormStep;
  direction: "forwards" | "backwards";
}

let formState: FormState = {
  currentStep: 1,
  direction: "forwards",
};

function getStepElement(step: FormStep): HTMLElement | null {
  return document.querySelector(`[data-form-step="${step}"]`);
}

function showStep(step: FormStep): void {
  // Hide all steps
  document.querySelectorAll("[data-form-step]").forEach((el) => {
    (el as HTMLElement).hidden = true;
  });

  // Show target step
  const stepElement = getStepElement(step);
  if (stepElement) {
    stepElement.hidden = false;
  }
}

export async function goToStep(step: FormStep): Promise<void> {
  if (step < 1 || step > TOTAL_STEPS) return;
  if (step === formState.currentStep) return;

  // Determine direction
  formState.direction = step > formState.currentStep ? "forwards" : "backwards";

  const updateFn = () => {
    formState.currentStep = step;
    showStep(step);
  };

  if (!document.startViewTransition) {
    updateFn();
    return;
  }

  // Set direction type for CSS targeting via :active-view-transition-type()
  const transition = document.startViewTransition({
    update: updateFn,
    types: [formState.direction],
  });

  await transition.finished;
}

export function nextStep(): void {
  if (formState.currentStep < TOTAL_STEPS) {
    goToStep((formState.currentStep + 1) as FormStep);
  }
}

export function prevStep(): void {
  if (formState.currentStep > 1) {
    goToStep((formState.currentStep - 1) as FormStep);
  }
}
```

**CSS for directional transitions:**

```css
:root {
  --form-slide-distance: 20px;
  --form-slide-duration: 250ms;
}

[data-form-step] {
  view-transition-name: form-step;
}

/* Forward navigation */
html:active-view-transition-type(forwards) {
  &::view-transition-old(form-step) {
    animation: slide-out-left var(--form-slide-duration) ease-in;
  }
  &::view-transition-new(form-step) {
    animation: slide-in-right var(--form-slide-duration) ease-out;
  }
}

/* Backward navigation */
html:active-view-transition-type(backwards) {
  &::view-transition-old(form-step) {
    animation: slide-out-right var(--form-slide-duration) ease-in;
  }
  &::view-transition-new(form-step) {
    animation: slide-in-left var(--form-slide-duration) ease-out;
  }
}

@keyframes slide-out-left {
  to {
    transform: translateX(calc(-1 * var(--form-slide-distance)));
    opacity: 0;
  }
}

@keyframes slide-in-right {
  from {
    transform: translateX(var(--form-slide-distance));
    opacity: 0;
  }
}

@keyframes slide-out-right {
  to {
    transform: translateX(var(--form-slide-distance));
    opacity: 0;
  }
}

@keyframes slide-in-left {
  from {
    transform: translateX(calc(-1 * var(--form-slide-distance)));
    opacity: 0;
  }
}
```

**Why good:** Direction-aware transitions, CSS custom properties for values, proper step validation

---

## Tab Panel Transitions

### Good Example - Tab Content Transitions

```typescript
// tab-transitions.ts

interface Tab {
  id: string;
  label: string;
}

let activeTabId: string = "";

function setActiveTab(tabId: string): void {
  activeTabId = tabId;

  // Update tab button states
  document.querySelectorAll('[role="tab"]').forEach((tab) => {
    const isActive = tab.getAttribute("data-tab-id") === tabId;
    tab.setAttribute("aria-selected", String(isActive));
  });

  // Update tab panel visibility
  document.querySelectorAll('[role="tabpanel"]').forEach((panel) => {
    const isActive = panel.getAttribute("data-tab-id") === tabId;
    (panel as HTMLElement).hidden = !isActive;
  });
}

export function switchTab(tabId: string): void {
  if (tabId === activeTabId) return;

  if (!document.startViewTransition) {
    setActiveTab(tabId);
    return;
  }

  document.startViewTransition(() => {
    setActiveTab(tabId);
  });
}
```

**CSS:**

```css
:root {
  --tab-duration: 200ms;
}

[role="tabpanel"] {
  view-transition-name: tab-content;
}

::view-transition-old(tab-content) {
  animation: fade-out var(--tab-duration) ease-out;
}

::view-transition-new(tab-content) {
  animation: fade-in var(--tab-duration) ease-in;
}

@keyframes fade-out {
  to {
    opacity: 0;
  }
}

@keyframes fade-in {
  from {
    opacity: 0;
  }
}
```

**Why good:** ARIA-compliant tab implementation, simple fade transition for content

---

## Accordion Transitions

### Good Example - Expanding Sections

```typescript
// accordion-transitions.ts

function toggleAccordionSection(triggerId: string): void {
  const trigger = document.getElementById(triggerId);
  const panelId = trigger?.getAttribute("aria-controls");
  const panel = panelId ? document.getElementById(panelId) : null;

  if (!trigger || !panel) return;

  const isExpanded = trigger.getAttribute("aria-expanded") === "true";

  const updateFn = () => {
    trigger.setAttribute("aria-expanded", String(!isExpanded));
    panel.hidden = isExpanded;
  };

  if (!document.startViewTransition) {
    updateFn();
    return;
  }

  // Give the panel a unique transition name while animating
  panel.style.viewTransitionName = `accordion-${panelId}`;

  const transition = document.startViewTransition(updateFn);

  transition.finished.then(() => {
    // Clean up transition name
    panel.style.viewTransitionName = "";
  });
}
```

**CSS:**

```css
:root {
  --accordion-duration: 300ms;
}

/* Dynamic transition names are set via JavaScript */
[id^="accordion-panel-"] {
  overflow: hidden;
}

::view-transition-old(*),
::view-transition-new(*) {
  animation-duration: var(--accordion-duration);
}
```

**Why good:** Dynamic transition names prevent conflicts, cleanup after transition, accessible ARIA pattern

---

## List Reordering Transitions

### Good Example - Animated List Sort

```typescript
// list-transitions.ts

interface ListItem {
  id: string;
  label: string;
  order: number;
}

let items: ListItem[] = [];

function renderList(): void {
  const container = document.getElementById("sortable-list");
  if (!container) return;

  // Sort by order
  const sortedItems = [...items].sort((a, b) => a.order - b.order);

  container.innerHTML = sortedItems
    .map(
      (item) => `
      <li data-item-id="${item.id}" style="view-transition-name: item-${item.id}">
        ${item.label}
      </li>
    `,
    )
    .join("");
}

export function sortListBy(
  compareFn: (a: ListItem, b: ListItem) => number,
): void {
  const updateFn = () => {
    items = [...items].sort(compareFn);
    items.forEach((item, index) => {
      item.order = index;
    });
    renderList();
  };

  if (!document.startViewTransition) {
    updateFn();
    return;
  }

  document.startViewTransition(updateFn);
}

// Usage
export function sortAlphabetically(): void {
  sortListBy((a, b) => a.label.localeCompare(b.label));
}

export function sortById(): void {
  sortListBy((a, b) => a.id.localeCompare(b.id));
}
```

**CSS:**

```css
:root {
  --list-reorder-duration: 300ms;
}

/* Each item has unique view-transition-name via inline style */
#sortable-list li {
  /* view-transition-name is set inline */
}

/* Animate position changes */
::view-transition-group(*) {
  animation-duration: var(--list-reorder-duration);
  animation-timing-function: ease-out;
}
```

**Why good:** Each item gets unique transition name for FLIP animation, sorts without remounting elements

---

## Reduced Motion Fallback

### Good Example - Accessible SPA Transitions

```typescript
// accessible-transitions.ts

const REDUCED_MOTION_QUERY = "(prefers-reduced-motion: reduce)";

interface TransitionConfig {
  duration: number;
  reducedDuration: number;
}

const DEFAULT_CONFIG: TransitionConfig = {
  duration: 300,
  reducedDuration: 100,
};

function prefersReducedMotion(): boolean {
  return window.matchMedia(REDUCED_MOTION_QUERY).matches;
}

export function getTransitionDuration(config = DEFAULT_CONFIG): number {
  return prefersReducedMotion() ? config.reducedDuration : config.duration;
}

export async function accessibleTransition(
  updateFn: () => void,
  config = DEFAULT_CONFIG,
): Promise<void> {
  // Skip transition entirely for reduced motion if preferred
  if (prefersReducedMotion()) {
    updateFn();
    return;
  }

  if (!document.startViewTransition) {
    updateFn();
    return;
  }

  const transition = document.startViewTransition(updateFn);
  await transition.finished;
}

// Monitor for preference changes
export function onReducedMotionChange(
  callback: (prefers: boolean) => void,
): () => void {
  const query = window.matchMedia(REDUCED_MOTION_QUERY);

  const handler = (e: MediaQueryListEvent) => callback(e.matches);
  query.addEventListener("change", handler);

  // Return cleanup function
  return () => query.removeEventListener("change", handler);
}
```

**CSS:**

```css
@media (prefers-reduced-motion: reduce) {
  ::view-transition-group(*),
  ::view-transition-old(*),
  ::view-transition-new(*) {
    animation-duration: 0.01ms !important;
  }
}
```

**Why good:** JavaScript and CSS both handle reduced motion, returns cleanup function for subscriptions
