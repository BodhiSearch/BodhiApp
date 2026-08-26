import { useCallback } from 'react';

/**
 * React-18-safe wrapper over `document.startViewTransition` (the API React 19's
 * `<ViewTransition>` also wraps). For in-page state changes only — route-level
 * transitions go through TanStack Router's `defaultViewTransition` (main.tsx).
 *
 * Falls back to a synchronous update when unsupported (Chromium <111, Firefox <144,
 * Safari <18.2), under `prefers-reduced-motion`, or on mobile (<768px), where the
 * rail's own `transform`-driven drawer animation would fight a document-level
 * transition. CSS lives in `styles/view-transitions.css`.
 */

type UpdateFn = () => void;

const supportsViewTransitions = (): boolean => typeof document !== 'undefined' && 'startViewTransition' in document;

const matchesMedia = (query: string): boolean =>
  typeof window !== 'undefined' && typeof window.matchMedia === 'function' && window.matchMedia(query).matches;

const prefersReducedMotion = (): boolean => matchesMedia('(prefers-reduced-motion: reduce)');

// Mobile = the shell's drawer breakpoint (shell.css `@media (max-width: 767px)`).
const isMobileViewport = (): boolean => matchesMedia('(max-width: 767px)');

interface DocWithViewTransition {
  startViewTransition(callback: UpdateFn): { ready: Promise<void>; finished: Promise<void> };
}

export function startViewTransition(updateFn: UpdateFn): void {
  if (!supportsViewTransitions() || prefersReducedMotion() || isMobileViewport()) {
    updateFn();
    return;
  }
  try {
    // Must be invoked as a method of `document` — calling an extracted reference
    // throws "Illegal invocation".
    const transition = (document as unknown as DocWithViewTransition).startViewTransition(updateFn);
    // `ready`/`finished` can reject with InvalidStateError when another transition interrupts
    // this one (e.g. router cross-fade); the DOM still updates correctly, so swallow both.
    transition.ready.catch(() => {});
    transition.finished.catch(() => {});
  } catch {
    // Throws synchronously (InvalidStateError) when another transition is mid-flight
    // (e.g. router cross-fade); apply directly so state still changes.
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
