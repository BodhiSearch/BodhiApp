'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';

import { ROUTE_MCP_SERVERS, ROUTE_MCPS } from '@/lib/constants';
import { cn } from '@/lib/utils';

const tabs = [
  {
    href: ROUTE_MCPS,
    label: 'My MCPs',
    description: 'Manage your MCP instances',
  },
  {
    href: ROUTE_MCP_SERVERS,
    label: 'MCP Servers',
    description: 'Browse available MCP servers',
  },
];

export function McpManagementTabs() {
  const pathname = usePathname();

  const isActive = (href: string) => pathname?.startsWith(href);

  return (
    <div className="bg-muted/50 p-1 rounded-lg mb-6">
      <nav className="flex space-x-1" aria-label="MCP Management Navigation" data-testid="mcp-management-tabs">
        {tabs.map((tab) => (
          <Link
            key={tab.href}
            href={tab.href}
            className={cn(
              'px-3 py-2 text-sm font-medium rounded-md transition-all',
              isActive(tab.href)
                ? 'bg-background text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground hover:bg-background/50'
            )}
            title={tab.description}
            data-testid={`mcp-tab-${tab.href.split('/').pop()}`}
          >
            {tab.label}
          </Link>
        ))}
      </nav>
    </div>
  );
}
