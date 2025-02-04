import fs from 'fs';
import matter from 'gray-matter';
import path from 'path';
import { getDocsDirectory } from './utils';

interface MetaData {
  order: number;
}

const DEFAULT_ORDER = 999;

export function getPathOrder(docPath: string): number {
  try {
    const docsDir = getDocsDirectory();

    // Special case for index - read from root _meta.json
    if (docPath === 'index') {
      const rootMetaPath = path.join(process.cwd(), docsDir, '_meta.json');
      if (fs.existsSync(rootMetaPath)) {
        const metaContent = fs.readFileSync(rootMetaPath, 'utf-8');
        const meta = JSON.parse(metaContent) as MetaData;
        return meta.order ?? DEFAULT_ORDER;
      }
      return DEFAULT_ORDER;
    }

    const fullPath = path.join(process.cwd(), docsDir, docPath);

    // Check if it's a directory
    const isDirectory =
      fs.existsSync(fullPath) && fs.statSync(fullPath).isDirectory();

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
    console.error(`Error getting order for path ${docPath}:`, error);
    return DEFAULT_ORDER;
  }
}
