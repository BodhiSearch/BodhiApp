import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';

import { type InputSchema } from './types';

export function FormInput({
  schema,
  values,
  onChange,
}: {
  schema: InputSchema;
  values: Record<string, unknown>;
  onChange: (values: Record<string, unknown>) => void;
}) {
  const properties = schema.properties || {};
  const required = schema.required || [];

  const handleChange = (key: string, value: unknown) => {
    onChange({ ...values, [key]: value });
  };

  return (
    <div className="space-y-4">
      {Object.entries(properties).map(([key, prop]) => {
        const isRequired = required.includes(key);
        const propType = prop.type || 'string';

        if (propType === 'boolean') {
          return (
            <div key={key} className="flex items-center gap-2" data-testid={`mcp-playground-param-${key}`}>
              <Checkbox
                id={`param-${key}`}
                checked={!!values[key]}
                onCheckedChange={(checked) => handleChange(key, !!checked)}
              />
              <Label htmlFor={`param-${key}`}>
                {key}
                {isRequired && <span className="text-destructive ml-1">*</span>}
              </Label>
              {prop.description && <span className="text-xs text-muted-foreground">({prop.description})</span>}
            </div>
          );
        }

        if (propType === 'array' || propType === 'object') {
          const strValue =
            typeof values[key] === 'string'
              ? (values[key] as string)
              : JSON.stringify(values[key] ?? (propType === 'array' ? [] : {}), null, 2);
          return (
            <div key={key} className="space-y-1" data-testid={`mcp-playground-param-${key}`}>
              <Label htmlFor={`param-${key}`}>
                {key}
                {isRequired && <span className="text-destructive ml-1">*</span>}
                {prop.description && <span className="text-xs text-muted-foreground ml-2">({prop.description})</span>}
              </Label>
              <Textarea
                id={`param-${key}`}
                value={strValue}
                onChange={(e) => {
                  try {
                    handleChange(key, JSON.parse(e.target.value));
                  } catch {
                    handleChange(key, e.target.value);
                  }
                }}
                className="font-mono text-sm"
                rows={3}
              />
            </div>
          );
        }

        return (
          <div key={key} className="space-y-1" data-testid={`mcp-playground-param-${key}`}>
            <Label htmlFor={`param-${key}`}>
              {key}
              {isRequired && <span className="text-destructive ml-1">*</span>}
              {prop.description && <span className="text-xs text-muted-foreground ml-2">({prop.description})</span>}
            </Label>
            <Input
              id={`param-${key}`}
              type={propType === 'number' || propType === 'integer' ? 'number' : 'text'}
              value={String(values[key] ?? '')}
              onChange={(e) => {
                if (propType === 'number' || propType === 'integer') {
                  handleChange(key, e.target.value === '' ? '' : Number(e.target.value));
                } else {
                  handleChange(key, e.target.value);
                }
              }}
            />
          </div>
        );
      })}
      {Object.keys(properties).length === 0 && (
        <div className="text-sm text-muted-foreground">This tool has no parameters.</div>
      )}
    </div>
  );
}
