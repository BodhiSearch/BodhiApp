import { DOCS_CONFIG } from '@/app/docs/constants';
import { createMockGroup } from '@/app/docs/test-utils';
import * as utils from '@/app/docs/utils';
import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import DocsPage from './page';

// Mock the utils module
vi.mock('@/app/docs/utils', () => ({
  getDocsForPath: vi.fn(),
}));

describe('DocsPage', () => {
  const mockGroups = [
    createMockGroup({
      title: 'Getting Started',
      key: 'getting-started',
      items: [
        {
          title: 'Introduction',
          description: 'Get started with our platform',
          slug: 'intro',
          order: 1,
        },
      ],
    }),
    createMockGroup({
      title: 'Advanced',
      key: 'advanced',
      items: [
        {
          title: 'Configuration',
          description: 'Advanced configuration options',
          slug: 'advanced/config',
          order: 1,
        },
      ],
    }),
  ];

  it('renders documentation index with root level docs', () => {
    vi.mocked(utils.getDocsForPath).mockReturnValue(mockGroups);
    render(<DocsPage />);

    // Verify getDocsForPath was called correctly
    expect(utils.getDocsForPath).toHaveBeenCalledWith(null);

    // Verify title and description
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(DOCS_CONFIG.defaultTitle);
    expect(screen.getByText(DOCS_CONFIG.defaultDescription)).toBeInTheDocument();

    // Verify groups and items
    expect(screen.getByRole('heading', { name: 'Getting Started' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Advanced' })).toBeInTheDocument();

    // Verify links and descriptions
    expect(screen.getByRole('link', { name: /Introduction/ })).toHaveAttribute('href', '/docs/intro');
    expect(screen.getByRole('link', { name: /Configuration/ })).toHaveAttribute('href', '/docs/advanced/config');
  });

  it('handles empty documentation gracefully', () => {
    vi.mocked(utils.getDocsForPath).mockReturnValue([]);
    render(<DocsPage />);

    // Verify empty state
    expect(screen.getByText('No documentation available.')).toBeInTheDocument();
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
