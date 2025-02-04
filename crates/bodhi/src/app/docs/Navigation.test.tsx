import { Navigation } from '@/app/docs/Navigation';
import { NavItem } from '@/app/docs/types';
import { render, screen } from '@testing-library/react';
import { usePathname } from 'next/navigation';
import { describe, expect, it, vi } from 'vitest';

// Mock next/navigation
vi.mock('next/navigation', () => ({
  usePathname: vi.fn(),
}));

// Mock next/link
vi.mock('next/link', () => ({
  default: ({ children, ...props }: any) => <a {...props}>{children}</a>,
}));

describe('Navigation', () => {
  const mockDocItems: NavItem[] = [
    {
      title: 'Getting Started',
      slug: 'getting-started',
      children: [
        {
          title: 'Introduction',
          slug: 'getting-started/intro',
        },
        {
          title: 'Installation',
          slug: 'getting-started/install',
        },
      ],
    },
    {
      title: 'Features',
      slug: 'features',
    },
  ];

  beforeEach(() => {
    vi.mocked(usePathname).mockReset();
  });

  it('renders navigation structure with correct paths', () => {
    vi.mocked(usePathname).mockReturnValue('/docs/getting-started/');

    render(<Navigation items={mockDocItems} />);

    expect(screen.getByTestId('main-navigation')).toBeInTheDocument();

    // Check parent items
    const gettingStartedLink = screen.getByText('Getting Started').closest('a');
    expect(gettingStartedLink).toHaveAttribute('href', '/docs/getting-started/');

    const featuresLink = screen.getByText('Features').closest('a');
    expect(featuresLink).toHaveAttribute('href', '/docs/features/');

    // Check child items
    const introLink = screen.getByText('Introduction').closest('a');
    expect(introLink).toHaveAttribute('href', '/docs/getting-started/intro/');

    const installLink = screen.getByText('Installation').closest('a');
    expect(installLink).toHaveAttribute('href', '/docs/getting-started/install/');
  });

  it('marks current page as active', () => {
    vi.mocked(usePathname).mockReturnValue('/docs/getting-started/');

    render(<Navigation items={mockDocItems} />);

    const activeLink = screen.getByTestId('nav-group-title-getting-started');
    expect(activeLink).toHaveAttribute('aria-current', 'page');
  });

  it('marks active child page correctly', () => {
    vi.mocked(usePathname).mockReturnValue('/docs/getting-started/intro/');

    render(<Navigation items={mockDocItems} />);

    const activeChildLink = screen.getByTestId('nav-link-introduction');
    expect(activeChildLink).toHaveAttribute('aria-current', 'page');
  });

  it('handles empty items array gracefully', () => {
    render(<Navigation items={[]} />);

    const nav = screen.getByTestId('main-navigation');
    expect(nav).toBeInTheDocument();
    expect(nav.querySelector('nav')).toBeEmptyDOMElement();
  });

  it('converts doc items to nav items with correct structure', () => {
    vi.mocked(usePathname).mockReturnValue('/docs/getting-started/');

    render(<Navigation items={mockDocItems} />);

    // Check parent folder structure
    const parentGroup = screen.getByTestId('nav-group-getting-started');
    expect(parentGroup).toBeInTheDocument();

    // Check children are nested correctly
    const childrenGroup = screen.getByTestId('nav-group-children-getting-started');
    expect(childrenGroup).toHaveAttribute('role', 'group');
    expect(childrenGroup).toHaveAttribute('aria-label', 'Getting Started sub-navigation');

    // Verify all items are present
    mockDocItems.forEach((item) => {
      expect(screen.getByText(item.title)).toBeInTheDocument();
      item.children?.forEach((child) => {
        expect(screen.getByText(child.title)).toBeInTheDocument();
      });
    });
  });
}); 