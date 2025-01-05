import { render, screen, fireEvent } from '@testing-library/react';
import { SidebarToggle } from '@/components/SidebarToggle';
import { Settings2 } from 'lucide-react';
import { describe, expect, it, vi } from 'vitest';

describe('SidebarToggle', () => {
  it('renders with default props', () => {
    render(<SidebarToggle open={false} onOpenChange={() => { }} />);
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('calls onOpenChange when clicked', () => {
    const handleOpenChange = vi.fn();
    render(<SidebarToggle open={false} onOpenChange={handleOpenChange} />);

    fireEvent.click(screen.getByRole('button'));
    expect(handleOpenChange).toHaveBeenCalledWith(true);
  });

  it('renders custom icon when provided', () => {
    render(
      <SidebarToggle
        open={false}
        onOpenChange={() => { }}
        icon={<Settings2 data-testid="settings-icon" />}
      />
    );
    expect(screen.getByTestId('settings-icon')).toBeInTheDocument();
  });

  it('applies correct positioning classes for left side', () => {
    const { container } = render(
      <SidebarToggle open={true} onOpenChange={() => { }} side="left" />
    );
    const div = container.firstChild as HTMLElement;
    expect(div.className).toContain('left-[16rem]');
  });

  it('applies correct positioning classes for right side', () => {
    const { container } = render(
      <SidebarToggle open={true} onOpenChange={() => { }} side="right" />
    );
    const div = container.firstChild as HTMLElement;
    expect(div.className).toContain('right-[16rem]');
  });
});