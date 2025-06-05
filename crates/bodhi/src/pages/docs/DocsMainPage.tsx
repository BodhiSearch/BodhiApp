import { useEffect, useState } from 'react';
import { getDocsForSlug } from '@/lib/docs-client';
import { DocGroup } from '@/app/docs/types';
import Link from '@/components/Link';

export default function DocsMainPage() {
  const [docGroups, setDocGroups] = useState<DocGroup[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadDocs() {
      try {
        setLoading(true);
        setError(null);
        const groups = await getDocsForSlug(null);
        setDocGroups(groups);
      } catch (err) {
        console.error('Error loading docs:', err);
        setError('Failed to load documentation');
      } finally {
        setLoading(false);
      }
    }

    loadDocs();
  }, []);

  if (loading) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 rounded w-1/3 mb-8"></div>
          <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
            {[...Array(6)].map((_, i) => (
              <div key={i} className="border rounded-lg p-6">
                <div className="h-6 bg-gray-200 rounded w-2/3 mb-4"></div>
                <div className="space-y-2">
                  <div className="h-4 bg-gray-200 rounded w-full"></div>
                  <div className="h-4 bg-gray-200 rounded w-3/4"></div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="text-center">
          <h1 className="text-2xl font-bold text-red-600 mb-4">Error</h1>
          <p className="text-gray-600">{error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <header className="mb-8">
        <h1 className="text-3xl font-bold mb-4">Documentation</h1>
        <p className="text-lg text-gray-600">
          Welcome to the Bodhi App documentation. Find guides, API references, and tutorials to help you get started.
        </p>
      </header>

      <div className="grid gap-8 md:grid-cols-2 lg:grid-cols-3">
        {docGroups.map((group) => (
          <div key={group.key || group.title} className="border rounded-lg p-6 hover:shadow-lg transition-shadow">
            <h2 className="text-xl font-semibold mb-4">{group.title}</h2>
            <ul className="space-y-2">
              {group.items.map((item) => (
                <li key={item.slug}>
                  <Link
                    href={`/docs/${item.slug}`}
                    className="text-blue-600 hover:text-blue-800 hover:underline"
                  >
                    {item.title}
                  </Link>
                  {item.description && (
                    <p className="text-sm text-gray-600 mt-1">{item.description}</p>
                  )}
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>

      {docGroups.length === 0 && !loading && (
        <div className="text-center py-12">
          <h2 className="text-xl font-semibold mb-4">No Documentation Found</h2>
          <p className="text-gray-600">
            Documentation files should be placed in the <code className="bg-gray-100 px-2 py-1 rounded">src/docs</code> directory.
          </p>
        </div>
      )}
    </div>
  );
}
