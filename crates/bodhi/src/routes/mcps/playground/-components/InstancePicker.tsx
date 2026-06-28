import type { Mcp } from '@bodhiapp/ts-client';
import { ChevronsUpDown } from 'lucide-react';

import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { monogram, tintIndex } from '@/routes/models/explore/-shared/catalog-format';

export interface InstancePickerProps {
  instances: Mcp[];
  selectedId: string;
  onSelect: (id: string) => void;
}

export function InstancePicker({ instances, selectedId, onSelect }: InstancePickerProps) {
  const selected = instances.find((m) => m.id === selectedId);
  const sorted = [...instances].sort((a, b) => a.name.localeCompare(b.name));

  return (
    <div className="pg-instance" data-testid="mcp-playground-instance-picker">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="outline"
            className="pg-instance-btn w-full justify-between"
            data-testid="mcp-playground-instance-trigger"
          >
            <span className="pg-instance-btn-inner">
              {selected ? (
                <>
                  <span className={`pg-instance-glyph cat-tint-${tintIndex(selected.id)}`} aria-hidden>
                    {monogram(selected.name)}
                  </span>
                  <span className="pg-instance-name truncate">{selected.name}</span>
                </>
              ) : (
                <span className="pg-instance-name truncate">Choose an MCP…</span>
              )}
            </span>
            <ChevronsUpDown className="h-3.5 w-3.5 opacity-60" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="pg-instance-menu">
          <DropdownMenuLabel>Switch MCP</DropdownMenuLabel>
          <DropdownMenuSeparator />
          {sorted.length === 0 ? (
            <DropdownMenuItem disabled>No MCPs available</DropdownMenuItem>
          ) : (
            sorted.map((m) => (
              <DropdownMenuItem
                key={m.id}
                onSelect={() => onSelect(m.id)}
                data-testid={`mcp-playground-instance-option-${m.id}`}
                data-test-active={m.id === selectedId}
              >
                <span className={`pg-instance-glyph small cat-tint-${tintIndex(m.id)}`} aria-hidden>
                  {monogram(m.name)}
                </span>
                <span className="truncate">{m.name}</span>
              </DropdownMenuItem>
            ))
          )}
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
