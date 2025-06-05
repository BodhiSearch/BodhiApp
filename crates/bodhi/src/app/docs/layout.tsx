import { DOCS_BASE_PATH, DocSidebar } from '@/app/docs/DocSidebar';
import '@/app/docs/prism-theme.css';
import type { NavItem } from '@/app/docs/types';
import { getAllDocSlugs, getPathOrder } from '@/app/docs/utils';
import fs from 'fs';
import matter from 'gray-matter';
import Link from '@/components/Link';
import path from 'path';

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

function getDefaultTitle(filePath: string): string {
  return (
    filePath
      .split(path.sep)
      .pop()
      ?.replace(/-/g, ' ')
      .replace(/\b\w/g, (c) => c.toUpperCase()) || 'Untitled'
  );
}

function buildNavigation(): NavItem[] {
  const slugs = getAllDocSlugs();
  const nav: NavItem[] = [];

  // Sort paths based on our custom order
  slugs
    .sort((a, b) => {
      const orderA = getPathOrder(a);
      const orderB = getPathOrder(b);
      return orderA - orderB;
    })
    .forEach((slug) => {
      const parts = slug.split('/');
      const title = getDocTitle(slug);

      if (parts.length === 1) {
        nav.push({ title, slug: slug });
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
          slug: slug,
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
          <Link href={DOCS_BASE_PATH}>
            <h2 className="text-lg font-semibold ml-10">Home</h2>
          </Link>
        </div>
        <div className="container py-8 px-6">{children}</div>
      </main>
    </div>
  );
}
