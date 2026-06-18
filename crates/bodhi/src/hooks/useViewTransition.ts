import { useCallback } from 'react';

/**
 * React-18-safe wrapper over the native View Transitions API
 * (`document.startViewTransition`). This is the same browser API React 19's
 * `<ViewTransition>` wraps — so we get smooth state transitions without a React
 * upgrade. Route-level transitions are handled separately by TanStack Router's
 * `defaultViewTransition` (see `main.tsx`); this hook is for in-page state
 * changes (e.g. opening/closing the detail rail, swapping a filtered list).
 *
 * Guarantees (per the web-animation-view-transitions skill):
 * - Feature-detected: falls back to a plain synchronous update on unsupported
 *   browsers (Chromium <111, Firefox <144, Safari <18.2).
 * - Honors `prefers-reduced-motion`: skips the transition entirely so motion-
 *   sensitive users get an instant update (CSS also guards via a media query).
 * The actual animation is CSS-driven via `::view-transition-*` pseudo-elements
 * in `styles/view-transitions.css`.
 */

type UpdateFn = () => void;

const supportsViewTransitions = (): boolean => typeof document !== 'undefined' && 'startViewTransition' in document;

const prefersReducedMotion = (): boolean =>
  typeof window !== 'undefined' &&
  typeof window.matchMedia === 'function' &&
  window.matchMedia('(prefers-reduced-motion: reduce)').matches;

interface DocWithViewTransition {
  startViewTransition(callback: UpdateFn): { ready: Promise<void>; finished: Promise<void> };
}

export function startViewTransition(updateFn: UpdateFn): void {
  if (!supportsViewTransitions() || prefersReducedMotion()) {
    updateFn();
    return;
  }
  try {
    // Must be invoked as a method of `document` — calling an extracted reference
    // throws "Illegal invocation".
    const transition = (document as unknown as DocWithViewTransition).startViewTransition(updateFn);
    // Swallow both promises' rejections. When a transition is interrupted by another
    // (e.g. the router-level navigation cross-fade overlapping an in-page rail toggle),
    // `ready` rejects asynchronously with InvalidStateError and `finished` may also reject —
    // the DOM is still updated correctly, so neither should surface as an uncaught error.
    transition.ready.catch(() => {});
    transition.finished.catch(() => {});
  } catch {
    // startViewTransition throws synchronously (InvalidStateError) when another
    // transition — e.g. the router-level navigation cross-fade — is mid-flight.
    // Apply the update directly so selection/rail state still changes.
    updateFn();
  }
}

/**
 * Returns a stable callback that runs `updateFn` inside a view transition
 * when supported, or synchronously otherwise. Use for state setters whose
 * resulting DOM change should animate (rail open/close, filter swap).
 */
export function useViewTransition(): (updateFn: UpdateFn) => void {
  return useCallback((updateFn: UpdateFn) => startViewTransition(updateFn), []);
}
