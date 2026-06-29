import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { ListingToggle } from './ListingToggle';

describe('ListingToggle', () => {
  it('renders label, code chip and description', () => {
    render(
      <ListingToggle
        checked={false}
        onToggle={vi.fn()}
        label="List all models"
        code="/v1/models"
        description="Let the app enumerate the catalog."
        testId="list-models"
      />
    );
    expect(screen.getByTestId('list-models')).toHaveAttribute('aria-checked', 'false');
    expect(screen.getByText('List all models')).toBeInTheDocument();
    expect(screen.getByText('/v1/models')).toBeInTheDocument();
    expect(screen.getByText(/enumerate the catalog/)).toBeInTheDocument();
  });

  it('toggles on click', async () => {
    const onToggle = vi.fn();
    render(
      <ListingToggle checked={false} onToggle={onToggle} label="List all MCPs" description="x" testId="list-mcps" />
    );
    await userEvent.click(screen.getByTestId('list-mcps'));
    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it('shows the redundant hint when access is already All', () => {
    render(
      <ListingToggle checked={false} onToggle={vi.fn()} label="List all models" description="x" redundant testId="lm" />
    );
    expect(screen.getByText(/already lists everything/)).toBeInTheDocument();
  });

  it('does not toggle when disabled', async () => {
    const onToggle = vi.fn();
    render(<ListingToggle checked onToggle={onToggle} label="List" description="x" disabled testId="lm" />);
    await userEvent.click(screen.getByTestId('lm'));
    expect(onToggle).not.toHaveBeenCalled();
  });
});
