import {
  getAllDocPaths,
  getDocDetails,
  getDocsForPath,
} from '@/app/docs/utils';
import fs from 'fs';
import path from 'path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// Constants
const TEST_DOCS_DIR = path.join('src', 'app', 'docs', '__tests__', 'test-docs');
const EMPTY_TEST_DOCS_DIR = path.join(
  'src',
  'app',
  'docs',
  '__tests__',
  'empty-test-docs'
);

// Mock Configuration
const mockOrderMap = {
  // Group orders
  index: 0,
  nested: 1,

  // Root level docs
  'root-doc': 1,
  'another-doc': 2,

  // Nested docs
  'nested/nested-doc': 10,
  'nested/deep/deep-nested': 20,

  // Mock test paths
  'some-path': 100,
  'no-frontmatter': 101,
  'non-existent-file': 102,
};

vi.mock('@/app/docs/config', () => ({
  getPathOrder: vi.fn((path: string) => mockOrderMap[path] ?? 999),
}));

describe('Documentation Utils', () => {
  const originalEnv = process.env.DOCS_DIR;

  beforeEach(() => {
    vi.clearAllMocks();
    process.env.DOCS_DIR = TEST_DOCS_DIR;
  });

  afterEach(() => {
    process.env.DOCS_DIR = originalEnv;
  });

  describe('getAllDocPaths', () => {
    it('should return all markdown files paths without extension', () => {
      const expectedPaths = [
        'root-doc',
        'another-doc',
        'nested/nested-doc',
        'nested/deep/deep-nested',
      ].sort();

      const paths = getAllDocPaths().sort();

      expect(paths).toEqual(expectedPaths);
    });

    it('should handle non-existent directory gracefully', () => {
      process.env.DOCS_DIR = '__tests__/non-existent';

      const consoleSpy = vi.spyOn(console, 'error');
      const paths = getAllDocPaths();

      expect(paths).toEqual([]);
      expect(consoleSpy).toHaveBeenCalledWith(
        'Error reading docs directory:',
        expect.any(Error)
      );

      consoleSpy.mockRestore();
    });

    it('should ignore non-markdown files', () => {
      const paths = getAllDocPaths();

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
          order: 1,
        });
      });

      it('should handle nested doc paths', () => {
        const fullPath = path.join(
          process.cwd(),
          TEST_DOCS_DIR,
          'nested',
          'nested-doc.md'
        );
        const details = getDocDetails(fullPath);

        expect(details).toEqual({
          title: 'Nested Document',
          description: 'A nested test document',
          slug: 'nested/nested-doc',
          order: 10,
        });
      });

      it('should handle deeply nested doc paths', () => {
        const fullPath = path.join(
          process.cwd(),
          TEST_DOCS_DIR,
          'nested',
          'deep',
          'deep-nested.md'
        );
        const details = getDocDetails(fullPath);

        expect(details).toEqual({
          title: 'Deep Nested',
          description: 'A deeply nested test document',
          slug: 'nested/deep/deep-nested',
          order: 20,
        });
      });

      it('should format title if no frontmatter title exists', async () => {
        const mockFs = vi.spyOn(fs, 'readFileSync');
        mockFs.mockReturnValueOnce(`---
description: "Some description"
---
# Content`);

        const fullPath = path.join(
          process.cwd(),
          TEST_DOCS_DIR,
          'some-path.md'
        );
        const details = getDocDetails(fullPath);

        expect(details).toEqual({
          title: 'Some Path',
          description: 'Some description',
          slug: 'some-path',
          order: 100,
        });
        mockFs.mockRestore();
      });

      it('should handle missing frontmatter', async () => {
        const mockFs = vi.spyOn(fs, 'readFileSync');
        mockFs.mockReturnValueOnce('# Just content without frontmatter');

        const fullPath = path.join(
          process.cwd(),
          TEST_DOCS_DIR,
          'no-frontmatter.md'
        );
        const details = getDocDetails(fullPath);

        expect(details).toEqual({
          title: 'No Frontmatter',
          description: '',
          slug: 'no-frontmatter',
          order: 101,
        });
        mockFs.mockRestore();
      });
    });

    describe('error handling', () => {
      it('should handle file read errors gracefully', () => {
        const consoleSpy = vi.spyOn(console, 'error');
        const fullPath = path.join(
          process.cwd(),
          TEST_DOCS_DIR,
          'non-existent-file.md'
        );
        const details = getDocDetails(fullPath);

        expect(details).toEqual({
          title: 'Non Existent File',
          description: '',
          slug: 'non-existent-file',
          order: 102,
        });

        expect(consoleSpy).toHaveBeenCalledWith(
          expect.stringContaining('Error reading doc details for'),
          expect.any(Error)
        );

        consoleSpy.mockRestore();
      });
    });
  });

  describe('getDocsForPath', () => {
    describe('grouping behavior', () => {
      it('should return all docs grouped when no path is provided', () => {
        const groups = getDocsForPath(null);

        expect(groups).toEqual([
          {
            title: 'Index',
            items: [
              {
                title: 'Root Document',
                description: 'A test root level document',
                slug: 'root-doc',
                order: 1,
              },
              {
                title: 'Another Document',
                description: 'Another test document',
                slug: 'another-doc',
                order: 2,
              },
            ],
            order: 0,
            key: 'index',
          },
          {
            title: 'Nested',
            items: [
              {
                title: 'Nested Document',
                description: 'A nested test document',
                slug: 'nested/nested-doc',
                order: 10,
              },
              {
                title: 'Deep Nested',
                description: 'A deeply nested test document',
                slug: 'nested/deep/deep-nested',
                order: 20,
              },
            ],
            order: 1,
            key: 'nested',
          },
        ]);
      });

      it('should return filtered docs for a specific path', () => {
        const groups = getDocsForPath(['nested']);

        expect(groups).toEqual([
          {
            title: 'Nested',
            items: [
              {
                title: 'Nested Document',
                description: 'A nested test document',
                slug: 'nested/nested-doc',
                order: 10,
              },
              {
                title: 'Deep Nested',
                description: 'A deeply nested test document',
                slug: 'nested/deep/deep-nested',
                order: 20,
              },
            ],
            order: 1,
            key: 'nested',
          },
        ]);
      });

      it('should return deeply nested docs for a specific path', () => {
        const groups = getDocsForPath(['nested', 'deep']);

        expect(groups).toEqual([
          {
            title: 'Nested',
            items: [
              {
                title: 'Deep Nested',
                description: 'A deeply nested test document',
                slug: 'nested/deep/deep-nested',
                order: 20,
              },
            ],
            order: 1,
            key: 'nested',
          },
        ]);
      });
    });

    describe('filtering behavior', () => {
      it('should exclude the current path from results', () => {
        const groups = getDocsForPath(['nested', 'nested-doc']);

        expect(groups.flatMap((g) => g.items).map((i) => i.slug)).not.toContain(
          'nested/nested-doc'
        );
      });
    });

    describe('sorting behavior', () => {
      it('should sort groups by order', () => {
        const groups = getDocsForPath(null);

        expect(groups.map((g) => g.key)).toEqual(['index', 'nested']);
      });

      it('should sort items within groups by order', () => {
        const groups = getDocsForPath(['nested']);

        expect(groups[0].items.map((item) => item.slug)).toEqual([
          'nested/nested-doc',
          'nested/deep/deep-nested',
        ]);
      });
    });

    describe('error handling', () => {
      it('should handle empty directory gracefully', () => {
        process.env.DOCS_DIR = EMPTY_TEST_DOCS_DIR;
        expect(getDocsForPath(null)).toEqual([]);
      });

      it('should return empty array for non-existent path', () => {
        expect(getDocsForPath(['non-existent'])).toEqual([]);
      });
    });
  });
});
