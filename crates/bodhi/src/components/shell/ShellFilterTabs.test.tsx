import { ShellFilterTabs } from '@/components/shell';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

const TABS = [
  { id: 'all', label: 'All', count: 5 },
  { id: 'active', label: 'Active', count: 3 },
  { id: 'inactive', label: 'Inactive', count: 2 },
];

describe('ShellFilterTabs', () => {
  it('renders numeric count badges when not loading', () => {
    render(<ShellFilterTabs tabs={TABS} value="all" onChange={vi.fn()} testIdPrefix="f" />);

    expect(within(screen.getByTestId('f-all')).getByText('5')).toBeInTheDocument();
    expect(within(screen.getByTestId('f-active')).getByText('3')).toBeInTheDocument();
    expect(screen.queryByLabelText('Loading count')).not.toBeInTheDocument();
  });

  it('shows shimmer placeholders instead of counts while loading', () => {
    render(<ShellFilterTabs tabs={TABS} value="all" onChange={vi.fn()} testIdPrefix="f" loading />);

    // every tab gets a loading placeholder, no numeric counts leak through
    expect(screen.getAllByLabelText('Loading count')).toHaveLength(TABS.length);
    expect(screen.queryByText('5')).not.toBeInTheDocument();
    expect(screen.queryByText('3')).not.toBeInTheDocument();

    const badge = within(screen.getByTestId('f-all')).getByLabelText('Loading count');
    expect(badge).toHaveClass('l-cat-badge', 'l-cat-badge--loading');
  });

  it('renders a loading placeholder even when a tab has no count', () => {
    render(
      <ShellFilterTabs tabs={[{ id: 'all', label: 'All' }]} value="all" onChange={vi.fn()} testIdPrefix="f" loading />
    );

    expect(within(screen.getByTestId('f-all')).getByLabelText('Loading count')).toBeInTheDocument();
  });

  it('marks the active tab and fires onChange on click', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    render(<ShellFilterTabs tabs={TABS} value="all" onChange={onChange} testIdPrefix="f" />);

    expect(screen.getByTestId('f-all')).toHaveAttribute('aria-selected', 'true');
    expect(screen.getByTestId('f-active')).toHaveAttribute('aria-selected', 'false');

    await user.click(screen.getByTestId('f-active'));
    expect(onChange).toHaveBeenCalledWith('active');
  });
});
