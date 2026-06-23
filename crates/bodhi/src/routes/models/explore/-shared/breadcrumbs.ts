import type { ShellBreadcrumbItem } from '@/components/shell';

/** Build the shared Explore catalog breadcrumb; only the leaf screen name differs across views. */
export function exploreBreadcrumb(screenName: string): ShellBreadcrumbItem[] {
  return [{ label: 'Bodhi' }, { label: 'Models', href: '/models/' }, { label: screenName, current: true }];
}
