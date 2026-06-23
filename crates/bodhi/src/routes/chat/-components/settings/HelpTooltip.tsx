import { HelpCircle } from 'lucide-react';

import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

interface HelpTooltipProps {
  text?: string;
  children?: React.ReactNode;
  sideOffset?: number;
  className?: string;
}

export function HelpTooltip({ text, children, sideOffset, className }: HelpTooltipProps) {
  return (
    <TooltipProvider>
      <Tooltip delayDuration={300}>
        <TooltipTrigger asChild>
          <HelpCircle className="h-4 w-4 text-muted-foreground hover:text-foreground transition-colors cursor-help" />
        </TooltipTrigger>
        <TooltipContent sideOffset={sideOffset} className={className}>
          {children ?? <p className="max-w-xs text-sm">{text}</p>}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
