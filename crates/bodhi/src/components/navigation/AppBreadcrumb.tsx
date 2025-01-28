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
    <div
      className="flex-1 flex h-16 items-center gap-2 px-4"
      data-testid="app-breadcrumb"
    >
      <BookOpenCheck className="size-5" data-testid="app-logo" />
      <Breadcrumb>
        <BreadcrumbList data-testid="breadcrumb-list">
          <BreadcrumbItem>
            <span className="font-semibold" data-testid="breadcrumb-app-name">
              Bodhi
            </span>
          </BreadcrumbItem>
          <BreadcrumbSeparator />
          {parent && (
            <>
              <BreadcrumbItem>
                <BreadcrumbLink
                  href={parent.href || '#'}
                  data-testid="breadcrumb-parent"
                >
                  {parent.title}
                </BreadcrumbLink>
              </BreadcrumbItem>
              <BreadcrumbSeparator />
            </>
          )}
          <BreadcrumbItem>
            <BreadcrumbLink
              href={item.href || '#'}
              data-testid="breadcrumb-current"
            >
              {item.title}
            </BreadcrumbLink>
          </BreadcrumbItem>
        </BreadcrumbList>
      </Breadcrumb>
    </div>
  );
}
