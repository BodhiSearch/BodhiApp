import DocsPage from '@/app/docs/page';
import * as utils from '@/app/docs/utils';
import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

// Mock the utils module
vi.mock('@/app/docs/utils', () => ({
  getDocsForPath: vi.fn(),
}));

describe('DocsPage', () => {
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
      ],
    },
    {
      title: 'Advanced',
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

  it('renders documentation index with root level docs', () => {
    // Setup mock
    vi.mocked(utils.getDocsForPath).mockReturnValue(mockGroups);

    render(<DocsPage />);

    // Verify getDocsForPath was called correctly
    expect(utils.getDocsForPath).toHaveBeenCalledWith(null);

    // Verify the docs index is rendered with the correct groups
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Documentation');
    expect(screen.getByText(/Welcome to our documentation/)).toBeInTheDocument();

    // Verify groups are rendered
    expect(screen.getByRole('heading', { name: 'Getting Started' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Advanced' })).toBeInTheDocument();

    // Verify doc items are rendered
    expect(screen.getByRole('link', { name: /Introduction/ })).toHaveAttribute(
      'href',
      '/docs/intro'
    );
    expect(screen.getByRole('link', { name: /Configuration/ })).toHaveAttribute(
      'href',
      '/docs/advanced/config'
    );
  });

  it('handles empty documentation gracefully', () => {
    // Setup mock to return empty groups
    vi.mocked(utils.getDocsForPath).mockReturnValue([]);

    render(<DocsPage />);

    // Verify getDocsForPath was called
    expect(utils.getDocsForPath).toHaveBeenCalledWith(null);

    // Verify basic structure is still rendered
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Documentation');
    expect(screen.getByText(/Welcome to our documentation/)).toBeInTheDocument();
    expect(screen.queryByRole('heading', { level: 2 })).not.toBeInTheDocument();
  });

  test.skip('handles error state gracefully', () => {
    // Setup mock to throw error
    vi.mocked(utils.getDocsForPath).mockImplementation(() => {
      throw new Error('Failed to load docs');
    });

    // Error should be caught and empty state rendered
    render(<DocsPage />);

    // Verify basic structure is still rendered
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Documentation');
    expect(screen.getByText(/Welcome to our documentation/)).toBeInTheDocument();
    expect(screen.queryByRole('heading', { level: 2 })).not.toBeInTheDocument();
  });
});
