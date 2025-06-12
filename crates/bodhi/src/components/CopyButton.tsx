import { Check, Copy } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useCopyToClipboard } from '@/hooks/use-copy-to-clipboard';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useState } from 'react';

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
  const { copyToClipboard } = useCopyToClipboard();
  const { showSuccess } = useToastMessages();
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await copyToClipboard(text);
    if (showToast) {
      showSuccess('Copied!', 'Copied to clipboard');
    }
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
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
