import { cn } from '@/lib/utils';
import { Nav } from '@/app/docs/nav';
import type { SidebarNavProps } from '@/app/docs/types';

export function SidebarNav({ className, items, ...props }: SidebarNavProps) {
  return (
    <div className={cn('space-y-4', className)} {...props}>
      <Nav items={items} />
    </div>
  );
}
