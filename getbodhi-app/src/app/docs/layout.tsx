import fs from 'fs';
import path from 'path';

import matter from 'gray-matter';
import Link from 'next/link';

import { DOCS_BASE_PATH, DocSidebar } from '@/app/docs/DocSidebar';
import '@/app/docs/prism-theme.css';
import type { MetaData, NavItem } from '@/app/docs/types';
import { getAllDocSlugs, getPathOrder } from '@/app/docs/utils';

function getDocTitle(slug: string): string {
  try {
    const relPath = slug.split('/').join(path.sep);
    const fullPath = path.join(process.cwd(), 'src', 'docs', `${relPath}.md`);
    const fileContents = fs.readFileSync(fullPath, 'utf8');
    const { data } = matter(fileContents);
    return data.title || getDefaultTitle(slug);
  } catch (e) {
    console.error(`Error reading doc title for ${slug}:`, e);
    return getDefaultTitle(slug);
  }
}

function getFolderTitle(folderSlug: string): string {
  const fallback = folderSlug
    .split('/')
    .pop()!
    .replace(/-/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase());
  try {
    const metaPath = path.join(process.cwd(), 'src', 'docs', ...folderSlug.split('/'), '_meta.json');
    if (!fs.existsSync(metaPath)) return fallback;
    const meta = JSON.parse(fs.readFileSync(metaPath, 'utf-8')) as MetaData;
    return meta.title ?? fallback;
  } catch {
    return fallback;
  }
}

function getDefaultTitle(filePath: string): string {
  return (
    filePath
      .split(path.sep)
      .pop()
      ?.replace(/-/g, ' ')
      .replace(/\b\w/g, (c) => c.toUpperCase()) || 'Untitled'
  );
}

function sortNavRecursive(items: NavItem[]): void {
  items.sort((a, b) => getPathOrder(a.slug) - getPathOrder(b.slug));
  items.forEach((item) => {
    if (item.children) sortNavRecursive(item.children);
  });
}

function buildNavigation(): NavItem[] {
  const slugs = getAllDocSlugs();
  const nav: NavItem[] = [];

  slugs.forEach((slug) => {
    const parts = slug.split('/');
    const title = getDocTitle(slug);

    if (parts.length === 1) {
      nav.push({ title, slug });
      return;
    }

    let currentLevel = nav;
    for (let i = 0; i < parts.length - 1; i++) {
      const parentSlug = parts.slice(0, i + 1).join('/');
      let parent = currentLevel.find((item) => item.slug === parentSlug);
      if (!parent) {
        parent = {
          title: getFolderTitle(parentSlug),
          slug: parentSlug,
          children: [],
        };
        currentLevel.push(parent);
      }
      parent.children = parent.children || [];
      currentLevel = parent.children;
    }
    currentLevel.push({ title, slug });
  });

  sortNavRecursive(nav);
  return nav;
}

export default function DocsLayout({ children }: { children: React.ReactNode }) {
  const navigation = buildNavigation();

  return (
    <div className="flex min-h-screen">
      <DocSidebar navigation={navigation} />

      {/* Main content */}
      <main className="flex-1 min-w-0" role="main">
        <div className="h-16 border-b px-6 flex items-center lg:hidden">
          <Link href={DOCS_BASE_PATH}>
            <h2 className="text-lg font-semibold ml-10">Home</h2>
          </Link>
        </div>
        <div className="container py-8 px-6">{children}</div>
      </main>
    </div>
  );
}
