import { BareLayout } from '@/components/shell/BareLayout';
import { ThemeProvider } from '@/components/ThemeProvider';
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

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
    // brand present
    expect(screen.getByText('Bodhi')).toBeInTheDocument();
    expect(screen.getByText('AI Operating System')).toBeInTheDocument();
    // theme toggle present (shadcn ThemeToggle exposes an sr-only label)
    expect(screen.getByText('Toggle theme')).toBeInTheDocument();
    // NOT the AppShell — no shell sidebar nav trigger
    expect(screen.queryByTestId('shell-nav-trigger')).not.toBeInTheDocument();
  });
});
