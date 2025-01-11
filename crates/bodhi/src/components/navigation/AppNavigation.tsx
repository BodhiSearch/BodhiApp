'use client';

import * as React from 'react';
import Link from 'next/link';
import { Menu, ChevronsUpDown } from 'lucide-react';
import { useIsMobile } from '@/hooks/use-mobile';
import { cn } from '@/lib/utils';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuGroup,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuPortal,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Button } from '@/components/ui/button';
import { useNavigation } from '@/hooks/use-navigation';
import { NavigationItem } from '@/types/navigation';

export function AppNavigation() {
  const isMobile = useIsMobile();
  const { navigationItems, currentItem } = useNavigation();

  const isSelected = (item: NavigationItem) => {
    return item.href === currentItem.item.href;
  };

  return (
    <div
      className={cn(
        'flex items-center p-2',
        !isMobile && 'w-64' // Set width to 256px only for desktop
      )}
    >
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="ghost"
            className={cn(
              'flex items-center gap-2 h-auto py-2 w-full',
              isMobile && 'w-10 px-0'
            )}
          >
            {isMobile ? (
              <Menu className="h-5 w-5" />
            ) : (
              <>
                <div className="flex items-center justify-center rounded-lg">
                  {currentItem.item.icon && (
                    <currentItem.item.icon className="h-4 w-4" />
                  )}
                </div>
                <div className="flex flex-col gap-0.5 items-start flex-1">
                  <span className="font-medium">
                    {currentItem.parent?.title || currentItem.item.title}
                  </span>
                  {currentItem.parent && (
                    <span className="text-sm text-muted-foreground">
                      {currentItem.item.title}
                    </span>
                  )}
                </div>
                <ChevronsUpDown className="h-4 w-4" />
              </>
            )}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent className="w-64" align="start">
          <DropdownMenuGroup>
            {navigationItems.map((item) => {
              if (item.items) {
                return (
                  <DropdownMenuSub key={item.title}>
                    <DropdownMenuSubTrigger
                      className={cn(
                        'flex items-center gap-2',
                        isSelected(item) && 'bg-accent'
                      )}
                    >
                      {item.icon && <item.icon className="h-4 w-4" />}
                      <span>{item.title}</span>
                    </DropdownMenuSubTrigger>
                    <DropdownMenuPortal>
                      <DropdownMenuSubContent>
                        {item.items.map((subItem) => (
                          <Link key={subItem.title} href={subItem.href || '#'}>
                            <DropdownMenuItem
                              className={cn(
                                'flex items-center gap-2 cursor-pointer',
                                isSelected(subItem) && 'bg-accent'
                              )}
                            >
                              {subItem.icon && (
                                <subItem.icon className="h-4 w-4" />
                              )}
                              <div className="flex flex-col gap-1">
                                <span>{subItem.title}</span>
                                {subItem.description && (
                                  <span className="text-xs text-muted-foreground">
                                    {subItem.description}
                                  </span>
                                )}
                              </div>
                            </DropdownMenuItem>
                          </Link>
                        ))}
                      </DropdownMenuSubContent>
                    </DropdownMenuPortal>
                  </DropdownMenuSub>
                );
              }

              return (
                <Link key={item.title} href={item.href || '#'}>
                  <DropdownMenuItem
                    className={cn(
                      'flex items-center gap-2 cursor-pointer',
                      isSelected(item) && 'bg-accent'
                    )}
                  >
                    {item.icon && <item.icon className="h-4 w-4" />}
                    <div className="flex flex-col gap-1">
                      <span>{item.title}</span>
                      {item.description && (
                        <span className="text-xs text-muted-foreground">
                          {item.description}
                        </span>
                      )}
                    </div>
                  </DropdownMenuItem>
                </Link>
              );
            })}
          </DropdownMenuGroup>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
