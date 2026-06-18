import { afterEach, describe, expect, it, vi } from 'vitest';

import { startViewTransition } from '@/hooks/useViewTransition';

describe('startViewTransition', () => {
  afterEach(() => {
    vi.restoreAllMocks();
    // remove any stub we added
    delete (document as unknown as { startViewTransition?: unknown }).startViewTransition;
  });

  it('falls back to a synchronous update when the API is unavailable', () => {
    // jsdom has no startViewTransition
    const update = vi.fn();
    startViewTransition(update);
    expect(update).toHaveBeenCalledTimes(1);
  });

  it('invokes document.startViewTransition with correct `this` binding (no Illegal invocation)', () => {
    // Stub the API and assert it's called as a method of `document`. A regression to
    // an unbound call (`const f = document.startViewTransition; f(cb)`) throws
    // "Illegal invocation" in real browsers — this guards against that.
    vi.spyOn(window, 'matchMedia').mockReturnValue({ matches: false } as MediaQueryList);
    const calls: { thisArg: unknown }[] = [];
    const stub = vi.fn(function (this: unknown, cb: () => void) {
      calls.push({ thisArg: this });
      cb();
      return { ready: Promise.resolve(), finished: Promise.resolve() };
    });
    (document as unknown as { startViewTransition: unknown }).startViewTransition = stub;

    const update = vi.fn();
    startViewTransition(update);

    expect(stub).toHaveBeenCalledTimes(1);
    expect(update).toHaveBeenCalledTimes(1);
    // called as document.startViewTransition(...) → `this` is the document
    expect(calls[0].thisArg).toBe(document);
  });

  it('swallows an async ready/finished rejection from an interrupted transition', async () => {
    // When the router-level navigation cross-fade interrupts an in-page transition, `ready`
    // rejects asynchronously with InvalidStateError; it must not surface as an unhandled rejection.
    vi.spyOn(window, 'matchMedia').mockReturnValue({ matches: false } as MediaQueryList);
    const rejected = Promise.reject(new DOMException('aborted', 'InvalidStateError'));
    (document as unknown as { startViewTransition: unknown }).startViewTransition = vi.fn((cb: () => void) => {
      cb();
      return { ready: rejected, finished: Promise.reject(new DOMException('aborted', 'InvalidStateError')) };
    });

    const update = vi.fn();
    expect(() => startViewTransition(update)).not.toThrow();
    expect(update).toHaveBeenCalledTimes(1);
    // let the microtask queue flush so the .catch() handlers run
    await Promise.resolve();
    await Promise.resolve();
  });

  it('applies the update when startViewTransition throws synchronously (InvalidStateError)', () => {
    // The browser throws InvalidStateError when another transition (e.g. the router-level
    // navigation cross-fade) is mid-flight; the update must still run.
    vi.spyOn(window, 'matchMedia').mockReturnValue({ matches: false } as MediaQueryList);
    (document as unknown as { startViewTransition: unknown }).startViewTransition = vi.fn(() => {
      throw new DOMException('Transition was aborted because of invalid state', 'InvalidStateError');
    });

    const update = vi.fn();
    expect(() => startViewTransition(update)).not.toThrow();
    expect(update).toHaveBeenCalledTimes(1);
  });

  it('skips the transition when prefers-reduced-motion is set, still applying the update', () => {
    (document as unknown as { startViewTransition: unknown }).startViewTransition = vi.fn();
    vi.spyOn(window, 'matchMedia').mockReturnValue({ matches: true } as MediaQueryList);

    const update = vi.fn();
    startViewTransition(update);

    expect(update).toHaveBeenCalledTimes(1);
    expect(
      (document as unknown as { startViewTransition: ReturnType<typeof vi.fn> }).startViewTransition
    ).not.toHaveBeenCalled();
  });

  it('skips the transition on a mobile viewport (<768px), still applying the update', () => {
    // On mobile the rail is a fixed drawer with its own transform transition; a document-level
    // view transition fights it and can leave the drawer from opening — so we skip it.
    (document as unknown as { startViewTransition: unknown }).startViewTransition = vi.fn();
    vi.spyOn(window, 'matchMedia').mockImplementation(
      (query: string) => ({ matches: query.includes('max-width: 767px') }) as MediaQueryList
    );

    const update = vi.fn();
    startViewTransition(update);

    expect(update).toHaveBeenCalledTimes(1);
    expect(
      (document as unknown as { startViewTransition: ReturnType<typeof vi.fn> }).startViewTransition
    ).not.toHaveBeenCalled();
  });
});
