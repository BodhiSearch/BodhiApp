import fs from 'fs';
import path from 'path';
import matter from 'gray-matter';
import { getPathOrder } from './config';

const DOCS_DIR_NAME = 'src/docs';
const MD_EXTENSION = '.md';

export interface DocDetails {
  title: string;
  description: string;
  slug: string;
  order: number;
}

export interface DocGroup {
  title: string;
  items: DocDetails[];
  order: number;
  key?: string;
}

export function getAllDocPaths() {
  const docsDirectory = path.join(process.cwd(), DOCS_DIR_NAME);

  const getAllFiles = (
    dirPath: string,
    arrayOfFiles: string[] = []
  ): string[] => {
    try {
      const files = fs.readdirSync(dirPath);

      files.forEach((file) => {
        const filePath = path.join(dirPath, file);
        if (fs.statSync(filePath).isDirectory()) {
          arrayOfFiles = getAllFiles(filePath, arrayOfFiles);
        } else if (path.extname(file) === MD_EXTENSION) {
          const relativePath = path.relative(docsDirectory, filePath);
          arrayOfFiles.push(relativePath.replace(/\.md$/, ''));
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
    const fullPath = path.join(process.cwd(), 'src/docs', `${filePath}.md`);
    const fileContents = fs.readFileSync(fullPath, 'utf8');
    const { data } = matter(fileContents);
    return {
      title: data.title || formatTitle(filePath),
      description: data.description || '',
      slug: filePath,
      order: getPathOrder(filePath),
    };
  } catch (e) {
    console.error(`Error reading doc details for ${filePath}:`, e);
    return {
      title: formatTitle(filePath),
      description: '',
      slug: filePath,
      order: getPathOrder(filePath),
    };
  }
};

export const formatTitle = (path: string): string => {
  return path
    .split('/')
    .pop()!
    .replace(/-/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase());
};

export const groupDocs = (paths: string[]): DocGroup[] => {
  const groups: { [key: string]: DocGroup } = {};

  paths.forEach((path) => {
    const parts = path.split('/');
    const groupName = parts.length > 1 ? parts[0] : 'intro';
    const details = getDocDetails(path);

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

export const getDocsForPath = (slugPath: string[] | null): DocGroup[] => {
  const basePath = slugPath ? slugPath.join('/') : '';
  const paths = getAllDocPaths();

  // Filter paths that belong to the current directory
  const relevantPaths = paths.filter((path) => {
    if (!basePath) return true;
    return path.startsWith(basePath + '/') && path !== basePath;
  });

  return groupDocs(relevantPaths);
};
