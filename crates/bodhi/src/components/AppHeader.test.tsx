import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import AppHeader from '@/components/AppHeader';

// Mock the child components
vi.mock('./PageNavigation', () => ({
  default: () => <div data-testid="page-navigation">Mocked PageNavigation</div>,
}));

vi.mock('./UserMenu', () => ({
  default: () => <div data-testid="user-menu">Mocked UserMenu</div>,
}));

vi.mock('next/image', () => ({
  default: (props: any) => <img {...props} />,
}));

describe('AppHeader', () => {
  it('renders all components correctly', () => {
    render(<AppHeader />);

    // Check for the presence of the logo and title
    expect(screen.getByAltText('Bodhi Logo')).toBeInTheDocument();
    expect(screen.getByText('Bodhi')).toBeInTheDocument();

    // Check for the presence of mocked components
    expect(screen.getByTestId('page-navigation')).toBeInTheDocument();
    expect(screen.getByTestId('user-menu')).toBeInTheDocument();
  });
});
