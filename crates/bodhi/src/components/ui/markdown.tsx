import { CodeBlock } from '@/components/ui/codeblock';
import { FC, memo } from 'react';
import ReactMarkdown, { type Components, type Options } from 'react-markdown';
import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import { cn } from '@/lib/utils';

const components: Partial<Components> = {
  // @ts-ignore -- Props from react-markdown include inline type
  code({ node, inline, className, children, ...props }) {
    const match = /language-(\w+)/.exec(className || '');
    if (match) {
      return (
        <CodeBlock
          key={Math.random()}
          language={(match && match[1]) || ''}
          value={String(children).replace(/\n$/, '')}
          {...props}
        />
      );
    }
    if (inline || !String(children).includes('\n')) {
      return (
        <code className={cn('rounded bg-muted px-1 py-0.5', className)} {...props}>
          {children}
        </code>
      );
    }

    // Handle code blocks
    return (
      <CodeBlock
        key={Math.random()}
        language={(match && match[1]) || ''}
        value={String(children).replace(/\n$/, '')}
        {...props}
      />
    );
  },
  p: ({ children }) => <p className="leading-7 [&:not(:first-child)]:mt-4">{children}</p>,
  ul: ({ children }) => <ul className="my-4 ml-6 list-disc marker:text-muted-foreground">{children}</ul>,
  ol: ({ children }) => <ol className="my-4 ml-6 list-decimal marker:text-muted-foreground">{children}</ol>,
  li: ({ children }) => <li className="mt-2">{children}</li>,
  h1: ({ children }) => <h1 className="mt-6 scroll-m-20 text-4xl font-bold">{children}</h1>,
  h2: ({ children }) => <h2 className="mt-6 scroll-m-20 text-3xl font-semibold">{children}</h2>,
  h3: ({ children }) => <h3 className="mt-6 scroll-m-20 text-2xl font-semibold">{children}</h3>,
  h4: ({ children }) => <h4 className="mt-6 scroll-m-20 text-xl font-semibold">{children}</h4>,
  a: ({ children, href }) => (
    <a
      href={href}
      className="font-medium underline underline-offset-4 hover:text-primary"
      target="_blank"
      rel="noopener noreferrer"
    >
      {children}
    </a>
  ),
  blockquote: ({ children }) => (
    <blockquote className="mt-4 border-l-2 border-muted pl-4 italic text-muted-foreground">{children}</blockquote>
  ),
  hr: () => <hr className="my-6 border-muted" />,
  table: ({ children }) => (
    <div className="my-4 w-full overflow-y-auto">
      <table className="w-full border-collapse text-sm">{children}</table>
    </div>
  ),
  th: ({ children }) => <th className="border border-muted px-4 py-2 text-left font-bold">{children}</th>,
  td: ({ children }) => <td className="border border-muted px-4 py-2">{children}</td>,
};

export const MemoizedReactMarkdown: FC<Options> = memo(
  ({ children, className, ...props }) => (
    <div className="text-base text-foreground">
      <ReactMarkdown
        remarkPlugins={[remarkGfm, remarkMath]}
        className={cn(
          // Base styles
          'space-y-4',
          // Text formatting
          'text-base leading-7',
          // Lists
          '[&>ul]:list-disc [&>ol]:list-decimal',
          '[&>ul]:ml-6 [&>ol]:ml-6',
          // Code blocks
          '[&>pre]:my-4 [&>pre]:overflow-auto',
          '[&>pre]:rounded-lg [&>pre]:bg-muted',
          '[&>pre]:p-4',
          // Custom className
          className
        )}
        components={components}
        {...props}
      >
        {children}
      </ReactMarkdown>
    </div>
  ),
  (prevProps, nextProps) => prevProps.children === nextProps.children && prevProps.className === nextProps.className
);
