import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { SystemPrompt } from './SystemPrompt';
import * as chatSettings from '@/hooks/use-chat-settings';

// Mock useChatSettings
vi.mock('@/hooks/use-chat-settings', () => ({
  useChatSettings: vi.fn()
}));

describe('SystemPrompt', () => {
  beforeEach(() => {
    // Reset mock before each test with default values
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      systemPrompt: '',
      systemPrompt_enabled: true,
      setSystemPrompt: vi.fn(),
      setSystemPromptEnabled: vi.fn(),
    } as any);
  });

  describe('loading state', () => {
    it('disables all interactive elements when loading', () => {
      render(<SystemPrompt isLoading={true} />);

      const textarea = screen.getByRole('textbox');
      const switchElement = screen.getByRole('switch');

      expect(textarea).toBeDisabled();
      expect(switchElement).toBeDisabled();
    });

    it.skip('prevents interactions while loading', () => {
      const mockSetSystemPrompt = vi.fn();
      const mockSetEnabled = vi.fn();

      vi.mocked(chatSettings.useChatSettings).mockReturnValue({
        systemPrompt: '',
        systemPrompt_enabled: true,
        setSystemPrompt: mockSetSystemPrompt,
        setSystemPromptEnabled: mockSetEnabled,
      } as any);

      render(<SystemPrompt isLoading={true} />);

      const switchElement = screen.getByRole('switch');
      const textarea = screen.getByRole('textbox');

      // Try to toggle switch
      fireEvent.click(switchElement);
      expect(mockSetEnabled).not.toHaveBeenCalled();

      // Try to input text
      fireEvent.change(textarea, { target: { value: 'Test input' } });
      expect(mockSetSystemPrompt).not.toHaveBeenCalled();
    });
  });

  describe('enabled state', () => {
    it('reflects enabled state from chat settings', () => {
      vi.mocked(chatSettings.useChatSettings).mockReturnValue({
        systemPrompt: '',
        systemPrompt_enabled: true,
        setSystemPrompt: vi.fn(),
        setSystemPromptEnabled: vi.fn(),
      } as any);

      render(<SystemPrompt />);

      const textarea = screen.getByRole('textbox');
      const switchElement = screen.getByRole('switch');

      expect(switchElement).toHaveAttribute('aria-checked', 'true');
      expect(textarea).not.toBeDisabled();
    });

    it('reflects disabled state from chat settings', () => {
      vi.mocked(chatSettings.useChatSettings).mockReturnValue({
        systemPrompt: '',
        systemPrompt_enabled: false,
        setSystemPrompt: vi.fn(),
        setSystemPromptEnabled: vi.fn(),
      } as any);

      render(<SystemPrompt />);

      const textarea = screen.getByRole('textbox');
      const switchElement = screen.getByRole('switch');

      expect(switchElement).toHaveAttribute('aria-checked', 'false');
      expect(textarea).toBeDisabled();
    });
  });

  it('updates system prompt in chat settings', () => {
    const mockSetSystemPrompt = vi.fn();
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      systemPrompt: '',
      systemPrompt_enabled: true,
      setSystemPrompt: mockSetSystemPrompt,
      setSystemPromptEnabled: vi.fn(),
    } as any);

    render(<SystemPrompt />);

    const textarea = screen.getByRole('textbox');
    fireEvent.change(textarea, { target: { value: 'Test prompt' } });

    expect(mockSetSystemPrompt).toHaveBeenCalledWith('Test prompt');
  });

  it('updates enabled state in chat settings', () => {
    const mockSetEnabled = vi.fn();
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      systemPrompt: '',
      systemPrompt_enabled: true,
      setSystemPrompt: vi.fn(),
      setSystemPromptEnabled: mockSetEnabled,
    } as any);

    render(<SystemPrompt />);

    const switchElement = screen.getByRole('switch');
    fireEvent.click(switchElement);

    expect(mockSetEnabled).toHaveBeenCalledWith(false);
  });

  it('displays existing system prompt from chat settings', () => {
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      systemPrompt: 'Existing prompt',
      systemPrompt_enabled: true,
      setSystemPrompt: vi.fn(),
      setSystemPromptEnabled: vi.fn(),
    } as any);

    render(<SystemPrompt />);

    const textarea = screen.getByRole('textbox');
    expect(textarea).toHaveValue('Existing prompt');
  });
});