import { DocDetails, DocGroup, MetaData } from '@/components/docs/types';
import fs from 'fs';
import matter from 'gray-matter';
import path from 'path';

const MD_EXTENSION = '.md';
const DEFAULT_ORDER = 999;

// sends the path of the docs directory as array of strings
function getDocsDirectory(): string[] {
  return (process.env.DOCS_DIR || 'src/docs').split('/');
}

// takes in the slug and returns the order
export function getPathOrder(slug: string): number {
  const rootDocs = getDocsDirectory();
  try {
    // Special case for index - read from root _meta.json
    if (slug === 'index') {
      const rootMetaPath = path.join(...rootDocs, '_meta.json');
      if (fs.existsSync(rootMetaPath)) {
        const metaContent = fs.readFileSync(rootMetaPath, 'utf-8');
        const meta = JSON.parse(metaContent) as MetaData;
        return meta.order ?? DEFAULT_ORDER;
      }
      return DEFAULT_ORDER;
    }

    const fullPath = path.join(...rootDocs, ...slug.split('/'));

    // Check if it's a directory
    const isDirectory = fs.existsSync(fullPath) && fs.statSync(fullPath).isDirectory();

    if (isDirectory) {
      const metaPath = path.join(fullPath, '_meta.json');
      if (fs.existsSync(metaPath)) {
        const metaContent = fs.readFileSync(metaPath, 'utf-8');
        const meta = JSON.parse(metaContent) as MetaData;
        return meta.order ?? DEFAULT_ORDER;
      }
    } else {
      // Check if it's a markdown file
      const mdPath = `${fullPath}.md`;
      if (fs.existsSync(mdPath)) {
        const fileContent = fs.readFileSync(mdPath, 'utf-8');
        const { data } = matter(fileContent);
        return data.order ?? DEFAULT_ORDER;
      }
    }

    return DEFAULT_ORDER;
  } catch (error) {
    console.error(`Error getting order for path ${slug}:`, error);
    return DEFAULT_ORDER;
  }
}

// returns all the slugs of the given docs directory
export function getAllDocSlugs() {
  const docsDirectory = path.join(process.cwd(), ...getDocsDirectory());

  const getAllFiles = (dirPath: string, arrayOfFiles: string[] = []): string[] => {
    try {
      const files = fs.readdirSync(dirPath);

      files.forEach((file) => {
        const filePath = path.join(dirPath, file);
        if (fs.statSync(filePath).isDirectory()) {
          arrayOfFiles = getAllFiles(filePath, arrayOfFiles);
        } else if (path.extname(file) === MD_EXTENSION) {
          const relativePath = path.relative(docsDirectory, filePath);
          const pathSlug = relativePath.replace(/\.md$/, '').replaceAll(path.sep, '/');
          arrayOfFiles.push(pathSlug);
        }
      });

      return arrayOfFiles;
    } catch (e) {
      console.error('Error reading docs directory:', e);
      return [];
    }
  };

  return getAllFiles(docsDirectory);
}

// takes in the doc full path and returns the details
export const getDocDetails = (filePath: string): DocDetails => {
  try {
    const fileContents = fs.readFileSync(filePath, 'utf8');
    const { data } = matter(fileContents);
    const docsDirectory = path.join(process.cwd(), ...getDocsDirectory());
    const derivedSlug = path.relative(docsDirectory, filePath).replace(/\.md$/, '').replaceAll(path.sep, '/');

    return {
      title: data.title || formatTitle(derivedSlug),
      description: data.description || '',
      slug: derivedSlug,
      order: getPathOrder(derivedSlug),
    };
  } catch (e) {
    console.error(`Error reading doc details for ${filePath}:`, e);
    const docsDirectory = path.join(process.cwd(), ...getDocsDirectory());
    const derivedSlug = path.relative(docsDirectory, filePath).replace(/\.md$/, '').replaceAll(path.sep, '/');
    return {
      title: formatTitle(derivedSlug),
      description: '',
      slug: derivedSlug,
      order: getPathOrder(derivedSlug),
    };
  }
};

export const formatTitle = (slug: string): string => {
  return slug
    .split('/')
    .pop()!
    .replace(/-/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase());
};

// takes in slug paths and return grouped docs
const groupDocs = (slugs: string[]): DocGroup[] => {
  const groups: { [key: string]: DocGroup } = {};
  const docsDirectory = path.join(process.cwd(), ...getDocsDirectory());

  slugs.forEach((slug) => {
    const parts = slug.split('/');
    const groupName = parts.length > 1 ? parts[0] : 'index';
    const filePath = slug.replaceAll('/', path.sep);
    const fullPath = path.join(docsDirectory, `${filePath}.md`);
    const details = getDocDetails(fullPath);

    if (!groups[groupName]) {
      groups[groupName] = {
        title: formatTitle(groupName),
        items: [],
        order: getPathOrder(groupName),
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
};

// returns the docs for all the slug paths
export const getDocsForSlug = (slugPath: string[] | null): DocGroup[] => {
  const basePath = slugPath ? slugPath.join('/') : '';
  const slugs = getAllDocSlugs();

  // Filter paths that belong to the current directory
  const relevantSlugs = slugs.filter((slug) => {
    if (!basePath) return true;
    return slug.startsWith(basePath + '/') && slug !== basePath;
  });

  return groupDocs(relevantSlugs);
};
