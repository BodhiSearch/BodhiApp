import { useNavigation } from '@/hooks/use-navigation';
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb';
import Image from '@/components/Image';

export function AppBreadcrumb() {
  const { currentItem } = useNavigation();
  const { item, parent } = currentItem;

  return (
    <div className="flex-1 flex h-16 items-center gap-2 px-4" data-testid="app-breadcrumb">
      <Image
        src="/bodhi-logo/bodhi-logo-60.svg"
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
                <BreadcrumbLink href={parent.href} data-testid="breadcrumb-parent">
                  {parent.title}
                </BreadcrumbLink>
              </BreadcrumbItem>
              <BreadcrumbSeparator />
            </>
          )}
          <BreadcrumbItem>
            <BreadcrumbLink href={item.href || '#'} data-testid="breadcrumb-current">
              {item.title}
            </BreadcrumbLink>
          </BreadcrumbItem>
        </BreadcrumbList>
      </Breadcrumb>
    </div>
  );
}
