import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { EmptyState } from './EmptyState';

describe('EmptyState', () => {
  it('renders the title, sub, and testid', () => {
    render(<EmptyState icon="search-x" title="No models found" sub="Try a different search." testId="cat-empty" />);

    const root = screen.getByTestId('cat-empty');
    expect(root).toHaveClass('empty-state');
    expect(screen.getByText('No models found')).toHaveClass('empty-title');
    expect(screen.getByText('Try a different search.')).toHaveClass('empty-sub');
  });

  it('omits the sub line when sub is not provided', () => {
    const { container } = render(<EmptyState icon="key-round" title="No tokens" />);

    expect(screen.getByText('No tokens')).toBeInTheDocument();
    expect(container.querySelector('.empty-sub')).toBeNull();
  });

  it('accepts a ReactNode title/sub', () => {
    render(<EmptyState icon="search-x" title={<span data-testid="t">x</span>} sub={<em data-testid="s">y</em>} />);
    expect(screen.getByTestId('t')).toBeInTheDocument();
    expect(screen.getByTestId('s')).toBeInTheDocument();
  });
});
