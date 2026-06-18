import { ShellFooter } from '@/components/shell';
import { ThemeProvider } from '@/components/ThemeProvider';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

function renderFooter() {
  return render(
    <ThemeProvider defaultTheme="system" storageKey="bodhi-ui-theme">
      <ShellFooter user={{ name: 'Tester', role: 'Admin', initials: 'TS' }} />
    </ThemeProvider>
  );
}

beforeEach(() => {
  localStorage.clear();
  document.documentElement.classList.remove('light', 'dark');
});

afterEach(() => {
  localStorage.clear();
});

describe('Shell theme switch (always visible above the user chip)', () => {
  it('exposes light/dark/system options without opening any menu and applies the choice', async () => {
    const user = userEvent.setup();
    renderFooter();

    // No menu interaction — the switch is always visible in the footer.
    const lightBtn = screen.getByTestId('shell-theme-light');
    const darkBtn = screen.getByTestId('shell-theme-dark');
    expect(screen.getByTestId('shell-theme-system')).toBeInTheDocument();

    await user.click(darkBtn);
    expect(darkBtn).toHaveAttribute('aria-pressed', 'true');
    expect(document.documentElement).toHaveClass('dark');
    expect(localStorage.getItem('bodhi-ui-theme')).toContain('dark');

    await user.click(lightBtn);
    expect(lightBtn).toHaveAttribute('aria-pressed', 'true');
    expect(document.documentElement).toHaveClass('light');
    expect(document.documentElement).not.toHaveClass('dark');
  });
});
