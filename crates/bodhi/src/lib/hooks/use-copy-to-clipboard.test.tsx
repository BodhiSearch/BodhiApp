import { renderHook, act } from '@testing-library/react';
import { useCopyToClipboard } from '@/lib/hooks/use-copy-to-clipboard';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

describe('useCopyToClipboard', () => {
  const originalClipboard = navigator.clipboard;
  const mockClipboard = {
    writeText: vi.fn()
  };

  beforeEach(() => {
    Object.defineProperty(navigator, 'clipboard', {
      value: mockClipboard,
      writable: true
    });
    vi.useFakeTimers();
  });

  afterEach(() => {
    Object.defineProperty(navigator, 'clipboard', {
      value: originalClipboard,
      writable: true
    });
    vi.clearAllMocks();
    vi.useRealTimers();
  });

  it('should copy text to clipboard', async () => {
    mockClipboard.writeText.mockResolvedValueOnce(undefined);
    const { result } = renderHook(() => useCopyToClipboard());

    await act(async () => {
      await result.current.copyToClipboard('test text');
    });

    expect(mockClipboard.writeText).toHaveBeenCalledWith('test text');
    expect(result.current.isCopied).toBe(true);
  });

  it('should reset isCopied after timeout', async () => {
    mockClipboard.writeText.mockResolvedValueOnce(undefined);
    const { result } = renderHook(() => useCopyToClipboard({ timeout: 1000 }));

    await act(async () => {
      await result.current.copyToClipboard('test text');
    });

    expect(result.current.isCopied).toBe(true);

    act(() => {
      vi.advanceTimersByTime(1000);
    });

    expect(result.current.isCopied).toBe(false);
  });

  it('should handle clipboard errors', async () => {
    const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => { });
    mockClipboard.writeText.mockRejectedValueOnce(new Error('Clipboard error'));

    const { result } = renderHook(() => useCopyToClipboard());

    await act(async () => {
      await result.current.copyToClipboard('test text');
    });

    expect(consoleWarnSpy).toHaveBeenCalledWith('Copy failed', expect.any(Error));
    expect(result.current.isCopied).toBe(false);

    consoleWarnSpy.mockRestore();
  });

  it('should handle missing clipboard API', async () => {
    const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => { });
    Object.defineProperty(navigator, 'clipboard', {
      value: undefined,
      writable: true
    });

    const { result } = renderHook(() => useCopyToClipboard());

    await act(async () => {
      await result.current.copyToClipboard('test text');
    });

    expect(consoleWarnSpy).toHaveBeenCalledWith('Clipboard not supported');
    expect(result.current.isCopied).toBe(false);

    consoleWarnSpy.mockRestore();
  });
}); 