import { DOCS_CONFIG } from '@/app/docs/constants';
import matter from 'gray-matter';
import rehypeAutolinkHeadings from 'rehype-autolink-headings';
import rehypePrism from 'rehype-prism-plus';
import rehypeSlug from 'rehype-slug';
import rehypeStringify from 'rehype-stringify';
import remarkGfm from 'remark-gfm';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import { unified } from 'unified';

export class MarkdownService {
  private static instance: MarkdownService;
  private cache = new Map<string, string>();

  private constructor() {}

  static getInstance(): MarkdownService {
    if (!MarkdownService.instance) {
      MarkdownService.instance = new MarkdownService();
    }
    return MarkdownService.instance;
  }

  async processMarkdown(content: string): Promise<string> {
    const cacheKey = content;
    if (this.cache.has(cacheKey)) {
      return this.cache.get(cacheKey)!;
    }

    try {
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

      const html = result.toString();
      this.cache.set(cacheKey, html);
      return html;
    } catch (error) {
      console.error(DOCS_CONFIG.errorMessages.markdownError, error);
      throw error;
    }
  }

  parseMarkdownFile(fileContent: string) {
    try {
      return matter(fileContent);
    } catch (error) {
      console.error(DOCS_CONFIG.errorMessages.markdownError, error);
      throw error;
    }
  }
}

export const markdownService = MarkdownService.getInstance();
