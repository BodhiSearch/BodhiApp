// Client-side docs utilities that work with pre-generated data
import { DocDetails, DocGroup } from '@/components/docs/types';

// Import the generated docs data
let docsData: {
  allSlugs: string[];
  docGroups: Record<string, DocGroup[]>;
  docContents: Record<string, { content: string; data: any }>;
} | null = null;

// Lazy load the docs data
async function getDocsData() {
  if (!docsData) {
    try {
      // Try to import from virtual module first (build time)
      docsData = await import('virtual:docs-data').then((m) => m.default);
    } catch {
      // Fallback to generated JSON file
      try {
        const response = await fetch('/src/generated/docs-data.json');
        docsData = await response.json();
      } catch (error) {
        console.error('Failed to load docs data:', error);
        docsData = { allSlugs: [], docGroups: {}, docContents: {} };
      }
    }
  }
  return docsData;
}

export async function getAllDocSlugs(): Promise<string[]> {
  const data = await getDocsData();
  return data?.allSlugs || [];
}

export async function getDocsForSlug(
  slugPath: string[] | null
): Promise<DocGroup[]> {
  const data = await getDocsData();
  if (!data) return [];

  const basePath = slugPath ? slugPath.join('/') : '';

  // Return pre-computed groups or filter from all slugs
  if (data.docGroups[basePath]) {
    return data.docGroups[basePath];
  }

  // Fallback: filter and group on client side
  const relevantSlugs = data.allSlugs.filter((slug) => {
    if (!basePath) return true;
    return slug.startsWith(basePath + '/') && slug !== basePath;
  });

  return groupDocsClient(relevantSlugs, data);
}

export async function getDocContent(
  slug: string
): Promise<{ content: string; data: any } | null> {
  const data = await getDocsData();
  return data?.docContents[slug] || null;
}

export async function getDocDetails(slug: string): Promise<DocDetails | null> {
  const data = await getDocsData();
  if (!data) return null;

  const content = data.docContents[slug];

  if (!content) return null;

  return {
    title: content.data.title || formatTitle(slug),
    description: content.data.description || '',
    slug,
    order: content.data.order ?? 999,
  };
}

function formatTitle(slug: string): string {
  return slug
    .split('/')
    .pop()!
    .replace(/-/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

function groupDocsClient(slugs: string[], data: any): DocGroup[] {
  const groups: { [key: string]: DocGroup } = {};

  slugs.forEach((slug) => {
    const parts = slug.split('/');
    const groupName = parts.length > 1 ? parts[0] : 'index';
    const content = data.docContents[slug];

    if (!content) return;

    const details: DocDetails = {
      title: content.data.title || formatTitle(slug),
      description: content.data.description || '',
      slug,
      order: content.data.order ?? 999,
    };

    if (!groups[groupName]) {
      groups[groupName] = {
        title: formatTitle(groupName),
        items: [],
        order: 999, // Default order for groups
      };
    }

    groups[groupName].items.push(details);
  });

  // Sort items within each group
  Object.values(groups).forEach((group) => {
    group.items.sort((a, b) => a.order - b.order);
  });

  // Convert groups object to sorted array
  return Object.entries(groups)
    .map(([key, group]) => ({
      ...group,
      key,
    }))
    .sort((a, b) => a.order - b.order);
}

// Utility functions that were previously server-side only
export function getPathOrder(_slug: string): number {
  // This is now handled at build time, but we can provide a fallback
  return 999;
}

export { formatTitle };
