import { DocsIndex } from '@/components/docs/DocsIndex';
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { createWrapper } from '@/tests/wrapper';

describe('DocsIndex', () => {
  const mockGroups = [
    {
      title: 'Getting Started',
      key: 'getting-started',
      order: 0,
      items: [
        {
          title: 'Introduction',
          description: 'Get started with our platform',
          slug: 'intro',
          order: 1,
        },
        {
          title: 'Installation',
          description: 'How to install the software',
          slug: 'install',
          order: 2,
        },
      ],
    },
    {
      title: 'Advanced Topics',
      key: 'advanced',
      order: 1,
      items: [
        {
          title: 'Configuration',
          description: 'Advanced configuration options',
          slug: 'advanced/config',
          order: 1,
        },
      ],
    },
  ];

  it('renders complete documentation index with default props', () => {
    render(<DocsIndex groups={mockGroups} title="Custom Title" description="Custom description" />, { wrapper: createWrapper() });

    // Check main title and description
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Custom Title');
    expect(screen.getByText('Custom description')).toBeInTheDocument();

    // Check all groups and their content
    expect(screen.getByRole('heading', { name: 'Getting Started' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Advanced Topics' })).toBeInTheDocument();

    // Check all items are rendered with correct links and descriptions
    const links = screen.getAllByRole('link');
    expect(links).toHaveLength(3);

    // Introduction item
    const introLink = screen.getByRole('link', { name: /Introduction/ });
    expect(introLink).toHaveAttribute('href', '/docs/intro');
    expect(screen.getByText('Get started with our platform')).toBeInTheDocument();

    // Installation item
    const installLink = screen.getByRole('link', { name: /Installation/ });
    expect(installLink).toHaveAttribute('href', '/docs/install');
    expect(screen.getByText('How to install the software')).toBeInTheDocument();

    // Configuration item
    const configLink = screen.getByRole('link', { name: /Configuration/ });
    expect(configLink).toHaveAttribute('href', '/docs/advanced/config');
    expect(screen.getByText('Advanced configuration options')).toBeInTheDocument();
  });

  it('does not render with title and description if not provided', () => {
    render(
      <DocsIndex
        groups={mockGroups}
      />, { wrapper: createWrapper() }
    );

    expect(screen.queryByRole('heading', { level: 1 })).not.toBeInTheDocument();
  });

  it('handles missing description and empty item descriptions', () => {
    const groupsWithoutDesc = [
      {
        title: 'Test',
        key: 'test',
        order: 0,
        items: [
          {
            title: 'Test Doc',
            description: '',
            slug: 'test',
            order: 1,
          },
        ],
      },
    ];

    render(
      <DocsIndex
        groups={groupsWithoutDesc}
      />, { wrapper: createWrapper() }
    );

    expect(screen.getByRole('link', { name: 'Test Doc' })).toBeInTheDocument();
    expect(screen.queryByText(/description/i)).not.toBeInTheDocument();
  });

  it('handles empty groups array', () => {
    render(<DocsIndex groups={[]} />, { wrapper: createWrapper() });

    expect(screen.queryByRole('heading', { level: 1 })).not.toBeInTheDocument();
  });
});
