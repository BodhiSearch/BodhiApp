'use client';

import * as React from 'react';
import Link from 'next/link';
import { Menu, ChevronsUpDown, Sun, Moon, Monitor } from 'lucide-react';
import { useIsMobile } from '@/hooks/use-mobile';
import { cn } from '@/lib/utils';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuGroup,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Button } from '@/components/ui/button';
import { useNavigation } from '@/hooks/use-navigation';
import { NavigationItem } from '@/types/navigation';
import { LoginMenu } from '@/components/LoginMenu';
import { useTheme } from '@/components/ThemeProvider';

export function AppNavigation() {
  const isMobile = useIsMobile();
  const { navigationItems, currentItem } = useNavigation();
  const { theme, setTheme } = useTheme();

  const isSelected = (item: NavigationItem) => {
    // Direct match
    if (item.href === currentItem.item.href) {
      return true;
    }

    // Check if any skipped child items match current path
    if (item.items) {
      return item.items.some(
        (subItem) => subItem.skip && subItem.href === currentItem.item.href
      );
    }

    return false;
  };

  return (
    <nav
      className={cn('flex items-center p-2', isMobile ? 'w-auto' : 'w-64')}
      data-testid="app-navigation"
    >
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="ghost"
            className={cn(
              'flex items-center gap-2 h-auto py-2',
              isMobile ? 'w-10 px-0' : 'w-full'
            )}
            data-testid="navigation-menu-button"
          >
            {isMobile ? (
              <Menu className="size-5" />
            ) : (
              <>
                {currentItem.item.icon && (
                  <currentItem.item.icon className="size-4" />
                )}
                <div className="flex-1 text-left space-y-0.5">
                  <p className="font-medium">
                    {currentItem.parent?.title || currentItem.item.title}
                  </p>
                  {currentItem.parent && (
                    <p className="text-sm text-muted-foreground">
                      {currentItem.item.title}
                    </p>
                  )}
                </div>
                <ChevronsUpDown className="size-4" />
              </>
            )}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          className="w-72"
          align="start"
          data-testid="navigation-menu-content"
        >
          <DropdownMenuGroup data-testid="navigation-menu-items">
            {navigationItems.map((item) => (
              <React.Fragment key={item.title}>
                {item.items ? (
                  <>
                    <DropdownMenuLabel className="flex items-center gap-2 px-2 py-1.5">
                      {item.icon && <item.icon className="size-4" />}
                      <span className="font-medium">{item.title}</span>
                    </DropdownMenuLabel>
                    {item.items
                      .filter((subItem) => subItem.skip !== true)
                      .map((subItem) => (
                        <Link
                          key={subItem.title}
                          href={subItem.href || '#'}
                          target={subItem.target}
                        >
                          <DropdownMenuItem
                            className={cn(
                              'flex items-center gap-2 pl-8 cursor-pointer',
                              isSelected(subItem) && 'bg-accent'
                            )}
                          >
                            {subItem.icon && (
                              <subItem.icon className="size-4" />
                            )}
                            <div className="space-y-1">
                              <p>{subItem.title}</p>
                              {subItem.description && (
                                <p className="text-xs text-muted-foreground">
                                  {subItem.description}
                                </p>
                              )}
                            </div>
                          </DropdownMenuItem>
                        </Link>
                      ))}
                  </>
                ) : (
                  <Link href={item.href || '#'} target={item.target}>
                    <DropdownMenuItem
                      className={cn(
                        'flex items-center gap-2 cursor-pointer',
                        isSelected(item) && 'bg-accent'
                      )}
                    >
                      {item.icon && <item.icon className="size-4" />}
                      <div className="space-y-1">
                        <p>{item.title}</p>
                        {item.description && (
                          <p className="text-xs text-muted-foreground">
                            {item.description}
                          </p>
                        )}
                      </div>
                    </DropdownMenuItem>
                  </Link>
                )}
                <DropdownMenuSeparator />
              </React.Fragment>
            ))}
          </DropdownMenuGroup>

          <div className="p-2" data-testid="theme-switcher">
            <div className="grid grid-cols-3 gap-2">
              <Button
                variant={theme === 'light' ? 'default' : 'ghost'}
                size="sm"
                onClick={() => setTheme('light')}
              >
                <Sun className="size-5" />
              </Button>
              <Button
                variant={theme === 'dark' ? 'default' : 'ghost'}
                size="sm"
                onClick={() => setTheme('dark')}
              >
                <Moon className="size-5" />
              </Button>
              <Button
                variant={theme === 'system' ? 'default' : 'ghost'}
                size="sm"
                onClick={() => setTheme('system')}
              >
                <Monitor className="size-5" />
              </Button>
            </div>
          </div>

          <DropdownMenuSeparator />
          <LoginMenu />
        </DropdownMenuContent>
      </DropdownMenu>
    </nav>
  );
}
