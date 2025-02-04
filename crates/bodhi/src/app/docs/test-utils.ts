import { DocDetails, DocGroup } from '@/app/docs/types';

export const createMockDoc = (overrides?: Partial<DocDetails>) => ({
  title: 'Test Doc',
  description: 'Test description',
  slug: 'test-doc',
  order: 1,
  ...overrides,
});

export const createMockGroup = (overrides?: Partial<DocGroup>) => ({
  title: 'Test Group',
  key: 'test-group',
  order: 0,
  items: [createMockDoc()],
  ...overrides,
});

export const mockMarkdownContent = `---
title: Test Title
description: Test Description
---

# Test Content

This is test content.`;
