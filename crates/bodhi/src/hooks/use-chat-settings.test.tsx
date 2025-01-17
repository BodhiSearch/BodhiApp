import { ChatSettingsProvider, useChatSettings } from '@/hooks/use-chat-settings';
import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

describe('useChatSettings', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <ChatSettingsProvider>{children}</ChatSettingsProvider>
  );

  describe('initialization', () => {
    it('should initialize with default enabled states', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      expect(result.current).toMatchObject({
        model: '',
        temperature_enabled: false,
        stream_enabled: true,
        max_tokens_enabled: false
        // ... other enabled states
      });
    });
  });

  describe('settings management', () => {
    it('should automatically enable setting when value is set', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      act(() => {
        result.current.setTemperature(0.7);
      });

      expect(result.current.temperature).toBe(0.7);
      expect(result.current.temperature_enabled).toBe(true);
    });

    it('should automatically disable setting when value is set to undefined', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      act(() => {
        result.current.setTemperature(0.7);
      });

      act(() => {
        result.current.setTemperature(undefined);
      });

      expect(result.current.temperature).toBeUndefined();
      expect(result.current.temperature_enabled).toBe(false);
    });

    it('should allow manual enabled state changes without affecting value', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      act(() => {
        result.current.setTemperature(0.7);
      });

      expect(result.current.temperature).toBe(0.7);
      expect(result.current.temperature_enabled).toBe(true);

      act(() => {
        result.current.setTemperatureEnabled(false);
      });

      expect(result.current.temperature).toBe(0.7);
      expect(result.current.temperature_enabled).toBe(false);

      act(() => {
        result.current.setTemperatureEnabled(true);
      });

      expect(result.current.temperature).toBe(0.7);
      expect(result.current.temperature_enabled).toBe(true);
    });
  });

  describe('request settings generation', () => {
    it('should only include enabled settings in request', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      act(() => {
        result.current.setModel('gpt-4');
        result.current.setTemperature(0.7);  // This automatically enables it
        result.current.setTopP(0.9);         // This automatically enables it
        result.current.setTopPEnabled(false); // Manually disable top_p
      });

      const requestSettings = result.current.getRequestSettings();
      expect(requestSettings).toEqual({
        model: 'gpt-4',
        temperature: 0.7,
        stream: true
      });
      expect(requestSettings).not.toHaveProperty('top_p');
    });
  });

  describe('reset functionality', () => {
    it('should reset all settings and enabled states to defaults', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      act(() => {
        result.current.setModel('gpt-4');
        result.current.setTemperature(0.7);
        result.current.setSystemPrompt('Test prompt');
        result.current.reset();
      });

      expect(result.current.model).toBe('');
      expect(result.current.temperature).toBeUndefined();
      expect(result.current.temperature_enabled).toBe(false);
      expect(result.current.systemPrompt).toBeUndefined();
      expect(result.current.systemPrompt_enabled).toBe(false);
    });
  });

  describe('persistence', () => {
    it('should persist settings and enabled states to localStorage', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      act(() => {
        result.current.setModel('gpt-4');
        result.current.setTemperature(0.7);  // Should automatically enable
        result.current.setTopP(0.9);         // Should automatically enable
        result.current.setTopPEnabled(false); // Manually disable
      });

      const saved = JSON.parse(localStorage.getItem('chat-settings') || '{}');
      expect(saved).toMatchObject({
        model: 'gpt-4',
        temperature: 0.7,
        temperature_enabled: true,
        top_p: 0.9,
        top_p_enabled: false
      });
    });

    it('should load persisted settings from localStorage', () => {
      // Setup initial state in localStorage
      localStorage.setItem('chat-settings', JSON.stringify({
        model: 'gpt-4',
        temperature: 0.7,
        temperature_enabled: true,
        top_p: 0.9,
        top_p_enabled: false
      }));

      const { result } = renderHook(() => useChatSettings(), { wrapper });

      // Verify loaded state
      expect(result.current.model).toBe('gpt-4');
      expect(result.current.temperature).toBe(0.7);
      expect(result.current.temperature_enabled).toBe(true);
      expect(result.current.top_p).toBe(0.9);
      expect(result.current.top_p_enabled).toBe(false);
    });

    it('should merge localStorage data with default enabled states', () => {
      // Setup partial state in localStorage
      localStorage.setItem('chat-settings', JSON.stringify({
        model: 'gpt-4',
        temperature: 0.7
        // Note: no enabled states specified
      }));

      const { result } = renderHook(() => useChatSettings(), { wrapper });

      // Verify merged state
      expect(result.current.model).toBe('gpt-4');
      expect(result.current.temperature).toBe(0.7);
      // Should have default enabled state
      expect(result.current.temperature_enabled).toBe(false);
      // Other enabled states should match defaults
      expect(result.current.top_p_enabled).toBe(false);
      expect(result.current.stream_enabled).toBe(true);
    });

    it('should handle invalid localStorage data gracefully', () => {
      // Setup invalid JSON in localStorage
      localStorage.setItem('chat-settings', 'invalid json');

      const { result } = renderHook(() => useChatSettings(), { wrapper });

      // Should fall back to defaults
      expect(result.current.model).toBe('');
      expect(result.current.temperature_enabled).toBe(false);
      expect(result.current.stream_enabled).toBe(true);
    });
  });

  describe('stop parameter handling', () => {
    it('should always store stop as string array', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      // Test with single string
      act(() => {
        result.current.setStop('stop');
      });
      expect(Array.isArray(result.current.stop)).toBe(true);
      expect(result.current.stop).toEqual(['stop']);

      // Test with array
      act(() => {
        result.current.setStop(['stop1', 'stop2']);
      });
      expect(result.current.stop).toEqual(['stop1', 'stop2']);

      // Test with undefined
      act(() => {
        result.current.setStop(undefined);
      });
      expect(result.current.stop).toBeUndefined();
      expect(result.current.stop_enabled).toBe(false);
    });
  });
}); 