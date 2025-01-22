'use client';

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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { useToast } from '@/hooks/use-toast';
import { useUpdateSetting } from '@/hooks/useQuery';
import { Setting } from '@/types/models';
import { useState } from 'react';

interface EditSettingDialogProps {
  setting: Setting;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function EditSettingDialog({
  setting,
  open,
  onOpenChange,
}: EditSettingDialogProps) {
  const [value, setValue] = useState(String(setting.current_value));
  const { toast } = useToast();
  const updateSetting = useUpdateSetting();

  const handleSubmit = async () => {
    try {
      let parsedValue: string | number | boolean = value;

      // Parse value based on type
      if (setting.metadata.type === 'number') {
        parsedValue = Number(value);
        if (isNaN(parsedValue)) {
          throw new Error('Invalid number');
        }
        // Validate range if specified
        if (setting.metadata.range) {
          if (
            parsedValue < setting.metadata.range.min ||
            parsedValue > setting.metadata.range.max
          ) {
            throw new Error(
              `Value must be between ${setting.metadata.range.min} and ${setting.metadata.range.max}`
            );
          }
        }
      } else if (setting.metadata.type === 'boolean') {
        parsedValue = value === 'true';
      }

      await updateSetting.mutateAsync({
        key: setting.key,
        value: parsedValue,
      });

      toast({
        title: 'Success',
        description: 'Setting updated successfully',
      });
      onOpenChange(false);
    } catch (error) {
      toast({
        title: 'Error',
        description:
          error instanceof Error ? error.message : 'Failed to update setting',
        variant: 'destructive',
      });
    }
  };

  const renderInput = () => {
    switch (setting.metadata.type) {
      case 'boolean':
        return (
          <div className="flex items-center space-x-2">
            <Switch
              id="value"
              checked={value === 'true'}
              onCheckedChange={(checked) => setValue(String(checked))}
            />
            <label htmlFor="value" className="text-sm font-medium">
              {value === 'true' ? 'Enabled' : 'Disabled'}
            </label>
          </div>
        );

      case 'option':
        return (
          <Select value={value} onValueChange={setValue}>
            <SelectTrigger>
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
            min={setting.metadata.range?.min}
            max={setting.metadata.range?.max}
          />
        );

      default: // string
        return (
          <Input
            id="value"
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder="Enter new value"
          />
        );
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Edit Setting</DialogTitle>
          <DialogDescription>
            Update the value for {setting.key}
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="space-y-2">
            {renderInput()}
            <p className="text-xs text-muted-foreground">
              Default: {String(setting.default_value)}
            </p>
            {setting.metadata.type === 'number' && setting.metadata.range && (
              <p className="text-xs text-muted-foreground">
                Range: {setting.metadata.range.min} -{' '}
                {setting.metadata.range.max}
              </p>
            )}
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={updateSetting.isLoading}>
            {updateSetting.isLoading ? 'Updating...' : 'Save'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
