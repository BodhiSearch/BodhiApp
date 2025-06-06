import { Nav } from '@/app/docs/nav';
import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

// Mock Link component
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    Link: ({ children, to, ...props }: any) => <a href={to} {...props}>{children}</a>,
  };
});

describe('Nav', () => {
  const mockItems = [
    {
      title: 'Getting Started',
      href: '/docs/getting-started/',
      selected: true,
      children: [
        {
          title: 'Introduction',
          href: '/docs/getting-started/intro/',
          selected: false,
        },
        {
          title: 'Installation',
          href: '/docs/getting-started/install/',
          selected: false,
        },
      ],
    },
    {
      title: 'Features',
      href: '/docs/features/',
      selected: false,
      label: 'New',
    },
    {
      title: 'External Link',
      href: 'https://example.com/',
      external: true,
      selected: false,
    },
    {
      title: 'Disabled Item',
      href: '/docs/disabled/',
      disabled: true,
      selected: false,
    },
  ];

  it('renders navigation structure correctly', () => {
    render(<Nav items={mockItems} />);

    // Check main navigation container
    const nav = screen.getByRole('navigation');
    expect(nav).toHaveAttribute('aria-label', 'Documentation navigation');

    // Check all top-level items are rendered
    mockItems.forEach((item) => {
      expect(screen.getByText(item.title)).toBeInTheDocument();
    });
  });

  it('renders nested items correctly', () => {
    render(<Nav items={mockItems} />);

    // Check parent item
    const parentGroup = screen.getByTestId('nav-group-getting-started');
    expect(parentGroup).toBeInTheDocument();

    // Check children items
    const children = mockItems[0].children!;
    children.forEach((child) => {
      const childLink = screen.getByText(child.title);
      expect(childLink).toBeInTheDocument();
      expect(childLink.closest('a')).toHaveAttribute('href', child.href);
    });

    // Check group structure
    const groupContainer = screen.getByTestId('nav-group-children-getting-started');
    expect(groupContainer).toHaveAttribute('role', 'group');
    expect(groupContainer).toHaveAttribute('aria-label', 'Getting Started sub-navigation');
  });

  it('handles external links correctly', () => {
    render(<Nav items={mockItems} />);

    const externalLink = screen.getByTestId('nav-link-external-link');
    expect(externalLink).toHaveAttribute('target', '_blank');
    expect(externalLink).toHaveAttribute('rel', 'noreferrer');
  });

  it('handles disabled items correctly', () => {
    render(<Nav items={mockItems} />);

    const disabledLink = screen.getByTestId('nav-link-disabled-item');
    expect(disabledLink).toHaveAttribute('aria-disabled', 'true');
  });

  it('displays labels with proper accessibility', () => {
    render(<Nav items={mockItems} />);

    const label = screen.getByTestId('nav-label-new');
    expect(label).toHaveTextContent('New');
    expect(label).toHaveAttribute('aria-label', 'New');
  });

  it('marks active group correctly', () => {
    render(<Nav items={mockItems} />);

    const activeLink = screen.getByTestId('nav-group-title-getting-started');
    expect(activeLink).toHaveAttribute('aria-current', 'page');
  });

  it('marks active child correctly', () => {
    const itemsWithActiveChild = [
      {
        title: 'Getting Started',
        href: '/docs/getting-started/',
        selected: false,
        children: [
          {
            title: 'Introduction',
            href: '/docs/getting-started/intro/',
            selected: true,
          },
          {
            title: 'Installation',
            href: '/docs/getting-started/install/',
            selected: false,
          },
        ],
      },
    ];

    render(<Nav items={itemsWithActiveChild} />);

    const activeChildLink = screen.getByTestId('nav-link-introduction');
    expect(activeChildLink).toHaveAttribute('aria-current', 'page');
  });

  it('handles empty items array gracefully', () => {
    render(<Nav items={[]} />);

    const nav = screen.getByRole('navigation');
    expect(nav).toBeEmptyDOMElement();
  });
}); 