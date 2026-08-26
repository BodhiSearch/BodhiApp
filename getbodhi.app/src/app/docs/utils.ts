import fs from 'fs';
import path from 'path';

import matter from 'gray-matter';

import { DocDetails, DocGroup, MetaData } from '@/app/docs/types';

const MD_EXTENSION = '.md';
const DEFAULT_ORDER = 999;

function getDocsDirectory(): string[] {
  return (process.env.DOCS_DIR || 'src/docs').split('/');
}

export function getPathOrder(slug: string): number {
  const rootDocs = getDocsDirectory();
  try {
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
    const isDirectory = fs.existsSync(fullPath) && fs.statSync(fullPath).isDirectory();

    if (isDirectory) {
      const metaPath = path.join(fullPath, '_meta.json');
      if (fs.existsSync(metaPath)) {
        const metaContent = fs.readFileSync(metaPath, 'utf-8');
        const meta = JSON.parse(metaContent) as MetaData;
        return meta.order ?? DEFAULT_ORDER;
      }
    } else {
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

  Object.values(groups).forEach((group) => {
    group.items.sort((a, b) => a.order - b.order);
  });

  return Object.entries(groups)
    .map(([key, group]) => ({
      ...group,
      key,
    }))
    .sort((a, b) => a.order - b.order);
};

export const getDocsForSlug = (slugPath: string[] | null): DocGroup[] => {
  const basePath = slugPath ? slugPath.join('/') : '';
  const slugs = getAllDocSlugs();

  const relevantSlugs = slugs.filter((slug) => {
    if (!basePath) return true;
    return slug.startsWith(basePath + '/') && slug !== basePath;
  });

  return groupDocs(relevantSlugs);
};
