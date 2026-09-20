import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { HelpChip } from '@/components/help-chip';

describe('HelpChip', () => {
  it('renders a quiet button with a leading help glyph', () => {
    render(<HelpChip data-testid="chip">How to install it</HelpChip>);

    const chip = screen.getByTestId('chip');
    expect(chip.tagName).toBe('BUTTON');
    expect(chip).toHaveAttribute('type', 'button');
    expect(chip).toHaveTextContent('How to install it');
    expect(chip.querySelector('svg')).toBeInTheDocument();
    expect(chip.className).toContain('text-muted-foreground');
    expect(chip.className).not.toContain('underline');
  });

  it('calls onClick', async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    render(<HelpChip onClick={onClick}>Why this happens</HelpChip>);

    await user.click(screen.getByRole('button', { name: /why this happens/i }));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('wraps an anchor with asChild, keeping the glyph inside the link', () => {
    render(
      <HelpChip asChild>
        <a href="https://example.com" data-testid="chip-link">
          Cloudflare docs
        </a>
      </HelpChip>
    );

    const link = screen.getByTestId('chip-link');
    expect(link.tagName).toBe('A');
    expect(link).toHaveAttribute('href', 'https://example.com');
    expect(link.querySelector('svg')).toBeInTheDocument();
    expect(link).toHaveTextContent('Cloudflare docs');
  });

  it('accepts a custom glyph', () => {
    render(
      <HelpChip icon={<span data-testid="custom-glyph">*</span>} data-testid="chip">
        Details
      </HelpChip>
    );

    expect(screen.getByTestId('custom-glyph')).toBeInTheDocument();
    expect(screen.getByTestId('chip').querySelector('svg')).toBeNull();
  });
});
