import type { ShellBreadcrumbItem } from '@/components/shell';
import { ROUTE_MCPS } from '@/lib/constants';

export function exploreMcpBreadcrumb(screenName: string): ShellBreadcrumbItem[] {
  return [{ label: 'Bodhi' }, { label: 'MCP', href: ROUTE_MCPS }, { label: screenName, current: true }];
}
