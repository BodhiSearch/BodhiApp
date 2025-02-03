import { getDocsForPath } from '@/app/docs/utils';
import Link from 'next/link';

export default function DocsPage() {
  const sortedGroups = getDocsForPath(null);

  return (
    <div className="max-w-none prose prose-slate dark:prose-invert">
      <h1>Documentation</h1>
      <p className="lead">
        Welcome to our documentation. Choose a topic below to get started.
      </p>

      {sortedGroups.map((group) => (
        <section key={group.key} className="mb-12">
          <h2 className="text-2xl font-bold mb-4">{group.title}</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {group.items.map((doc) => (
              <Link
                key={doc.slug}
                href={`/docs/${doc.slug}`}
                className="block p-4 border rounded-lg hover:border-blue-500 transition-colors no-underline"
              >
                <h3 className="text-lg font-semibold mb-1 mt-0">{doc.title}</h3>
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
