import { DOCS_CONFIG, PROSE_CLASSES } from '@/app/docs/constants';
import { DocsIndex } from '@/app/docs/DocsIndex';
import { markdownService } from '@/app/docs/markdown';
import { getAllDocPaths, getDocsForPath } from '@/app/docs/utils';
import fs from 'fs';
import { notFound } from 'next/navigation';
import path from 'path';

// Generate static paths for all markdown files
export function generateStaticParams() {
  const paths = getAllDocPaths();
  const allPaths = new Set<string>();

  paths.forEach((path) => {
    // Add the full path
    allPaths.add(path);

    // Add all parent directory paths
    const parts = path.split('/');
    for (let i = 1; i < parts.length; i++) {
      allPaths.add(parts.slice(0, i).join('/'));
    }
  });

  return Array.from(allPaths).map((path) => ({
    slug: path.split('/'),
  }));
}

interface DocsSlugPageProps {
  params: {
    slug: string[];
  };
}

export default async function DocsSlugPage({ params }: DocsSlugPageProps) {
  const sortedGroups = getDocsForPath(params.slug);
  const slug = params.slug.join('/');
  const filePath = path.join(process.cwd(), DOCS_CONFIG.rootPath, `${slug}.md`);
  const dirPath = path.join(process.cwd(), DOCS_CONFIG.rootPath, slug);

  try {
    if (sortedGroups.length > 0) {
      return <DocsIndex groups={sortedGroups} />;
    }

    if (!fs.existsSync(dirPath) && !fs.existsSync(filePath)) {
      notFound();
      return null;
    }

    if (fs.existsSync(dirPath) && fs.statSync(dirPath).isDirectory()) {
      return <DocsIndex groups={sortedGroups} />;
    }

    const fileContents = fs.readFileSync(filePath, 'utf8');
    const { content } = markdownService.parseMarkdownFile(fileContents);
    const htmlContent = await markdownService.processMarkdown(content);

    return (
      <article
        className={PROSE_CLASSES.root}
        dangerouslySetInnerHTML={{ __html: htmlContent }}
      />
    );
  } catch (e) {
    console.error(`Error loading doc page for ${slug}:`, e);
    notFound();
    return null;
  }
}
