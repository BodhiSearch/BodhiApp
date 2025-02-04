import { DocsIndex } from '@/app/docs/DocsIndex';
import { getAllDocPaths, getDocsForPath } from '@/app/docs/utils';
import fs from 'fs';
import matter from 'gray-matter';
import Link from 'next/link';
import { notFound } from 'next/navigation';
import path from 'path';
import rehypeAutolinkHeadings from 'rehype-autolink-headings';
import rehypePrism from 'rehype-prism-plus';
import rehypeSlug from 'rehype-slug';
import rehypeStringify from 'rehype-stringify';
import remarkGfm from 'remark-gfm';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import { unified } from 'unified';

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

async function markdownToHtml(content: string) {
  const result = await unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkRehype, { allowDangerousHtml: true })
    .use(rehypeSlug)
    .use(rehypeAutolinkHeadings)
    .use(rehypePrism, {
      showLineNumbers: true,
      ignoreMissing: true,
    })
    .use(rehypeStringify, { allowDangerousHtml: true })
    .process(content);

  return result.toString();
}

interface DocsSlugPageProps {
  params: {
    slug: string[];
  };
}

export default async function DocsSlugPage({ params }: DocsSlugPageProps) {
  const sortedGroups = getDocsForPath(params.slug);

  // If there are nested docs, show the index
  if (sortedGroups.length > 0) {
    return (
      <DocsIndex
        groups={sortedGroups}
        title={`${params.slug[params.slug.length - 1]} Documentation`}
        description={`Documentation for ${params.slug.join('/')}`}
      />
    );
  }

  // Otherwise, render the document content
  const slug = params.slug.join('/');
  const filePath = path.join(process.cwd(), 'src/docs', `${slug}.md`);
  const dirPath = path.join(process.cwd(), 'src/docs', slug);

  try {
    // Check if it's a directory first
    if (fs.existsSync(dirPath) && fs.statSync(dirPath).isDirectory()) {
      const sortedGroups = getDocsForPath(params.slug);
      const title = params.slug[params.slug.length - 1]
        .replace(/-/g, ' ')
        .replace(/\b\w/g, (c) => c.toUpperCase());

      return (
        <div className="max-w-none prose prose-slate dark:prose-invert">
          <h1>{title}</h1>

          {sortedGroups.map((group) => (
            <section key={group.key} className="mb-12">
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {group.items.map((doc) => (
                  <Link
                    key={doc.slug}
                    href={`/docs/${doc.slug}`}
                    className="block p-4 border rounded-lg hover:border-blue-500 transition-colors no-underline"
                  >
                    <h3 className="text-lg font-semibold mb-1 mt-0">
                      {doc.title}
                    </h3>
                    {doc.description && (
                      <p className="text-sm text-gray-600 dark:text-gray-400 m-0">
                        {doc.description}
                      </p>
                    )}
                  </Link>
                ))}
              </div>
            </section>
          ))}
        </div>
      );
    }

    // If not a directory, treat as a markdown file
    const fileContents = fs.readFileSync(filePath, 'utf8');
    const { content } = matter(fileContents);
    const htmlContent = await markdownToHtml(content);

    return (
      <article
        className="max-w-none prose prose-slate dark:prose-invert
        prose-headings:font-semibold
        prose-h1:text-3xl
        prose-h2:text-2xl
        prose-h3:text-xl
        prose-pre:bg-gray-800
        prose-pre:border
        prose-pre:border-gray-700
        prose-code:text-blue-500
        prose-code:before:content-none
        prose-code:after:content-none
        prose-blockquote:border-l-4
        prose-blockquote:border-gray-300
        prose-blockquote:pl-4
        prose-blockquote:italic
        prose-img:rounded-lg
        prose-a:text-blue-600
        hover:prose-a:text-blue-500"
        dangerouslySetInnerHTML={{ __html: htmlContent }}
      />
    );
  } catch (e) {
    console.error(`Error loading doc page for ${slug}:`, e);
    notFound();
  }
}
