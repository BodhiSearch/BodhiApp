import React, { useState } from 'react';

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
import { useMediaQuery } from '@/hooks/useMediaQuery';

type Status = {
  value: string;
  label: string;
};

interface ComboBoxResponsiveProps extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'id'> {
  selectedStatus: Status | null;
  setSelectedStatus: (status: Status | null) => void;
  statuses: Status[];
  placeholder?: string;
  id?: string;
  loading?: boolean;
}

const ComboBoxTriggerButton = React.forwardRef<
  HTMLButtonElement,
  {
    testId: string;
    open: boolean;
    id?: string;
    loading: boolean;
    selectedStatus: Status | null;
    placeholder: string;
    buttonProps: Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'id'>;
  } & React.ButtonHTMLAttributes<HTMLButtonElement>
>(({ testId, open, id, loading, selectedStatus, placeholder, buttonProps, ...injectedProps }, ref) => {
  return (
    <Button
      ref={ref}
      variant="outline"
      className="w-full justify-start truncate"
      role="combobox"
      id={id}
      aria-expanded={open}
      aria-haspopup="listbox"
      type="button"
      disabled={loading}
      data-testid={testId}
      {...buttonProps}
      {...injectedProps}
    >
      <span className="truncate">{selectedStatus ? selectedStatus.label : placeholder}</span>
    </Button>
  );
});
ComboBoxTriggerButton.displayName = 'ComboBoxTriggerButton';

export function ComboBoxResponsive({
  selectedStatus,
  setSelectedStatus,
  statuses,
  placeholder = '+ Set status',
  id,
  loading = false,
  ...buttonProps
}: ComboBoxResponsiveProps) {
  const [open, setOpen] = useState(false);
  const isDesktop = useMediaQuery('(min-width: 768px)');
  const isTablet = useMediaQuery('(min-width: 640px) and (max-width: 767px)');

  if (isDesktop) {
    return (
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <ComboBoxTriggerButton
            testId="combobox-trigger"
            open={open}
            id={id}
            loading={loading}
            selectedStatus={selectedStatus}
            placeholder={placeholder}
            buttonProps={buttonProps}
          />
        </PopoverTrigger>
        <PopoverContent className="w-full p-0" align="start">
          <StatusList setOpen={setOpen} setSelectedStatus={setSelectedStatus} statuses={statuses} />
        </PopoverContent>
      </Popover>
    );
  }

  // Mobile and Tablet both use Drawer, but with different testids
  const testId = isTablet ? 'tab-combobox-trigger' : 'm-combobox-trigger';

  return (
    <Drawer open={open} onOpenChange={setOpen}>
      <DrawerTrigger asChild>
        <ComboBoxTriggerButton
          testId={testId}
          open={open}
          id={id}
          loading={loading}
          selectedStatus={selectedStatus}
          placeholder={placeholder}
          buttonProps={buttonProps}
        />
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
              data-testid={`combobox-option-${status.value}`}
            >
              {status.label}
            </CommandItem>
          ))}
        </CommandGroup>
      </CommandList>
    </Command>
  );
}
