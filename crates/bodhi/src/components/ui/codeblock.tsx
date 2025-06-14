// Inspired by Chatbot-UI and modified to fit the needs of this project
// @see https://github.com/mckaywrigley/chatbot-ui/blob/main/components/Markdown/CodeBlock.tsx

'use client';

import { useTheme } from '@/components/ThemeProvider';
import { Button } from '@/components/ui/button';
import { useCopyToClipboard } from '@/hooks/use-copy-to-clipboard';
import { Check, Copy } from 'lucide-react';
import { FC, memo } from 'react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark, oneLight } from 'react-syntax-highlighter/dist/esm/styles/prism';

interface CodeBlockProps {
  language: string;
  value: string;
}

// Map of supported programming languages to their file extensions
const SUPPORTED_LANGUAGES = {
  javascript: 'js',
  typescript: 'ts',
  python: 'py',
  rust: 'rs',
  go: 'go',
  java: 'java',
  kotlin: 'kt',
  swift: 'swift',
  cpp: 'cpp',
  'c++': 'cpp',
  'c#': 'cs',
  ruby: 'rb',
  php: 'php',
  html: 'html',
  css: 'css',
  sql: 'sql',
  shell: 'sh',
  yaml: 'yaml',
  json: 'json',
  markdown: 'md',
} as const;

type SupportedLanguage = keyof typeof SUPPORTED_LANGUAGES;

const CopyButton = memo(({ onClick, isCopied }: { onClick: () => void; isCopied: boolean }) => (
  <Button
    variant="ghost"
    size="icon"
    className="absolute right-0 top-0 h-6 w-6 opacity-0 transition-opacity group-hover:opacity-100"
    onClick={onClick}
  >
    {isCopied ? (
      <Check className="h-3 w-3" data-testid="check-icon" />
    ) : (
      <Copy className="h-3 w-3" data-testid="copy-icon" />
    )}
    <span className="sr-only">Copy code</span>
  </Button>
));
CopyButton.displayName = 'CopyButton';

const LanguageLabel = memo(({ language }: { language: string }) => (
  <div className="flex items-center bg-muted/50 px-4 py-1.5 text-xs text-muted-foreground">
    <span className="lowercase">{language}</span>
  </div>
));
LanguageLabel.displayName = 'LanguageLabel';

export const CodeBlock: FC<CodeBlockProps> = memo(({ language, value }) => {
  const { theme } = useTheme();
  const { isCopied, copyToClipboard } = useCopyToClipboard({ timeout: 2000 });

  const handleCopy = () => {
    if (!isCopied) {
      copyToClipboard(value);
    }
  };

  // Normalize language identifier
  const normalizedLanguage = (language.toLowerCase() as SupportedLanguage) || 'text';

  return (
    <div className="group relative">
      <CopyButton onClick={handleCopy} isCopied={isCopied} />
      <LanguageLabel language={normalizedLanguage} />
      <SyntaxHighlighter
        language={normalizedLanguage}
        style={theme === 'dark' ? oneDark : oneLight}
        showLineNumbers
        className="syntax-highlighter"
        useInlineStyles={true}
        customStyle={{
          margin: 0,
          borderRadius: '0.5rem',
        }}
        codeTagProps={{
          className: 'syntax-highlighter-code',
        }}
      >
        {value}
      </SyntaxHighlighter>
    </div>
  );
});
CodeBlock.displayName = 'CodeBlock';
