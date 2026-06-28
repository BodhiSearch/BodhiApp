import { Fragment, type ReactNode } from 'react';

import { BASE_PATH } from '@/lib/constants';

import { ShellIcon } from './ShellIcon';

export interface ShellBreadcrumbItem {
  label: string;
  href?: string;
  current?: boolean;
}

export interface ShellBreadcrumbProps {
  items?: ShellBreadcrumbItem[] | ReactNode;
}

/** Resolve a breadcrumb href so it works under the SPA's `/ui` basepath. Callers pass clean
 *  route paths (e.g. `/models/`); we prepend BASE_PATH for absolute paths and leave hashes,
 *  protocol-relative URLs, and externals untouched. */
function resolveBreadcrumbHref(href?: string): string {
  if (!href) return '#';
  if (href.startsWith('#')) return href;
  if (href.startsWith(BASE_PATH + '/') || href === BASE_PATH) return href;
  if (/^([a-z]+:)?\/\//i.test(href)) return href;
  if (href.startsWith('/')) return BASE_PATH + href;
  return href;
}

export function ShellBreadcrumb({ items }: ShellBreadcrumbProps) {
  if (!items) return null;
  if (!Array.isArray(items)) return <div className="shell-bc">{items}</div>;
  return (
    <div className="shell-bc">
      {items.map((it, i) => (
        <Fragment key={i}>
          {i > 0 && <ShellIcon name="chevron-right" size={11} />}
          {it.current ? (
            <span className="shell-bc-current">{it.label}</span>
          ) : (
            <a className="shell-bc-seg" href={resolveBreadcrumbHref(it.href)}>
              {it.label}
            </a>
          )}
        </Fragment>
      ))}
    </div>
  );
}
