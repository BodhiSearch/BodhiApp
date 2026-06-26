import type { ShellBreadcrumbItem } from '@/components/shell';
import { ROUTE_MCPS } from '@/lib/constants';

/** Breadcrumb for the Explore · MCP Servers catalog page. */
export function exploreMcpBreadcrumb(screenName: string): ShellBreadcrumbItem[] {
  return [{ label: 'Bodhi' }, { label: 'MCP', href: ROUTE_MCPS }, { label: screenName, current: true }];
}
