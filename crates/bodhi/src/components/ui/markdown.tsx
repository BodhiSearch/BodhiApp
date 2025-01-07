import { FC, memo } from 'react'
import ReactMarkdown, { type Components, type Options } from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { CodeBlock } from './codeblock';
import remarkMath from 'remark-math';


const components: Partial<Components> = {
  // @ts-ignore -- Props from react-markdown include inline type
  code({ node, inline, className, children, ...props }) {
    const match = /language-(\w+)/.exec(className || '')
    if (match) {
      return <CodeBlock
        key={Math.random()}
        language={(match && match[1]) || ''}
        value={String(children).replace(/\n$/, '')}
        {...props}
      />
    }
    if (inline || !String(children).includes('\n')) {
      return (
        <code className={className} {...props}>
          {children}
        </code>
      )
    }
    return <CodeBlock
      key={Math.random()}
      language={''}
      value={String(children)}
      {...props}
    />
  }
};

export const MemoizedReactMarkdown: FC<Options> = memo(
  ({ children, className, ...props }) => (
    <ReactMarkdown
      remarkPlugins={[remarkGfm, remarkMath]}
      className={className}
      components={components}
      {...props}
    >
      {children}
    </ReactMarkdown>
  ),
  (prevProps, nextProps) =>
    prevProps.children === nextProps.children &&
    prevProps.className === nextProps.className
)
