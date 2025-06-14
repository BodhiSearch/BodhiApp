import { render, screen } from '@testing-library/react';
import { ChatMessage } from './ChatMessage';
import { Message } from '@/types/chat';
import { vi } from 'vitest';
import userEvent from '@testing-library/user-event';

// Mock the markdown component
vi.mock('@/components/ui/markdown', () => ({
  MemoizedReactMarkdown: ({ children }: { children: string }) => <div data-testid="markdown">{children}</div>,
}));

// Mock the copy button component
vi.mock('@/components/CopyButton', () => ({
  CopyButton: ({ text, className }: { text: string; className?: string }) => (
    <button data-testid="copy-button" data-copy-text={text} className={className}>
      Copy
    </button>
  ),
}));

describe('ChatMessage', () => {
  const baseMessage: Message = {
    role: 'assistant',
    content: 'Test message content',
  };

  const messageWithMetadata: Message = {
    ...baseMessage,
    metadata: {
      model: 'test-model',
      usage: {
        completion_tokens: 16,
        prompt_tokens: 5,
        total_tokens: 21,
      },
      timings: {
        prompt_per_second: 41.7157,
        predicted_per_second: 31.04,
      },
    },
  };

  describe('basic rendering', () => {
    it('renders user message correctly', () => {
      render(<ChatMessage message={{ ...baseMessage, role: 'user' }} />);

      expect(screen.getByText('You')).toBeInTheDocument();
      expect(screen.getByTestId('markdown')).toHaveTextContent('Test message content');
      expect(screen.queryByText(/tokens/)).not.toBeInTheDocument();
    });

    it('renders assistant message correctly', async () => {
      const user = userEvent.setup();
      const { container } = render(<ChatMessage message={baseMessage} />);

      expect(screen.getByText('Assistant')).toBeInTheDocument();
      expect(screen.getByTestId('markdown')).toHaveTextContent('Test message content');

      // Find copy button by test id, regardless of visibility
      const copyButton = screen.getByTestId('copy-button');
      expect(copyButton).toBeInTheDocument();
      expect(copyButton.className).toContain('opacity-0');

      // Hover over message to show copy button
      await user.hover(container.firstChild as Element);
      expect(copyButton.className).toContain('group-hover:opacity-100');
    });
  });

  describe('metadata display', () => {
    it('displays complete metadata for assistant message', () => {
      render(<ChatMessage message={messageWithMetadata} />);

      // Check token information
      expect(screen.getByText(/Response:.+16 tokens/)).toBeInTheDocument();
      expect(screen.getByText(/Query:.+5 tokens/)).toBeInTheDocument();

      // Check speed information with flexible text matching
      expect(screen.getByText(/Speed:.+31\.04 t\/s/)).toBeInTheDocument();
    });

    it('does not display metadata section for streaming messages', () => {
      render(<ChatMessage message={messageWithMetadata} isStreaming={true} />);

      expect(screen.queryByText(/tokens/)).not.toBeInTheDocument();
      expect(screen.queryByText(/Speed:/)).not.toBeInTheDocument();
    });

    it('handles missing metadata gracefully', () => {
      render(<ChatMessage message={baseMessage} />);

      expect(screen.queryByText(/tokens/)).not.toBeInTheDocument();
      expect(screen.queryByText(/Speed:/)).not.toBeInTheDocument();
      expect(screen.getByTestId('copy-button')).toBeInTheDocument();
    });

    it('handles partial metadata gracefully', () => {
      const partialMetadataMessage: Message = {
        ...baseMessage,
        metadata: {
          usage: {
            completion_tokens: 16,
            prompt_tokens: 5,
            total_tokens: 21,
          },
          // No timings data
        },
      };

      render(<ChatMessage message={partialMetadataMessage} />);

      // Should show token information
      expect(screen.getByText(/Response:.+16 tokens/)).toBeInTheDocument();
      expect(screen.getByText(/Query:.+5 tokens/)).toBeInTheDocument();

      // Should not show speed information
      expect(screen.queryByText(/Speed:/)).not.toBeInTheDocument();
    });

    it('handles zero values in metadata correctly', () => {
      const zeroMetadataMessage: Message = {
        ...baseMessage,
        metadata: {
          usage: {
            completion_tokens: 0,
            prompt_tokens: 0,
            total_tokens: 0,
          },
          timings: {
            prompt_per_second: 0,
            predicted_per_second: 0,
          },
        },
      };

      render(<ChatMessage message={zeroMetadataMessage} />);

      expect(screen.getByText(/Response: 0 tokens/)).toBeInTheDocument();
      expect(screen.getByText(/Query: 0 tokens/)).toBeInTheDocument();
    });
  });

  describe('copy button behavior', () => {
    it('shows copy button only for assistant messages', () => {
      const { rerender } = render(<ChatMessage message={messageWithMetadata} />);
      expect(screen.getByTestId('copy-button')).toBeInTheDocument();

      rerender(<ChatMessage message={{ ...messageWithMetadata, role: 'user' }} />);
      expect(screen.queryByTestId('copy-button')).not.toBeInTheDocument();
    });

    it('does not show copy button during streaming', () => {
      render(<ChatMessage message={messageWithMetadata} isStreaming={true} />);
      expect(screen.queryByTestId('copy-button')).not.toBeInTheDocument();
    });

    it('copy button has correct text content', () => {
      render(<ChatMessage message={messageWithMetadata} />);
      const copyButton = screen.getByTestId('copy-button');
      expect(copyButton).toHaveAttribute('data-copy-text', 'Test message content');
    });
  });
});
