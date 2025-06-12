import { Plugin } from 'vite';
import fs from 'fs';
import path from 'path';
import matter from 'gray-matter';
import { DocDetails, DocGroup, MetaData } from '../src/app/docs/types';

const MD_EXTENSION = '.md';
const DEFAULT_ORDER = 999;

interface DocsData {
  allSlugs: string[];
  docGroups: Record<string, DocGroup[]>;
  docContents: Record<string, { content: string; data: any }>;
}

function getDocsDirectory(): string[] {
  return (process.env.DOCS_DIR || 'src/docs').split('/');
}

function getPathOrder(slug: string, docsDir: string): number {
  try {
    if (slug === 'index') {
      const rootMetaPath = path.join(docsDir, '_meta.json');
      if (fs.existsSync(rootMetaPath)) {
        const metaContent = fs.readFileSync(rootMetaPath, 'utf-8');
        const meta = JSON.parse(metaContent) as MetaData;
        return meta.order ?? DEFAULT_ORDER;
      }
      return DEFAULT_ORDER;
    }

    const fullPath = path.join(docsDir, ...slug.split('/'));
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

function getAllDocSlugs(docsDirectory: string): string[] {
  const getAllFiles = (dirPath: string, arrayOfFiles: string[] = []): string[] => {
    try {
      if (!fs.existsSync(dirPath)) {
        return arrayOfFiles;
      }

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

function getDocDetails(filePath: string, docsDirectory: string): DocDetails {
  try {
    const fileContents = fs.readFileSync(filePath, 'utf8');
    const { data } = matter(fileContents);
    const derivedSlug = path.relative(docsDirectory, filePath).replace(/\.md$/, '').replaceAll(path.sep, '/');

    return {
      title: data.title || formatTitle(derivedSlug),
      description: data.description || '',
      slug: derivedSlug,
      order: getPathOrder(derivedSlug, docsDirectory),
    };
  } catch (e) {
    console.error(`Error reading doc details for ${filePath}:`, e);
    const derivedSlug = path.relative(docsDirectory, filePath).replace(/\.md$/, '').replaceAll(path.sep, '/');
    return {
      title: formatTitle(derivedSlug),
      description: '',
      slug: derivedSlug,
      order: getPathOrder(derivedSlug, docsDirectory),
    };
  }
}

function formatTitle(slug: string): string {
  return slug
    .split('/')
    .pop()!
    .replace(/-/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

function groupDocs(slugs: string[], docsDirectory: string): DocGroup[] {
  const groups: { [key: string]: DocGroup } = {};

  slugs.forEach((slug) => {
    const parts = slug.split('/');
    const groupName = parts.length > 1 ? parts[0] : 'index';
    const filePath = slug.replaceAll('/', path.sep);
    const fullPath = path.join(docsDirectory, `${filePath}.md`);
    const details = getDocDetails(fullPath, docsDirectory);

    if (!groups[groupName]) {
      groups[groupName] = {
        title: formatTitle(groupName),
        items: [],
        order: getPathOrder(groupName, docsDirectory),
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

function generateDocsData(): DocsData {
  const docsDirectory = path.join(process.cwd(), ...getDocsDirectory());
  const allSlugs = getAllDocSlugs(docsDirectory);

  const docGroups: Record<string, DocGroup[]> = {};
  const docContents: Record<string, { content: string; data: any }> = {};

  // Generate doc groups for different base paths
  docGroups[''] = groupDocs(allSlugs, docsDirectory);

  // Generate content for each doc
  allSlugs.forEach((slug) => {
    const filePath = path.join(docsDirectory, `${slug.replaceAll('/', path.sep)}.md`);
    if (fs.existsSync(filePath)) {
      const fileContent = fs.readFileSync(filePath, 'utf-8');
      const { content, data } = matter(fileContent);
      docContents[slug] = { content, data };
    }
  });

  return {
    allSlugs,
    docGroups,
    docContents,
  };
}

export function docsGeneratorPlugin(): Plugin {
  return {
    name: 'docs-generator',
    buildStart() {
      // Generate docs data at build time
      const docsData = generateDocsData();

      // Write the generated data to a virtual module
      const outputPath = path.join(process.cwd(), 'src/generated/docs-data.json');
      const outputDir = path.dirname(outputPath);

      if (!fs.existsSync(outputDir)) {
        fs.mkdirSync(outputDir, { recursive: true });
      }

      fs.writeFileSync(outputPath, JSON.stringify(docsData, null, 2));
      console.log('📚 Generated docs data for', docsData.allSlugs.length, 'documents');
    },
    resolveId(id) {
      if (id === 'virtual:docs-data') {
        return id;
      }
    },
    load(id) {
      if (id === 'virtual:docs-data') {
        const docsData = generateDocsData();
        return `export default ${JSON.stringify(docsData)};`;
      }
    },
  };
}
