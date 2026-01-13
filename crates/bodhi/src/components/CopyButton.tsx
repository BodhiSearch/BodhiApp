import { useState } from 'react';

import { Check, Copy } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { useToastMessages } from '@/hooks/use-toast-messages';

interface CopyButtonProps {
  text: string;
  size?: 'default' | 'sm' | 'lg' | 'icon';
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
  className?: string;
  showToast?: boolean;
}

export const CopyButton = ({
  text,
  size = 'icon',
  variant = 'ghost',
  className = '',
  showToast = true,
}: CopyButtonProps) => {
  const { showError: showErrorToast } = useToastMessages();
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.log('Failed to copy to clipboard:', error);
      if (showToast) {
        showErrorToast('Copy Failed', 'Failed to copy to clipboard');
      }
      setCopied(false);
    }
  };

  return (
    <Button
      variant={variant}
      size={size}
      onClick={handleCopy}
      type="button"
      title="Copy to clipboard"
      className={className}
      data-testid={copied ? 'copied-content' : 'copy-content'}
    >
      {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
    </Button>
  );
};
