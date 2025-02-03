'use client'

import { cn } from '@/lib/utils'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { memo } from 'react'

interface NavItem {
  title: string
  href?: string
  disabled?: boolean
  external?: boolean
  label?: string
  children?: NavItem[]
}

interface NavProps {
  items: NavItem[]
}

export const Nav = memo(function Nav({ items }: NavProps) {
  return (
    <nav 
      className="grid gap-1 w-full" 
      role="navigation"
      aria-label="Documentation navigation"
    >
      {items.map((item, index) => (
        <NavItem key={`nav-item-${item.title}-${index}`} item={item} />
      ))}
    </nav>
  )
})

interface NavItemProps {
  item: NavItem
}

function NavItem({ item }: NavItemProps) {
  const pathname = usePathname()
  const isActive = item.href ? pathname === item.href : false
  const isParentOfActive = item.children?.some(child => child.href === pathname)

  if (item.children) {
    return (
      <div className="grid gap-1 w-full">
        <div
          className={cn(
            "w-full rounded-md",
            "px-3 py-2",
            "text-sm font-medium",
            "transition-colors duration-200",
            "text-gray-900 dark:text-gray-100",
            "hover:bg-accent/25 dark:hover:bg-accent/10",
            isParentOfActive && "bg-accent/50 dark:bg-accent/25"
          )}
          role="button"
          aria-expanded={isParentOfActive}
        >
          {item.title}
        </div>
        <div 
          className="grid gap-1 pl-6 w-full"
          role="group"
          aria-label={`${item.title} sub-navigation`}
        >
          {item.children.map((child, index) => (
            <NavItem 
              key={`${item.title}-child-${child.title}-${index}`} 
              item={child} 
            />
          ))}
        </div>
      </div>
    )
  }

  return (
    <Link
      href={item.href || '#'}
      className={cn(
        "w-full block rounded-md",
        "px-3 py-2",
        "text-sm font-medium",
        "transition-colors duration-200",
        "hover:bg-accent hover:text-accent-foreground",
        isActive && "bg-accent text-accent-foreground",
        item.disabled && "pointer-events-none opacity-60"
      )}
      aria-current={isActive ? 'page' : undefined}
      aria-disabled={item.disabled}
      target={item.external ? '_blank' : undefined}
      rel={item.external ? 'noreferrer' : undefined}
    >
      <span className="flex items-center justify-between">
        {item.title}
        {item.label && (
          <span 
            className={cn(
              "ml-2 rounded-md px-1.5 py-0.5",
              "text-xs leading-none",
              "bg-[#adfa1d] text-[#000000]",
              "no-underline group-hover:no-underline"
            )}
            aria-label={item.label}
          >
            {item.label}
          </span>
        )}
      </span>
    </Link>
  )
} 