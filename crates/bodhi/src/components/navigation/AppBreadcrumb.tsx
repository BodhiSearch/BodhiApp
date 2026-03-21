'use client';

import Image from 'next/image';
import Link from 'next/link';

import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb';
import { useNavigation } from '@/hooks/navigation';
import { BASE_PATH } from '@/lib/constants';

export function AppBreadcrumb() {
  const { currentItem } = useNavigation();
  const { item, parent } = currentItem;

  return (
    <div className="flex-1 flex h-16 items-center gap-2 px-4" data-testid="app-breadcrumb">
      <Image
        src={`${BASE_PATH}/bodhi-logo/bodhi-logo-60.svg`}
        alt="Bodhi Logo"
        width={20}
        height={20}
        className="text-primary"
        data-testid="app-logo"
      />
      <Breadcrumb>
        <BreadcrumbList data-testid="breadcrumb-list">
          <BreadcrumbItem>
            <span className="font-semibold" data-testid="breadcrumb-app-name">
              Bodhi
            </span>
          </BreadcrumbItem>
          <BreadcrumbSeparator />
          {parent?.href && (
            <>
              <BreadcrumbItem>
                <BreadcrumbLink asChild data-testid="breadcrumb-parent">
                  <Link href={parent.href}>{parent.title}</Link>
                </BreadcrumbLink>
              </BreadcrumbItem>
              <BreadcrumbSeparator />
            </>
          )}
          <BreadcrumbItem>
            <BreadcrumbLink asChild data-testid="breadcrumb-current">
              <Link href={item.href || '#'}>{item.title}</Link>
            </BreadcrumbLink>
          </BreadcrumbItem>
        </BreadcrumbList>
      </Breadcrumb>
    </div>
  );
}
