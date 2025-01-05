import { cn } from '@/lib/utils';
import { PanelLeft } from 'lucide-react';
import * as React from 'react';
import { Button } from '@/components/ui/button';

export interface SidebarToggleProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  icon?: React.ReactNode;
  className?: string;
  side?: 'left' | 'right';
}

export function SidebarToggle({
  open,
  onOpenChange,
  icon,
  className,
  side = 'left',
}: SidebarToggleProps) {
  return (
    <div
      className={cn(
        'fixed top-4 z-40 transition-all duration-300',
        // Left side positioning
        side === 'left' && 'left-0',
        side === 'left' && open && 'left-[16rem]', // Adjust based on sidebar width
        side === 'left' && !open && 'left-4',
        // Right side positioning
        side === 'right' && 'right-0',
        side === 'right' && open && 'right-[16rem]', // Adjust based on sidebar width
        side === 'right' && !open && 'right-4'
      )}
    >
      <Button
        variant="ghost"
        size="icon"
        className={cn('h-7 w-7', className)}
        onClick={() => onOpenChange(!open)}
      >
        {icon || <PanelLeft />}
        <span className="sr-only">Toggle Sidebar</span>
      </Button>
    </div>
  );
}
