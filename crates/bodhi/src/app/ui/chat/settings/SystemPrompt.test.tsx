import { SystemPrompt } from '@/app/ui/chat/settings/SystemPrompt';
import * as chatSettings from '@/hooks/use-chat-settings';
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

// Mock useChatSettings
vi.mock('@/hooks/use-chat-settings', () => ({
  useChatSettings: vi.fn(),
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
