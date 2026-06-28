import { renderHook, waitFor, act } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';

import { useExtensionDetection } from './useExtensionDetection';

describe('useExtensionDetection hook', () => {
  beforeEach(() => {
    delete (window as any).bodhiext;
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('extension detection states', () => {
    it('starts in detecting state', () => {
      const { result } = renderHook(() => useExtensionDetection());

      expect(result.current.status).toBe('detecting');
      expect(result.current.extensionId).toBeNull();
    });

    it('detects installed extension with ID', async () => {
      const mockExtensionId = 'test-extension-id-12345';
      (window as any).bodhiext = {
        getExtensionId: vi.fn().mockResolvedValue(mockExtensionId),
      };

      const { result } = renderHook(() => useExtensionDetection());

      await waitFor(
        () => {
          expect(result.current.status).toBe('installed');
        },
        { timeout: 1000 }
      );

      expect(result.current.extensionId).toBe(mockExtensionId);
      expect((window as any).bodhiext.getExtensionId).toHaveBeenCalled();
    });

    it('handles extension not installed', async () => {
      const { result } = renderHook(() => useExtensionDetection());

      await waitFor(
        () => {
          expect(result.current.status).toBe('not-installed');
        },
        { timeout: 1000 }
      );

      expect(result.current.extensionId).toBeNull();
    });

    it('handles getExtensionId error gracefully', async () => {
      (window as any).bodhiext = {
        getExtensionId: vi.fn().mockRejectedValue(new Error('Extension error')),
      };

      const { result } = renderHook(() => useExtensionDetection());

      await waitFor(
        () => {
          expect(result.current.status).toBe('not-installed');
        },
        { timeout: 1000 }
      );

      expect(result.current.extensionId).toBeNull();
    });
  });

  describe('extension initialization event', () => {
    it('listens for bodhiext:initialized event', async () => {
      const { result } = renderHook(() => useExtensionDetection());

      await waitFor(() => {
        expect(result.current.status).toBe('not-installed');
      });

      const mockExtensionId = 'event-extension-id-67890';
      const initEvent = new CustomEvent('bodhiext:initialized', {
        detail: { extensionId: mockExtensionId },
      });

      window.dispatchEvent(initEvent);

      await waitFor(() => {
        expect(result.current.status).toBe('installed');
        expect(result.current.extensionId).toBe(mockExtensionId);
      });
    });

    it('ignores event without extensionId', async () => {
      const { result } = renderHook(() => useExtensionDetection());

      await waitFor(() => {
        expect(result.current.status).toBe('not-installed');
      });

      const invalidEvent = new CustomEvent('bodhiext:initialized', {
        detail: {},
      });

      window.dispatchEvent(invalidEvent);

      expect(result.current.status).toBe('not-installed');
      expect(result.current.extensionId).toBeNull();
    });
  });

  describe('utility functions', () => {
    it('refresh function is available', () => {
      const { result } = renderHook(() => useExtensionDetection());

      expect(typeof result.current.refresh).toBe('function');
      // Note: We can't easily test window.location.reload in jsdom,
      // but we can verify the function is available
    });

    it('redetect re-checks extension status', async () => {
      const { result } = renderHook(() => useExtensionDetection());

      await waitFor(() => {
        expect(result.current.status).toBe('not-installed');
      });

      const mockExtensionId = 'redetect-extension-id';
      (window as any).bodhiext = {
        getExtensionId: vi.fn().mockResolvedValue(mockExtensionId),
      };

      await act(async () => {
        result.current.redetect();
      });

      expect(result.current.status).toBe('detecting');

      await waitFor(() => {
        expect(result.current.status).toBe('installed');
        expect(result.current.extensionId).toBe(mockExtensionId);
      });
    });
  });
});
