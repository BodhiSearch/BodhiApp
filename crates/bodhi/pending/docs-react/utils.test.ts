import fs from 'fs';
import path from 'path';

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { getAllDocSlugs, getDocDetails, getDocsForSlug } from '@/app/docs/utils';

// Constants
const TEST_DOCS_DIR = path.join('src', 'app', 'docs', '__tests__', 'test-docs');
const EMPTY_TEST_DOCS_DIR = path.join('src', 'app', 'docs', '__tests__', 'empty-test-docs');

describe('Documentation Utils', () => {
  const originalEnv = process.env.DOCS_DIR;

  beforeEach(() => {
    vi.clearAllMocks();
    process.env.DOCS_DIR = TEST_DOCS_DIR;
  });

  afterEach(() => {
    process.env.DOCS_DIR = originalEnv;
  });

  describe('getAllDocSlugs', () => {
    it('should return all markdown files paths without extension', () => {
      const expectedPaths = ['root-doc', 'another-doc', 'nested/nested-doc', 'nested/deep/deep-nested'].sort();

      const paths = getAllDocSlugs().sort();

      expect(paths).toEqual(expectedPaths);
    });

    it('should handle non-existent directory gracefully', () => {
      process.env.DOCS_DIR = '__tests__/non-existent';

      const consoleSpy = vi.spyOn(console, 'error');
      const paths = getAllDocSlugs();

      expect(paths).toEqual([]);
      expect(consoleSpy).toHaveBeenCalledWith('Error reading docs directory:', expect.any(Error));

      consoleSpy.mockRestore();
    });

    it('should ignore non-markdown files', () => {
      const paths = getAllDocSlugs();

      expect(paths).not.toContain('not-markdown');
    });
  });

  describe('getDocDetails', () => {
    describe('successful cases', () => {
      it('should return doc details from frontmatter', () => {
        const fullPath = path.join(process.cwd(), TEST_DOCS_DIR, 'root-doc.md');
        const details = getDocDetails(fullPath);

        expect(details).toEqual({
          title: 'Root Document',
          description: 'A test root level document',
          slug: 'root-doc',
          order: 10,
        });
      });

      it('should handle nested doc paths', () => {
        const fullPath = path.join(process.cwd(), TEST_DOCS_DIR, 'nested', 'nested-doc.md');
        const details = getDocDetails(fullPath);

        expect(details).toEqual({
          title: 'Nested Document',
          description: 'A nested test document',
          slug: 'nested/nested-doc',
          order: 110,
        });
      });

      it('should handle deeply nested doc paths', () => {
        const fullPath = path.join(process.cwd(), TEST_DOCS_DIR, 'nested', 'deep', 'deep-nested.md');
        const details = getDocDetails(fullPath);

        expect(details).toEqual({
          title: 'Deep Nested',
          description: 'A deeply nested test document',
          slug: 'nested/deep/deep-nested',
          order: 160,
        });
      });

      it('should format title if no frontmatter title exists', () => {
        // Create a temporary test file
        const testFilePath = path.join(TEST_DOCS_DIR, 'no-title.md');
        fs.writeFileSync(
          testFilePath,
          `---
description: "Some description"
---
# Content`
        );

        try {
          const fullPath = path.join(process.cwd(), TEST_DOCS_DIR, 'no-title.md');
          const details = getDocDetails(fullPath);

          expect(details).toEqual({
            title: 'No Title',
            description: 'Some description',
            slug: 'no-title',
            order: 999,
          });
        } finally {
          // Clean up - remove the temporary file
          fs.unlinkSync(path.join(TEST_DOCS_DIR, 'no-title.md'));
        }
      });

      it('should handle missing frontmatter', () => {
        // Create a temporary test file
        const testFilePath = path.join(TEST_DOCS_DIR, 'no-frontmatter.md');
        fs.writeFileSync(testFilePath, '# Just content without frontmatter');

        try {
          const fullPath = path.join(process.cwd(), TEST_DOCS_DIR, 'no-frontmatter.md');
          const details = getDocDetails(fullPath);

          expect(details).toEqual({
            title: 'No Frontmatter',
            description: '',
            slug: 'no-frontmatter',
            order: 999,
          });
        } finally {
          // Clean up - remove the temporary file
          fs.unlinkSync(path.join(TEST_DOCS_DIR, 'no-frontmatter.md'));
        }
      });
    });

    describe('error handling', () => {
      it('should handle file read errors gracefully', () => {
        const consoleSpy = vi.spyOn(console, 'error');
        const fullPath = path.join(process.cwd(), TEST_DOCS_DIR, 'non-existent-file.md');
        const details = getDocDetails(fullPath);

        expect(details).toEqual({
          title: 'Non Existent File',
          description: '',
          slug: 'non-existent-file',
          order: 999,
        });

        expect(consoleSpy).toHaveBeenCalledWith(
          expect.stringContaining('Error reading doc details for'),
          expect.any(Error)
        );

        consoleSpy.mockRestore();
      });
    });
  });

  describe('getDocsForSlug', () => {
    describe('grouping behavior', () => {
      it('should return all docs grouped when no path is provided', () => {
        const groups = getDocsForSlug(null);

        expect(groups).toEqual([
          {
            title: 'Index',
            items: [
              {
                title: 'Root Document',
                description: 'A test root level document',
                slug: 'root-doc',
                order: 10,
              },
              {
                title: 'Another Document',
                description: 'Another test document',
                slug: 'another-doc',
                order: 20,
              },
            ],
            order: -1,
            key: 'index',
          },
          {
            title: 'Nested',
            items: [
              {
                title: 'Nested Document',
                description: 'A nested test document',
                slug: 'nested/nested-doc',
                order: 110,
              },
              {
                title: 'Deep Nested',
                description: 'A deeply nested test document',
                slug: 'nested/deep/deep-nested',
                order: 160,
              },
            ],
            order: 100,
            key: 'nested',
          },
        ]);
      });

      it('should return filtered docs for a specific path', () => {
        const groups = getDocsForSlug(['nested']);

        expect(groups).toEqual([
          {
            title: 'Nested',
            items: [
              {
                title: 'Nested Document',
                description: 'A nested test document',
                slug: 'nested/nested-doc',
                order: 110,
              },
              {
                title: 'Deep Nested',
                description: 'A deeply nested test document',
                slug: 'nested/deep/deep-nested',
                order: 160,
              },
            ],
            order: 100,
            key: 'nested',
          },
        ]);
      });

      it('should return deeply nested docs for a specific path', () => {
        const groups = getDocsForSlug(['nested', 'deep']);

        expect(groups).toEqual([
          {
            title: 'Nested',
            items: [
              {
                title: 'Deep Nested',
                description: 'A deeply nested test document',
                slug: 'nested/deep/deep-nested',
                order: 160,
              },
            ],
            order: 100,
            key: 'nested',
          },
        ]);
      });
    });

    describe('filtering behavior', () => {
      it('should exclude the current path from results', () => {
        const groups = getDocsForSlug(['nested', 'nested-doc']);

        expect(groups.flatMap((g) => g.items).map((i) => i.slug)).not.toContain('nested/nested-doc');
      });
    });

    describe('sorting behavior', () => {
      it('should sort groups by order', () => {
        const groups = getDocsForSlug(null);

        expect(groups.map((g) => g.key)).toEqual(['index', 'nested']);
      });

      it('should sort items within groups by order', () => {
        const groups = getDocsForSlug(['nested']);

        expect(groups[0].items.map((item) => item.slug)).toEqual(['nested/nested-doc', 'nested/deep/deep-nested']);
      });
    });

    describe('error handling', () => {
      it('should handle empty directory gracefully', () => {
        process.env.DOCS_DIR = EMPTY_TEST_DOCS_DIR;
        expect(getDocsForSlug(null)).toEqual([]);
      });

      it('should return empty array for non-existent path', () => {
        expect(getDocsForSlug(['non-existent'])).toEqual([]);
      });
    });
  });
});
