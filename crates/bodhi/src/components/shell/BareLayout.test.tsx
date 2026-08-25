import { BareLayout } from '@/components/shell/BareLayout';
import { ThemeProvider } from '@/components/ThemeProvider';
import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: { to: string; children: React.ReactNode } & Record<string, unknown>) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
  };
});

describe('BareLayout', () => {
  it('renders a slim topbar with brand + theme toggle and the children, no shell sidebar', () => {
    render(
      <ThemeProvider>
        <BareLayout>
          <div data-testid="bare-child">consent form</div>
        </BareLayout>
      </ThemeProvider>
    );

    expect(screen.getByTestId('bare-layout')).toBeInTheDocument();
    expect(screen.getByTestId('bare-child')).toHaveTextContent('consent form');
    expect(screen.getByText('Bodhi')).toBeInTheDocument();
    expect(screen.getByText('AI Operating System')).toBeInTheDocument();
    // theme toggle present (shadcn ThemeToggle exposes an sr-only label)
    expect(screen.getByText('Toggle theme')).toBeInTheDocument();
    // confirms this is not the AppShell
    expect(screen.queryByTestId('shell-nav-trigger')).not.toBeInTheDocument();
  });
});
