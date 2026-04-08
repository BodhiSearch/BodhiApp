import { SystemPrompt } from '@/routes/chat/-components/settings/SystemPrompt';
import { useChatSettingsStore, defaultSettings } from '@/stores/chatSettingsStore';
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi, beforeEach } from 'vitest';

vi.mock('@/stores/chatStore', () => {
  const { create } = require('zustand');
  return { useChatStore: create(() => ({ getChatSettings: vi.fn() })) };
});

vi.mock('@/stores/chatSettingsStore', async () => {
  const { create } = require('zustand');
  const defaults = {
    model: '',
    apiFormat: 'openai',
    stream: true,
    temperature_enabled: false,
    top_p_enabled: false,
    n_enabled: false,
    stream_enabled: true,
    max_tokens_enabled: false,
    presence_penalty_enabled: false,
    frequency_penalty_enabled: false,
    logit_bias_enabled: false,
    stop_enabled: false,
    seed_enabled: false,
    systemPrompt_enabled: false,
    response_format_enabled: false,
    api_token_enabled: false,
    maxToolIterations: 5,
    maxToolIterations_enabled: true,
  };
  const store = create(() => ({
    ...defaults,
    setSystemPrompt: vi.fn(),
    setSystemPromptEnabled: vi.fn(),
  }));
  return { useChatSettingsStore: store, defaultSettings: defaults };
});

describe('SystemPrompt', () => {
  beforeEach(() => {
    useChatSettingsStore.setState({
      systemPrompt: '',
      systemPrompt_enabled: true,
      setSystemPrompt: vi.fn(),
      setSystemPromptEnabled: vi.fn(),
    });
  });

  describe('loading state', () => {
    it('disables all interactive elements when loading', () => {
      render(<SystemPrompt isLoading={true} />);

      const textarea = screen.getByRole('textbox');
      const switchElement = screen.getByRole('switch');

      expect(textarea).toBeDisabled();
      expect(switchElement).toBeDisabled();
    });
  });

  describe('enabled state', () => {
    it('reflects enabled state from chat settings', () => {
      useChatSettingsStore.setState({
        systemPrompt: '',
        systemPrompt_enabled: true,
      });

      render(<SystemPrompt />);

      const textarea = screen.getByRole('textbox');
      const switchElement = screen.getByRole('switch');

      expect(switchElement).toHaveAttribute('aria-checked', 'true');
      expect(textarea).not.toBeDisabled();
    });

    it('reflects disabled state from chat settings', () => {
      useChatSettingsStore.setState({
        systemPrompt: '',
        systemPrompt_enabled: false,
      });

      render(<SystemPrompt />);

      const textarea = screen.getByRole('textbox');
      const switchElement = screen.getByRole('switch');

      expect(switchElement).toHaveAttribute('aria-checked', 'false');
      expect(textarea).toBeDisabled();
    });
  });

  it('updates system prompt in chat settings', () => {
    const mockSetSystemPrompt = vi.fn();
    useChatSettingsStore.setState({
      systemPrompt: '',
      systemPrompt_enabled: true,
      setSystemPrompt: mockSetSystemPrompt,
    });

    render(<SystemPrompt />);

    const textarea = screen.getByRole('textbox');
    fireEvent.change(textarea, { target: { value: 'Test prompt' } });

    expect(mockSetSystemPrompt).toHaveBeenCalledWith('Test prompt');
  });

  it('updates enabled state in chat settings', () => {
    const mockSetEnabled = vi.fn();
    useChatSettingsStore.setState({
      systemPrompt: '',
      systemPrompt_enabled: true,
      setSystemPromptEnabled: mockSetEnabled,
    });

    render(<SystemPrompt />);

    const switchElement = screen.getByRole('switch');
    fireEvent.click(switchElement);

    expect(mockSetEnabled).toHaveBeenCalledWith(false);
  });

  it('displays existing system prompt from chat settings', () => {
    useChatSettingsStore.setState({
      systemPrompt: 'Existing prompt',
      systemPrompt_enabled: true,
    });

    render(<SystemPrompt />);

    const textarea = screen.getByRole('textbox');
    expect(textarea).toHaveValue('Existing prompt');
  });
});
