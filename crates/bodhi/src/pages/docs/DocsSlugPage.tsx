import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { getDocContent, getDocDetails } from '@/lib/docs-client';
import { DocDetails } from '@/components/docs/types';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { tomorrow } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { notFound } from '@/lib/navigation';

interface DocsSlugPageProps {
  params?: { slug: string[] };
}

export default function DocsSlugPage({ params }: DocsSlugPageProps) {
  const routerParams = useParams();
  const [content, setContent] = useState<string>('');
  const [docDetails, setDocDetails] = useState<DocDetails | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Get slug from either props (for lazy loading) or router params
  const slug = params?.slug || (routerParams['*'] ? routerParams['*'].split('/') : []);
  const slugString = Array.isArray(slug) ? slug.join('/') : slug;

  useEffect(() => {
    async function loadDoc() {
      if (!slugString) {
        setError('No document specified');
        setLoading(false);
        return;
      }

      try {
        setLoading(true);
        setError(null);

        const [docContent, details] = await Promise.all([
          getDocContent(slugString),
          getDocDetails(slugString)
        ]);

        if (!docContent || !details) {
          notFound();
          return;
        }

        setContent(docContent.content);
        setDocDetails(details);
      } catch (err) {
        console.error('Error loading document:', err);
        setError('Failed to load document');
      } finally {
        setLoading(false);
      }
    }

    loadDoc();
  }, [slugString]);

  if (loading) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 rounded w-1/3 mb-4"></div>
          <div className="h-4 bg-gray-200 rounded w-full mb-2"></div>
          <div className="h-4 bg-gray-200 rounded w-full mb-2"></div>
          <div className="h-4 bg-gray-200 rounded w-2/3"></div>
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

  if (!docDetails) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="text-center">
          <h1 className="text-2xl font-bold mb-4">Document Not Found</h1>
          <p className="text-gray-600">The requested document could not be found.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <article className="prose prose-lg max-w-none">
        <header className="mb-8">
          <h1 className="text-3xl font-bold mb-2">{docDetails.title}</h1>
          {docDetails.description && (
            <p className="text-lg text-gray-600">{docDetails.description}</p>
          )}
        </header>
        
        <div className="markdown-content">
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            components={{
              code({ className, children, ...props }: any) {
                const match = /language-(\w+)/.exec(className || '');
                const isInline = !props.node || props.node.tagName !== 'pre';

                return !isInline && match ? (
                  <SyntaxHighlighter
                    style={tomorrow as any}
                    language={match[1]}
                    PreTag="div"
                    {...props}
                  >
                    {String(children).replace(/\n$/, '')}
                  </SyntaxHighlighter>
                ) : (
                  <code className={className} {...props}>
                    {children}
                  </code>
                );
              },
            }}
          >
            {content}
          </ReactMarkdown>
        </div>
      </article>
    </div>
  );
}
