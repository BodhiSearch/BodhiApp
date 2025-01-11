'use client';

import { BookOpenCheck } from 'lucide-react';
import { useNavigation } from '@/hooks/use-navigation';
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb';

export function AppBreadcrumb() {
  const { currentItem } = useNavigation();
  const { item, parent } = currentItem;

  return (
    <div className="flex h-16 shrink-0 items-center gap-2 px-4 flex-1">
      <BookOpenCheck className="h-5 w-5" />
      <Breadcrumb>
        <BreadcrumbList>
          <BreadcrumbItem>
            <span className="font-semibold">Bodhi</span>
          </BreadcrumbItem>
          <BreadcrumbSeparator />
          {parent && (
            <>
              <BreadcrumbItem>
                <BreadcrumbLink href={parent.href || '#'}>
                  {parent.title}
                </BreadcrumbLink>
              </BreadcrumbItem>
              <BreadcrumbSeparator />
            </>
          )}
          <BreadcrumbItem>
            <BreadcrumbLink href={item.href || '#'}>
              {item.title}
            </BreadcrumbLink>
          </BreadcrumbItem>
        </BreadcrumbList>
      </Breadcrumb>
    </div>
  );
}
