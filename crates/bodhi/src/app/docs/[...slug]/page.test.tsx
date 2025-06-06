import { createMockDoc, createMockGroup, mockMarkdownContent } from '@/app/docs/test-utils';
import { getAllDocSlugs, getDocsForSlug } from '@/app/docs/utils';
import { act, render, screen } from '@testing-library/react';
import fs from 'fs';
import matter from 'gray-matter';
import { notFound } from '@/lib/navigation';
import { describe, expect, it, vi } from 'vitest';
import { createWrapper } from '@/tests/wrapper';
import DocsSlugPage, { generateStaticParams } from './page';

// Mock dependencies
vi.mock('@/lib/navigation', () => ({
  notFound: vi.fn(() => {
    throw new Error('Not Found');
  }),
}));

vi.mock('@/app/docs/utils', () => ({
  getAllDocSlugs: vi.fn(),
  getDocsForSlug: vi.fn(),
}));

// Mock fs module with default export
vi.mock('fs', () => {
  const mockFs = {
    existsSync: vi.fn(),
    statSync: vi.fn(),
    readFileSync: vi.fn(),
  };
  return {
    default: mockFs,
    ...mockFs,
  };
});

// Mock gray-matter
vi.mock('gray-matter', () => {
  return {
    default: vi.fn(),
  };
});

describe('DocsSlugPage', () => {
  const mockGroups = [createMockGroup({
    title: 'Nested',
    key: 'nested',
    items: [createMockDoc({
      title: 'Nested Doc',
      description: 'A nested document',
      slug: 'nested/nested-doc',
    })],
  })];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('generateStaticParams', () => {
    it('generates params for all doc paths and their parent directories', () => {
      vi.mocked(getAllDocSlugs).mockReturnValue([
        'docs/intro',
        'docs/nested/guide',
        'docs/nested/deep/advanced',
      ]);

      const params = generateStaticParams();

      expect(params).toEqual([
        { slug: ['docs', 'intro'] },
        { slug: ['docs'] },
        { slug: ['docs', 'nested', 'guide'] },
        { slug: ['docs', 'nested'] },
        { slug: ['docs', 'nested', 'deep', 'advanced'] },
        { slug: ['docs', 'nested', 'deep'] },
      ]);
    });

    it('handles empty paths', () => {
      vi.mocked(getAllDocSlugs).mockReturnValue([]);
      expect(generateStaticParams()).toEqual([]);
    });
  });

  describe('page rendering', () => {
    it('renders docs index for directory with nested docs', async () => {
      vi.mocked(getDocsForSlug).mockReturnValue(mockGroups);
      vi.mocked(fs.existsSync).mockReturnValue(true);
      vi.mocked(fs.statSync).mockReturnValue({ isDirectory: () => true } as any);

      const page = await DocsSlugPage({ params: { slug: ['nested'] } });
      render(page, { wrapper: createWrapper() });

      // Check group structure
      expect(screen.getByRole('heading', { name: 'Nested' })).toBeInTheDocument();

      // Check doc item
      const docLink = screen.getByRole('link', { name: /Nested Doc/ });
      expect(docLink).toHaveAttribute('href', '/docs/nested/nested-doc');
      expect(screen.getByText('A nested document')).toBeInTheDocument();
    });

    it('renders docs index with multiple groups and items', async () => {
      const multipleGroups = [
        {
          title: 'Getting Started',
          key: 'getting-started',
          order: 0,
          items: [
            {
              title: 'Introduction',
              description: 'Get started here',
              slug: 'getting-started/intro',
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
              description: 'Advanced settings',
              slug: 'advanced/config',
              order: 1,
            },
          ],
        },
      ];

      vi.mocked(getDocsForSlug).mockReturnValue(multipleGroups);
      vi.mocked(fs.existsSync).mockReturnValue(true);
      vi.mocked(fs.statSync).mockReturnValue({ isDirectory: () => true } as any);

      const page = await DocsSlugPage({ params: { slug: ['docs'] } });
      render(page, { wrapper: createWrapper() });

      // Check group headings
      expect(screen.getByRole('heading', { name: 'Getting Started' })).toBeInTheDocument();
      expect(screen.getByRole('heading', { name: 'Advanced' })).toBeInTheDocument();

      // Check items in first group
      const introLink = screen.getByRole('link', { name: /Introduction/ });
      expect(introLink).toHaveAttribute('href', '/docs/getting-started/intro');
      expect(screen.getByText('Get started here')).toBeInTheDocument();

      // Check items in second group
      const configLink = screen.getByRole('link', { name: /Configuration/ });
      expect(configLink).toHaveAttribute('href', '/docs/advanced/config');
      expect(screen.getByText('Advanced settings')).toBeInTheDocument();
    });

    it('renders markdown content for document files', async () => {
      vi.mocked(getDocsForSlug).mockReturnValue([]);
      vi.mocked(fs.existsSync).mockReturnValue(true);
      vi.mocked(fs.statSync).mockReturnValue({ isDirectory: () => false } as any);
      vi.mocked(fs.readFileSync).mockReturnValue(mockMarkdownContent);

      // Mock matter with proper return value
      vi.mocked(matter).mockImplementation(() => ({
        content: '# Test Content',
        data: {},
        orig: '',
        language: '',
        matter: '',
        stringify: () => '',
      }));


      await act(async () => {
        const page = await DocsSlugPage({ params: { slug: ['test-doc'] } });
        render(page, { wrapper: createWrapper() });
      });

      // Check the rendered content
      const article = screen.getByRole('article');
      expect(article).toHaveClass('max-w-none prose prose-slate dark:prose-invert');

      // Check the heading content
      const heading = screen.getByRole('heading', { level: 1 });
      expect(heading).toHaveTextContent('Test Content');

      // Check the anchor link
      const anchor = heading.querySelector('a');
      expect(anchor).toHaveAttribute('href', '#test-content');
    });

    it('shows 404 for non-existent paths', async () => {
      vi.mocked(getDocsForSlug).mockReturnValue([]);
      // Both file and directory don't exist
      vi.mocked(fs.existsSync).mockImplementation(() => false);

      await expect(DocsSlugPage({ params: { slug: ['non-existent'] } })).rejects.toThrow('Not Found');
    });

    it('handles file read errors gracefully', async () => {
      vi.mocked(getDocsForSlug).mockReturnValue([]);
      vi.mocked(fs.existsSync).mockReturnValue(true);
      vi.mocked(fs.statSync).mockReturnValue({ isDirectory: () => false } as any);
      vi.mocked(fs.readFileSync).mockImplementation(() => {
        throw new Error('File read error');
      });

      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => { });

      await expect(DocsSlugPage({ params: { slug: ['error-doc'] } })).rejects.toThrow('Not Found');

      expect(consoleSpy).toHaveBeenCalledWith(
        'Error loading doc page for error-doc:',
        expect.any(Error)
      );

      consoleSpy.mockRestore();
    });

    it('handles markdown processing errors gracefully', async () => {
      vi.mocked(getDocsForSlug).mockReturnValue([]);
      vi.mocked(fs.existsSync).mockReturnValue(true);
      vi.mocked(fs.statSync).mockReturnValue({ isDirectory: () => false } as any);
      vi.mocked(fs.readFileSync).mockReturnValue('invalid markdown');

      // Mock matter to throw error
      vi.mocked(matter).mockImplementation(() => {
        throw new Error('Markdown processing error');
      });

      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => { });

      await expect(DocsSlugPage({ params: { slug: ['invalid-doc'] } })).rejects.toThrow('Not Found');

      expect(consoleSpy).toHaveBeenCalledWith(
        'Error loading doc page for invalid-doc:',
        expect.any(Error)
      );

      consoleSpy.mockRestore();
    });
  });
}); 