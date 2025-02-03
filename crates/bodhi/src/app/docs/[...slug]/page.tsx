import fs from 'fs';
import matter from 'gray-matter';
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
import { getAllDocPaths } from '@/app/docs/utils';

// Generate static paths for all markdown files
export function generateStaticParams() {
  const paths = getAllDocPaths();
  return paths.map((path) => ({
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

// Page component
export default async function DocPage({
  params,
}: {
  params: { slug: string[] };
}) {
  const slug = params.slug.join('/');
  const filePath = path.join(process.cwd(), 'src/docs', `${slug}.md`);

  try {
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
