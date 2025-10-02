import { Button } from '@/components/ui/button';
import { Eye, EyeOff } from 'lucide-react';
import { cn } from '@/lib/utils';

interface ShowHideInputProps {
  value: string;
  shown: boolean;
  onToggle: () => void;
  hiddenChar?: string;
  className?: string;
  inputClassName?: string;
  containerClassName?: string;
  actions?: React.ReactNode;
  'data-testid'?: string;
}

export const ShowHideInput = ({
  value,
  shown,
  onToggle,
  hiddenChar = '•',
  className = '',
  inputClassName = '',
  containerClassName = '',
  actions,
  'data-testid': dataTestId,
}: ShowHideInputProps) => {
  return (
    <div className={cn('relative', containerClassName)} data-testid={dataTestId}>
      <div
        className={cn('rounded-md bg-muted p-3 font-mono text-sm break-all', inputClassName)}
        data-testid={dataTestId ? `${dataTestId}-content` : undefined}
      >
        {shown ? value : hiddenChar.repeat(40)}
      </div>
      <div className={cn('absolute right-2 top-2 space-x-2', className)}>
        <Button
          variant="ghost"
          size="icon"
          onClick={onToggle}
          type="button"
          title={shown ? 'Hide content' : 'Show content'}
          data-testid="toggle-show-content"
        >
          {shown ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
        </Button>
        {actions}
      </div>
    </div>
  );
};
