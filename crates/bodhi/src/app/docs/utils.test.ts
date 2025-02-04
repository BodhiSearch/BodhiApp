import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import path from 'path';
import { getAllDocPaths, getDocDetails } from './utils';
import fs from 'fs';

// Mock the config module
vi.mock('@/app/docs/config', () => ({
  getPathOrder: vi.fn((path: string) => {
    // Mock implementation that simulates order based on path
    const orderMap: { [key: string]: number } = {
      'root-doc': 1,
      'another-doc': 2,
      'nested/nested-doc': 10,
      'nested/deep/deep-nested': 20,
      'some-path': 100,
      'no-frontmatter': 101,
      'non-existent-file': 102,
    };
    return orderMap[path] ?? 999;
  }),
}));

beforeEach(() => {
  vi.clearAllMocks();
});

const TEST_DOCS_DIR = path.join('src', 'app', 'docs', '__tests__', 'test-docs');

describe('getAllDocPaths', () => {
  const originalEnv = process.env.DOCS_DIR;

  beforeEach(() => {
    process.env.DOCS_DIR = TEST_DOCS_DIR;
  });

  afterEach(() => {
    process.env.DOCS_DIR = originalEnv;
  });

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
  const originalEnv = process.env.DOCS_DIR;

  beforeEach(() => {
    process.env.DOCS_DIR = TEST_DOCS_DIR;
  });

  afterEach(() => {
    process.env.DOCS_DIR = originalEnv;
  });

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
