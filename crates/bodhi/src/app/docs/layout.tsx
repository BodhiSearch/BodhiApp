import fs from 'fs';
import path from 'path';
import matter from 'gray-matter';
import '@/app/docs/prism-theme.css';
import { getAllDocPaths } from '@/app/docs/utils';
import type { NavItem } from '@/app/docs/types';
import { getPathOrder } from '@/app/docs/config';
import { DocSidebar } from './DocSidebar';

function getDocTitle(filePath: string): string {
  try {
    const fullPath = path.join(process.cwd(), 'src/docs', `${filePath}.md`);
    const fileContents = fs.readFileSync(fullPath, 'utf8');
    const { data } = matter(fileContents);
    return data.title || getDefaultTitle(filePath);
  } catch (e) {
    console.error(`Error reading doc title for ${filePath}:`, e);
    return getDefaultTitle(filePath);
  }
}

function getDefaultTitle(filePath: string): string {
  return (
    filePath
      .split('/')
      .pop()
      ?.replace(/-/g, ' ')
      .replace(/\b\w/g, (c) => c.toUpperCase()) || 'Untitled'
  );
}

function buildNavigation(): NavItem[] {
  const paths = getAllDocPaths();
  const nav: NavItem[] = [];

  // Sort paths based on our custom order
  paths
    .sort((a, b) => {
      const orderA = getPathOrder(a);
      const orderB = getPathOrder(b);
      return orderA - orderB;
    })
    .forEach((path) => {
      const parts = path.split('/');
      const title = getDocTitle(path);

      if (parts.length === 1) {
        nav.push({ title, slug: path });
      } else {
        // Handle nested paths
        let currentLevel = nav;
        for (let i = 0; i < parts.length - 1; i++) {
          const parentSlug = parts.slice(0, i + 1).join('/');
          let parent = currentLevel.find((item) => item.slug === parentSlug);
          if (!parent) {
            parent = {
              title: parts[i]
                .replace(/-/g, ' ')
                .replace(/\b\w/g, (c) => c.toUpperCase()),
              slug: parentSlug,
              children: [],
            };
            currentLevel.push(parent);
          }
          parent.children = parent.children || [];
          currentLevel = parent.children;
          currentLevel.sort(
            (a, b) => getPathOrder(a.slug) - getPathOrder(b.slug)
          );
        }
        currentLevel.push({
          title,
          slug: path,
        });
        currentLevel.sort(
          (a, b) => getPathOrder(a.slug) - getPathOrder(b.slug)
        );
      }
    });

  return nav;
}

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const navigation = buildNavigation();

  return (
    <div className="flex min-h-screen">
      <DocSidebar navigation={navigation} />

      {/* Main content */}
      <main className="flex-1 min-w-0" role="main">
        <div className="h-16 border-b px-6 flex items-center lg:hidden">
          <h2 className="text-lg font-semibold ml-12">Documentation</h2>
        </div>
        <div className="container py-8 px-6">{children}</div>
      </main>
    </div>
  );
}
