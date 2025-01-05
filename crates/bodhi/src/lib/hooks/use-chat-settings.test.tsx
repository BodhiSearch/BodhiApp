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

  it('should initialize with default settings', () => {
    const { result } = renderHook(() => useChatSettings(), { wrapper });

    expect(result.current.model).toBe('');
    expect(result.current.temperature).toBe(0.7);
    expect(result.current.max_tokens).toBe(2048);
    expect(result.current.stop).toEqual([]);
    expect(result.current.top_p).toBe(1);
    expect(result.current.n).toBe(1);
    expect(result.current.stream).toBe(true);
    expect(result.current.presence_penalty).toBe(0);
    expect(result.current.frequency_penalty).toBe(0);
    expect(result.current.logit_bias).toEqual({});
  });

  it('should update model', () => {
    const { result } = renderHook(() => useChatSettings(), { wrapper });

    act(() => {
      result.current.setModel('gpt-4');
    });

    expect(result.current.model).toBe('gpt-4');
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

  it('should persist settings in localStorage', () => {
    const { result } = renderHook(() => useChatSettings(), { wrapper });

    act(() => {
      result.current.setModel('gpt-4');
      result.current.setTemperature(0.9);
    });

    const saved = JSON.parse(localStorage.getItem('chat-settings') || '{}');
    expect(saved.model).toBe('gpt-4');
    expect(saved.temperature).toBe(0.9);
  });

  it('should load settings from localStorage', () => {
    const initialSettings = {
      model: 'gpt-4',
      temperature: 0.8,
      top_p: 0.9,
      n: 2,
      stream: false,
      max_tokens: 1000,
      presence_penalty: 0.1,
      frequency_penalty: 0.2,
      logit_bias: {},
      stop: ['stop']
    };
    
    localStorage.setItem('chat-settings', JSON.stringify(initialSettings));
    
    const { result } = renderHook(() => useChatSettings(), { wrapper });
    
    expect(result.current.model).toBe('gpt-4');
    expect(result.current.temperature).toBe(0.8);
    expect(result.current.top_p).toBe(0.9);
    expect(result.current.n).toBe(2);
    expect(result.current.stream).toBe(false);
  });

  it('should reset to default settings', () => {
    const { result } = renderHook(() => useChatSettings(), { wrapper });

    act(() => {
      result.current.setModel('gpt-4');
      result.current.setTemperature(0.9);
      result.current.setTopP(0.8);
      result.current.reset();
    });

    expect(result.current.model).toBe('');
    expect(result.current.temperature).toBe(0.7);
    expect(result.current.top_p).toBe(1);
    expect(result.current.n).toBe(1);
    expect(result.current.stream).toBe(true);
  });

  it('should update response format', () => {
    const { result } = renderHook(() => useChatSettings(), { wrapper });

    act(() => {
      result.current.setResponseFormat({ type: 'json_object' });
    });

    expect(result.current.response_format).toEqual({ type: 'json_object' });
  });
}); 