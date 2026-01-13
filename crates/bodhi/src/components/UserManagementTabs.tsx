'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';

import { ROUTE_ACCESS_REQUESTS_PENDING, ROUTE_ACCESS_REQUESTS_ALL, ROUTE_USERS } from '@/lib/constants';
import { cn } from '@/lib/utils';

const tabs = [
  {
    href: ROUTE_ACCESS_REQUESTS_PENDING,
    label: 'Pending Requests',
    description: 'Review new access requests',
  },
  {
    href: ROUTE_ACCESS_REQUESTS_ALL,
    label: 'All Requests',
    description: 'View all access request history',
  },
  {
    href: ROUTE_USERS,
    label: 'All Users',
    description: 'Manage users and roles',
  },
];

export function UserManagementTabs() {
  const pathname = usePathname();

  return (
    <div className="bg-muted/50 p-1 rounded-lg mb-6">
      <nav className="flex space-x-1" aria-label="User Management Navigation">
        {tabs.map((tab) => (
          <Link
            key={tab.href}
            href={tab.href}
            className={cn(
              'px-3 py-2 text-sm font-medium rounded-md transition-all',
              pathname === tab.href
                ? 'bg-background text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground hover:bg-background/50'
            )}
            title={tab.description}
          >
            {tab.label}
          </Link>
        ))}
      </nav>
    </div>
  );
}
