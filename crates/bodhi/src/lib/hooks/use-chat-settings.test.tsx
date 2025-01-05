import { renderHook, act } from '@testing-library/react';
import { useChatSettings, ChatSettingsProvider } from './use-chat-settings';
import { beforeEach, describe, expect, it } from 'vitest';

describe('useChatSettings', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <ChatSettingsProvider>{children}</ChatSettingsProvider>
  );

  describe('initialization and persistence', () => {
    it('should initialize with only model in default settings', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      expect(result.current).toMatchObject({
        model: ''
      });
      // Verify other properties are undefined
      expect(result.current.temperature).toBeUndefined();
      expect(result.current.stream).toBeUndefined();
      expect(result.current.max_tokens).toBeUndefined();
    });

    it('should allow setting and removing optional settings', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      act(() => {
        result.current.setTemperature(0.7);
        result.current.setStream(true);
      });

      expect(result.current.temperature).toBe(0.7);
      expect(result.current.stream).toBe(true);

      act(() => {
        result.current.setTemperature(undefined);
        result.current.setStream(undefined);
      });

      expect(result.current.temperature).toBeUndefined();
      expect(result.current.stream).toBeUndefined();
    });

    it('should only include defined values in request settings', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      act(() => {
        result.current.setModel('gpt-4');
        result.current.setTemperature(0.7);
        result.current.setStream(undefined);
      });

      const requestSettings = result.current.getRequestSettings();
      expect(requestSettings).toEqual({
        model: 'gpt-4',
        temperature: 0.7
      });
      expect(requestSettings).not.toHaveProperty('stream');
    });
  });

  describe('settings updates', () => {
    it('should update basic settings', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      act(() => {
        result.current.setModel('gpt-4');
        result.current.setTemperature(0.9);
        result.current.setResponseFormat({ type: 'json_object' });
        result.current.setStream(false);
      });

      expect(result.current.model).toBe('gpt-4');
      expect(result.current.temperature).toBe(0.9);
      expect(result.current.response_format).toEqual({ type: 'json_object' });
      expect(result.current.stream).toBe(false);
    });

    it('should update advanced settings', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      act(() => {
        result.current.setTopP(0.8);
        result.current.setN(2);
        result.current.setStream(false);
        result.current.setStop(['stop1', 'stop2']);
        result.current.setLogitBias({ '123': 1 });
      });

      expect(result.current.top_p).toBe(0.8);
      expect(result.current.n).toBe(2);
      expect(result.current.stream).toBe(false);
      expect(result.current.stop).toEqual(['stop1', 'stop2']);
      expect(result.current.logit_bias).toEqual({ '123': 1 });
    });
  });

  describe('system prompt and request settings', () => {
    it('should manage system prompt separately from request settings', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      act(() => {
        result.current.setModel('gpt-4');
        result.current.setSystemPrompt('Test prompt');
      });

      expect(result.current.systemPrompt).toBe('Test prompt');
      
      const requestSettings = result.current.getRequestSettings();
      expect(requestSettings.model).toBe('gpt-4');
      expect(requestSettings).not.toHaveProperty('systemPrompt');
    });
  });

  describe('reset functionality', () => {
    it('should reset all settings to defaults', () => {
      const { result } = renderHook(() => useChatSettings(), { wrapper });

      act(() => {
        result.current.setModel('gpt-4');
        result.current.setTemperature(0.9);
        result.current.setSystemPrompt('Test prompt');
        result.current.reset();
      });

      expect(result.current.model).toBe('');
      expect(result.current.temperature).toBeUndefined();
      expect(result.current.systemPrompt).toBeUndefined();
    });
  });
}); 