/**
 * Tests for extension detection hook
 */

import { renderHook, waitFor, act } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import { useExtensionDetection } from './use-extension-detection';

describe('useExtensionDetection hook', () => {
  beforeEach(() => {
    // Clean up window.bodhiext before each test
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
      // Mock extension with getExtensionId method
      const mockExtensionId = 'test-extension-id-12345';
      (window as any).bodhiext = {
        getExtensionId: vi.fn().mockResolvedValue(mockExtensionId),
      };

      const { result } = renderHook(() => useExtensionDetection());

      // Wait for the initial check to complete
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
      // No window.bodhiext object
      const { result } = renderHook(() => useExtensionDetection());

      // Wait for the initial check to complete
      await waitFor(
        () => {
          expect(result.current.status).toBe('not-installed');
        },
        { timeout: 1000 }
      );

      expect(result.current.extensionId).toBeNull();
    });

    it('handles getExtensionId error gracefully', async () => {
      // Mock extension with failing getExtensionId method
      (window as any).bodhiext = {
        getExtensionId: vi.fn().mockRejectedValue(new Error('Extension error')),
      };

      const { result } = renderHook(() => useExtensionDetection());

      // Wait for the initial check to complete
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

      // Initially not installed
      await waitFor(() => {
        expect(result.current.status).toBe('not-installed');
      });

      // Dispatch initialization event
      const mockExtensionId = 'event-extension-id-67890';
      const initEvent = new CustomEvent('bodhiext:initialized', {
        detail: { extensionId: mockExtensionId },
      });

      window.dispatchEvent(initEvent);

      // Wait for event handler to process
      await waitFor(() => {
        expect(result.current.status).toBe('installed');
        expect(result.current.extensionId).toBe(mockExtensionId);
      });
    });

    it('ignores event without extensionId', async () => {
      const { result } = renderHook(() => useExtensionDetection());

      // Initially not installed
      await waitFor(() => {
        expect(result.current.status).toBe('not-installed');
      });

      // Dispatch event without extensionId
      const invalidEvent = new CustomEvent('bodhiext:initialized', {
        detail: {},
      });

      window.dispatchEvent(invalidEvent);

      // Should remain not-installed
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

      // Wait for initial check
      await waitFor(() => {
        expect(result.current.status).toBe('not-installed');
      });

      // Add extension after initial check
      const mockExtensionId = 'redetect-extension-id';
      (window as any).bodhiext = {
        getExtensionId: vi.fn().mockResolvedValue(mockExtensionId),
      };

      // Call redetect wrapped in act
      await act(async () => {
        result.current.redetect();
      });

      // Should go to detecting state first
      expect(result.current.status).toBe('detecting');

      // Wait for redetection to complete
      await waitFor(() => {
        expect(result.current.status).toBe('installed');
        expect(result.current.extensionId).toBe(mockExtensionId);
      });
    });
  });
});
