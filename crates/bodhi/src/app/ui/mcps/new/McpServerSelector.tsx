'use client';

import { Check, ChevronsUpDown, Loader2, Plus } from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from '@/components/ui/command';
import { FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import type { McpServerResponse } from '@/hooks/useMcps';
import type { Control, FieldValues, Path } from 'react-hook-form';

type McpServerSelectorProps<T extends FieldValues> = {
  control: Control<T>;
  name: Path<T>;
  editId: string | null;
  selectedServer: McpServerResponse | null;
  comboboxOpen: boolean;
  onComboboxOpenChange: (open: boolean) => void;
  onServerSelect: (server: McpServerResponse) => void;
  enabledServers: McpServerResponse[];
  loadingServers: boolean;
  isAdmin: boolean;
  isSubmitting: boolean;
};

const McpServerSelector = <T extends FieldValues>({
  control,
  name,
  editId,
  selectedServer,
  comboboxOpen,
  onComboboxOpenChange,
  onServerSelect,
  enabledServers,
  loadingServers,
  isAdmin,
  isSubmitting,
}: McpServerSelectorProps<T>) => {
  const router = useRouter();

  return (
    <FormField
      control={control}
      name={name}
      render={({ field }) => (
        <FormItem className="flex flex-col">
          <FormLabel>MCP Server</FormLabel>
          {editId ? (
            <div className="space-y-2">
              <Input value={selectedServer?.url || ''} disabled data-testid="mcp-server-url-readonly" />
              <p className="text-xs text-muted-foreground">Server: {selectedServer?.name}</p>
            </div>
          ) : (
            <Popover open={comboboxOpen} onOpenChange={onComboboxOpenChange}>
              <PopoverTrigger asChild>
                <FormControl>
                  <Button
                    variant="outline"
                    role="combobox"
                    aria-expanded={comboboxOpen}
                    className={cn('w-full justify-between', !field.value && 'text-muted-foreground')}
                    disabled={isSubmitting}
                    data-testid="mcp-server-combobox"
                  >
                    {selectedServer ? `${selectedServer.name} — ${selectedServer.url}` : 'Select an MCP server...'}
                    <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                  </Button>
                </FormControl>
              </PopoverTrigger>
              <PopoverContent className="w-[--radix-popover-trigger-width] p-0" align="start">
                <Command>
                  <CommandInput placeholder="Search by name, URL, or description..." data-testid="mcp-server-search" />
                  <CommandList>
                    <CommandEmpty>
                      {loadingServers ? (
                        <div className="flex items-center gap-2 py-2">
                          <Loader2 className="h-4 w-4 animate-spin" />
                          Loading servers...
                        </div>
                      ) : (
                        <div className="text-center py-4">
                          <p className="text-sm text-muted-foreground mb-2">No servers found</p>
                          {isAdmin && (
                            <Button asChild variant="link" size="sm">
                              <Link href="/ui/mcp-servers/new">Register a new server</Link>
                            </Button>
                          )}
                        </div>
                      )}
                    </CommandEmpty>
                    <CommandGroup>
                      {enabledServers.map((server) => (
                        <CommandItem
                          key={server.id}
                          value={`${server.name} ${server.url} ${server.description || ''}`}
                          onSelect={() => onServerSelect(server)}
                          data-testid={`mcp-server-option-${server.id}`}
                        >
                          <Check
                            className={cn(
                              'mr-2 h-4 w-4',
                              selectedServer?.id === server.id ? 'opacity-100' : 'opacity-0'
                            )}
                          />
                          <div className="flex-1 min-w-0">
                            <div className="font-medium">{server.name}</div>
                            <div className="text-xs text-muted-foreground font-mono truncate">{server.url}</div>
                            {server.description && (
                              <div className="text-xs text-muted-foreground truncate">{server.description}</div>
                            )}
                          </div>
                        </CommandItem>
                      ))}
                    </CommandGroup>
                    {isAdmin && (
                      <>
                        <CommandSeparator />
                        <CommandGroup>
                          <CommandItem
                            onSelect={() => router.push('/ui/mcp-servers/new')}
                            data-testid="mcp-server-add-new"
                          >
                            <Plus className="mr-2 h-4 w-4" />
                            Add New MCP Server
                          </CommandItem>
                        </CommandGroup>
                      </>
                    )}
                  </CommandList>
                </Command>
              </PopoverContent>
            </Popover>
          )}
          {selectedServer && !editId && <p className="text-xs text-muted-foreground font-mono">{selectedServer.url}</p>}
          <FormMessage />
        </FormItem>
      )}
    />
  );
};

export default McpServerSelector;
