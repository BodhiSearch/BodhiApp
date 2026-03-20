'use client';

import { useState } from 'react';

import { SettingInfo } from '@bodhiapp/ts-client';

import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useUpdateSetting } from '@/hooks/useSettings';

interface EditSettingDialogProps {
  setting: SettingInfo;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function EditSettingDialog({ setting, open, onOpenChange }: EditSettingDialogProps) {
  const [value, setValue] = useState(String(setting.current_value));
  const { showSuccess, showError } = useToastMessages();

  const updateSetting = useUpdateSetting({
    onSuccess: () => {
      showSuccess('Success', `Setting ${setting.key} updated successfully`);
      onOpenChange(false);
    },
    onError: (message) => {
      showError('Error', message);
    },
  });

  const handleSubmit = async () => {
    let parsedValue: string | number | boolean = value;

    if (setting.metadata.type === 'number') {
      parsedValue = Number(value);
      if (isNaN(parsedValue)) {
        showError('Error', 'Invalid number');
        return;
      }
      if (setting.metadata.type === 'number') {
        if (parsedValue < setting.metadata.min || parsedValue > setting.metadata.max) {
          showError('Error', `Value must be between ${setting.metadata.min} and ${setting.metadata.max}`);
          return;
        }
      }
    } else if (setting.metadata.type === 'boolean') {
      parsedValue = value === 'true';
    }

    updateSetting.mutate({
      key: setting.key,
      value: parsedValue,
    });
  };

  const renderInput = () => {
    switch (setting.metadata.type) {
      case 'boolean':
        return (
          <div className="flex items-center gap-2">
            <Switch id="value" checked={value === 'true'} onCheckedChange={(checked) => setValue(String(checked))} />
            <label
              htmlFor="value"
              className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
            >
              {value === 'true' ? 'Enabled' : 'Disabled'}
            </label>
          </div>
        );

      case 'option':
        return (
          <Select value={value} onValueChange={setValue}>
            <SelectTrigger className="w-full">
              <SelectValue placeholder="Select a value" />
            </SelectTrigger>
            <SelectContent>
              {setting.metadata.options?.map((option) => (
                <SelectItem key={option} value={option}>
                  {option}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        );

      case 'number':
        return (
          <Input
            id="value"
            type="number"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder="Enter new value"
            min={setting.metadata.type === 'number' ? setting.metadata.min : undefined}
            max={setting.metadata.type === 'number' ? setting.metadata.max : undefined}
            className="w-full"
          />
        );

      default: // string
        return (
          <Input
            id="value"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder="Enter new value"
            className="w-full"
          />
        );
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[calc(100%-2rem)] sm:max-w-[425px]">
        <DialogHeader className="space-y-1">
          <DialogTitle>Edit Setting</DialogTitle>
          <DialogDescription className="text-sm text-muted-foreground break-words">
            Update the value for {setting.key}
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-col gap-4 py-4">
          <div className="flex flex-col gap-2">
            {renderInput()}
            <div className="flex flex-col gap-1">
              <p className="text-xs text-muted-foreground break-words">Default: {String(setting.default_value)}</p>
              {setting.metadata.type === 'number' && (
                <p className="text-xs text-muted-foreground">
                  Range: {setting.metadata.min} - {setting.metadata.max}
                </p>
              )}
            </div>
          </div>
        </div>

        <DialogFooter className="flex flex-col sm:flex-row gap-2">
          <Button variant="outline" onClick={() => onOpenChange(false)} className="w-full sm:w-auto">
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={updateSetting.isLoading} className="w-full sm:w-auto">
            {updateSetting.isLoading ? 'Updating...' : 'Save'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
