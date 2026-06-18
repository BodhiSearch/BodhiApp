import { LinkRow } from '@/components/shell';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

describe('LinkRow', () => {
  it('renders a stretched anchor with the given accessible name', () => {
    render(<LinkRow onActivate={vi.fn()} label="Open token Prod" />);
    const link = screen.getByTestId('row-link');
    expect(link.tagName).toBe('A');
    expect(link).toHaveAccessibleName('Open token Prod');
    expect(link).toHaveAttribute('href', '#');
  });

  it('falls back to a generic accessible name when no label is given', () => {
    render(<LinkRow onActivate={vi.fn()} />);
    expect(screen.getByTestId('row-link')).toHaveAccessibleName('Open details');
  });

  it('activates once on click and does not navigate or bubble', async () => {
    const onActivate = vi.fn();
    const onParentClick = vi.fn();
    const user = userEvent.setup();
    render(
      // a parent onClick stands in for the row div's own onClick — stopPropagation must keep it from firing
      <div onClick={onParentClick}>
        <LinkRow onActivate={onActivate} label="Open thing" />
      </div>
    );

    await user.click(screen.getByTestId('row-link'));
    expect(onActivate).toHaveBeenCalledTimes(1);
    expect(onParentClick).not.toHaveBeenCalled();
  });
});
