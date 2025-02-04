import { getPathOrder } from '@/app/docs/config';
import path from 'path';
import { describe, expect, it } from 'vitest';

const TEST_DOCS_DIR = path.join('src', 'app', 'docs', '__tests__', 'test-docs');

describe('config', () => {
  describe('getPathOrder', () => {
    const originalEnv = process.env.DOCS_DIR;

    beforeEach(() => {
      process.env.DOCS_DIR = TEST_DOCS_DIR;
    });

    afterEach(() => {
      process.env.DOCS_DIR = originalEnv;
    });

    it('returns order from _meta.json for directories', () => {
      const order = getPathOrder('nested');
      expect(order).toBe(100);
    });

    it('returns order from _meta.json for nested directories', () => {
      const order = getPathOrder('nested/deep');
      expect(order).toBe(150);
    });

    it('returns order from frontmatter for root markdown files', () => {
      const order = getPathOrder('root-doc');
      expect(order).toBe(10);
    });

    it('returns order from frontmatter for nested markdown files', () => {
      const order = getPathOrder('nested/nested-doc');
      expect(order).toBe(110);
    });

    it('returns order from frontmatter for deeply nested markdown files', () => {
      const order = getPathOrder('nested/deep/deep-nested');
      expect(order).toBe(160);
    });

    it('returns default order when no order is specified', () => {
      // not-markdown.txt has no order
      const order = getPathOrder('not-markdown');
      expect(order).toBe(999);
    });

    it('returns default order when file does not exist', () => {
      const order = getPathOrder('non-existent');
      expect(order).toBe(999);
    });

    it('returns order from root _meta.json for index path', () => {
      const order = getPathOrder('index');
      expect(order).toBe(-1);
    });
  });
});
