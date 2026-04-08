import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { ThinkingBlock } from './ThinkingBlock';

describe('ThinkingBlock', () => {
  it('should render thinking toggle with collapsed content', () => {
    render(<ThinkingBlock thinking="Let me think about this..." />);

    expect(screen.getByTestId('thinking-block')).toBeInTheDocument();
    expect(screen.getByText('Thought process')).toBeInTheDocument();
    expect(screen.queryByTestId('thinking-block-content')).not.toBeVisible();
  });

  it('should expand to show thinking content when toggled', () => {
    render(<ThinkingBlock thinking="Let me analyze step by step..." />);

    fireEvent.click(screen.getByTestId('thinking-block-toggle'));

    expect(screen.getByTestId('thinking-block-content')).toBeVisible();
    expect(screen.getByText('Let me analyze step by step...')).toBeInTheDocument();
  });

  it('should show streaming label when isStreaming', () => {
    render(<ThinkingBlock thinking="Still thinking..." isStreaming />);

    expect(screen.getByText('Thinking...')).toBeInTheDocument();
  });

  it('should show completed label when not streaming', () => {
    render(<ThinkingBlock thinking="Done thinking" isStreaming={false} />);

    expect(screen.getByText('Thought process')).toBeInTheDocument();
  });

  it('should render nothing when thinking is empty', () => {
    const { container } = render(<ThinkingBlock thinking="" />);
    expect(container.firstChild).toBeNull();
  });
});
