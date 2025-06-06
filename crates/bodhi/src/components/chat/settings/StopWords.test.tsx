import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { StopWords } from '@/components/chat/settings/StopWords';
import * as chatSettings from '@/hooks/use-chat-settings';
import userEvent from '@testing-library/user-event';

// Mock useChatSettings
vi.mock('@/hooks/use-chat-settings', () => ({
  useChatSettings: vi.fn()
}));

// Example of mocking Input component with proper disabled state handling
vi.mock('@/components/ui/input', () => ({
  Input: ({ disabled, onChange, onKeyDown, ...props }: any) => (
    <input
      {...props}
      disabled={disabled}
      data-disabled={disabled}
      onChange={(e) => !disabled && onChange?.(e)}
      onKeyDown={(e) => !disabled && onKeyDown?.(e)}
    />
  ),
}));

// Mock the Switch component from shadcn
vi.mock('@/components/ui/switch', () => ({
  Switch: ({ checked, onCheckedChange, disabled, ...props }: any) => (
    <button
      role="switch"
      aria-checked={checked}
      onClick={() => !disabled && onCheckedChange(!checked)}
      disabled={disabled}
      {...props}
    />
  ),
}));

describe('StopWords', () => {
  beforeEach(() => {
    // Reset mock before each test with default values
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      stop: [],
      stop_enabled: true,
      setStop: vi.fn(),
      setStopEnabled: vi.fn(),
    } as any);
  });

  describe('loading state', () => {
    it('disables all interactive elements when loading', () => {
      render(<StopWords isLoading={true} />);

      const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
      const switchElement = screen.getByRole('switch');

      expect(input).toBeDisabled();
      expect(switchElement).toBeDisabled();
    });

    it('prevents interactions while loading', () => {
      const mockSetStop = vi.fn();
      const mockSetEnabled = vi.fn();

      vi.mocked(chatSettings.useChatSettings).mockReturnValue({
        stop: ['test'],
        stop_enabled: true,
        setStop: mockSetStop,
        setStopEnabled: mockSetEnabled,
      } as any);

      render(<StopWords isLoading={true} />);

      const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
      const switchElement = screen.getByRole('switch');
      const removeButton = screen.getByLabelText('Remove test');

      // Try interactions
      fireEvent.click(switchElement);
      fireEvent.click(removeButton);
      fireEvent.change(input, { target: { value: 'new' } });
      fireEvent.keyDown(input, { key: 'Enter' });

      expect(mockSetEnabled).not.toHaveBeenCalled();
      expect(mockSetStop).not.toHaveBeenCalled();
    });
  });

  describe('enabled state', () => {
    it('reflects enabled state from chat settings', () => {
      vi.mocked(chatSettings.useChatSettings).mockReturnValue({
        stop: [],
        stop_enabled: true,
        setStop: vi.fn(),
        setStopEnabled: vi.fn(),
      } as any);

      render(<StopWords />);

      const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
      const switchElement = screen.getByRole('switch');

      expect(switchElement).toHaveAttribute('aria-checked', 'true');
      expect(input).not.toBeDisabled();
    });

    it('reflects disabled state from chat settings', () => {
      vi.mocked(chatSettings.useChatSettings).mockReturnValue({
        stop: [],
        stop_enabled: false,
        setStop: vi.fn(),
        setStopEnabled: vi.fn(),
      } as any);

      render(<StopWords />);

      const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
      const switchElement = screen.getByRole('switch');

      expect(switchElement).toHaveAttribute('aria-checked', 'false');
      expect(input).toBeDisabled();
    });
  });

  it('displays existing stop words from chat settings', () => {
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      stop: ['word1', 'word2'],
      stop_enabled: true,
      setStop: vi.fn(),
      setStopEnabled: vi.fn(),
    } as any);

    render(<StopWords />);

    expect(screen.getByText('word1')).toBeInTheDocument();
    expect(screen.getByText('word2')).toBeInTheDocument();
  });

  it('adds new stop word to chat settings', () => {
    const mockSetStop = vi.fn();
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      stop: [],
      stop_enabled: true,
      setStop: mockSetStop,
      setStopEnabled: vi.fn(),
    } as any);

    render(<StopWords />);

    const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
    fireEvent.change(input, { target: { value: 'test' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(mockSetStop).toHaveBeenCalledWith(['test']);
  });

  it('adds to existing stop words in chat settings', () => {
    const mockSetStop = vi.fn();
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      stop: ['existing'],
      stop_enabled: true,
      setStop: mockSetStop,
      setStopEnabled: vi.fn(),
    } as any);

    render(<StopWords />);

    const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
    fireEvent.change(input, { target: { value: 'test' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(mockSetStop).toHaveBeenCalledWith(['existing', 'test']);
  });

  it('removes stop word from chat settings', () => {
    const mockSetStop = vi.fn();
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      stop: ['word1', 'word2'],
      stop_enabled: true,
      setStop: mockSetStop,
      setStopEnabled: vi.fn(),
    } as any);

    render(<StopWords />);

    const removeButton = screen.getByLabelText('Remove word1');
    fireEvent.click(removeButton);

    expect(mockSetStop).toHaveBeenCalledWith(['word2']);
  });

  it('sets empty array when removing last stop word', () => {
    const mockSetStop = vi.fn();
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      stop: ['word1'],
      stop_enabled: true,
      setStop: mockSetStop,
      setStopEnabled: vi.fn(),
    } as any);

    render(<StopWords />);

    const removeButton = screen.getByLabelText('Remove word1');
    fireEvent.click(removeButton);

    expect(mockSetStop).toHaveBeenCalledWith([]);
  });

  it('updates enabled state in chat settings', () => {
    const mockSetEnabled = vi.fn();
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      stop: [],
      stop_enabled: true,
      setStop: vi.fn(),
      setStopEnabled: mockSetEnabled,
    } as any);

    render(<StopWords />);

    const switchElement = screen.getByRole('switch');
    fireEvent.click(switchElement);

    expect(mockSetEnabled).toHaveBeenCalledWith(false);
  });

  it('handles stop words consistently as arrays', async () => {
    const user = userEvent.setup();
    const mockSetStop = vi.fn();
    let currentStop = ['existing'];

    // Update mock to track state changes
    mockSetStop.mockImplementation((newStop) => {
      currentStop = newStop;
    });

    vi.mocked(chatSettings.useChatSettings).mockImplementation(() => ({
      stop: currentStop,
      stop_enabled: true,
      setStop: mockSetStop,
      setStopEnabled: vi.fn(),
    } as any));

    render(<StopWords />);

    // Add new word using userEvent
    const input = screen.getByPlaceholderText('Type and press Enter to add stop words...');
    await user.type(input, 'test{Enter}');

    // Verify first array update
    expect(mockSetStop).toHaveBeenCalledWith(['existing', 'test']);

    // Re-render to get updated state
    const removeButton = screen.getByLabelText('Remove existing');
    await user.click(removeButton);

    // Verify second array update
    expect(mockSetStop).toHaveBeenCalledWith(['test']);

    // Add another word
    await user.type(input, 'another{Enter}');
    
    // Verify final array state
    expect(mockSetStop).toHaveBeenCalledWith(['test', 'another']);
  });
});