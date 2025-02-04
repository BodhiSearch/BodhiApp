import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import path from 'path';
import { getAllDocPaths } from './utils';

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
      'install',
      'intro',
      'developer-docs/api-reference',
      'developer-docs/authentication',
      'developer-docs/intro',
      'developer-docs/model-configuration',
      'features/api-tokens',
      'features/chat-ui',
      'features/model-alias',
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
    // The test docs directory should include a non-markdown file
    const paths = getAllDocPaths();

    expect(paths).not.toContain('ignore-me');
  });
});
