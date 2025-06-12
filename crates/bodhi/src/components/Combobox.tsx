import { Button } from '@/components/ui/button';
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from '@/components/ui/command';
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
} from '@/components/ui/drawer';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { useMediaQuery } from '@/hooks/use-media-query';
import { useState } from 'react';

type Status = {
  value: string;
  label: string;
};

interface ComboBoxResponsiveProps {
  selectedStatus: Status | null;
  setSelectedStatus: (status: Status | null) => void;
  statuses: Status[];
  placeholder?: string;
  id?: string;
  loading?: boolean;
}

export function ComboBoxResponsive({
  selectedStatus,
  setSelectedStatus,
  statuses,
  placeholder = '+ Set status',
  id,
  loading = false,
}: ComboBoxResponsiveProps) {
  const [open, setOpen] = useState(false);
  const isDesktop = useMediaQuery('(min-width: 768px)');

  if (isDesktop) {
    return (
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            className="w-full justify-start truncate"
            role="combobox"
            id={id}
            aria-expanded={open}
            aria-haspopup="listbox"
            type="button"
            disabled={loading}
          >
            <span className="truncate">{selectedStatus ? selectedStatus.label : placeholder}</span>
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-full p-0" align="start">
          <StatusList setOpen={setOpen} setSelectedStatus={setSelectedStatus} statuses={statuses} />
        </PopoverContent>
      </Popover>
    );
  }

  return (
    <Drawer open={open} onOpenChange={setOpen}>
      <DrawerTrigger asChild>
        <Button
          variant="outline"
          className="w-full justify-start truncate"
          role="combobox"
          id={id}
          aria-expanded={open}
          aria-haspopup="listbox"
          type="button"
          disabled={loading}
        >
          <span className="truncate">{selectedStatus ? selectedStatus.label : placeholder}</span>
        </Button>
      </DrawerTrigger>
      <DrawerContent>
        <div className="mx-4 mt-4">
          <DrawerHeader>
            <DrawerTitle>Select Status</DrawerTitle>
            <DrawerDescription>Choose a status from the list below</DrawerDescription>
          </DrawerHeader>
          <div className="border-t">
            <StatusList setOpen={setOpen} setSelectedStatus={setSelectedStatus} statuses={statuses} />
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  );
}

function StatusList({
  setOpen,
  setSelectedStatus,
  statuses,
}: {
  setOpen: (open: boolean) => void;
  setSelectedStatus: (status: Status | null) => void;
  statuses: Status[];
}) {
  return (
    <Command>
      <CommandInput placeholder="Filter ..." />
      <CommandList>
        <CommandEmpty>No results found.</CommandEmpty>
        <CommandGroup>
          {statuses.map((status) => (
            <CommandItem
              key={status.value}
              value={status.value}
              onSelect={(value) => {
                setSelectedStatus(statuses.find((priority) => priority.value === value) || null);
                setOpen(false);
              }}
            >
              {status.label}
            </CommandItem>
          ))}
        </CommandGroup>
      </CommandList>
    </Command>
  );
}
