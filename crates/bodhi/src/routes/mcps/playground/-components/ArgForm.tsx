import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';

import type { ArgField, InputSchema } from './playgroundTypes';

export type ArgFormProps =
  | {
      kind: 'schema';
      schema: InputSchema | undefined;
      values: Record<string, unknown>;
      onChange: (values: Record<string, unknown>) => void;
      errors?: Record<string, true>;
    }
  | {
      kind: 'fields';
      fields: ArgField[] | undefined;
      values: Record<string, string>;
      onChange: (values: Record<string, string>) => void;
      errors?: Record<string, true>;
    };

/**
 * Unified argument form. JSON-schema mode (tools) drives input type from `inputSchema.properties[*].type`;
 * field-list mode (prompts/templates) renders one string input per arg. The form is intentionally simple —
 * complex schemas degrade to a JSON textarea so users can paste a payload.
 */
export function ArgForm(props: ArgFormProps) {
  if (props.kind === 'schema') return <SchemaArgForm {...props} />;
  return <FieldArgForm {...props} />;
}

function SchemaArgForm({
  schema,
  values,
  onChange,
  errors,
}: {
  schema: InputSchema | undefined;
  values: Record<string, unknown>;
  onChange: (values: Record<string, unknown>) => void;
  errors?: Record<string, true>;
}) {
  const properties = schema?.properties || {};
  const required = schema?.required || [];

  if (Object.keys(properties).length === 0) {
    return (
      <div className="pg-noargs" data-testid="mcp-playground-noargs">
        No inputs needed — just run it.
      </div>
    );
  }

  const handleChange = (key: string, value: unknown) => {
    onChange({ ...values, [key]: value });
  };

  return (
    <div className="pg-form" data-testid="mcp-playground-form">
      {Object.entries(properties).map(([key, prop]) => {
        const isRequired = required.includes(key);
        const propType = (prop?.type as string | undefined) || 'string';
        const err = errors?.[key];
        const desc = (prop?.description as string | undefined) || undefined;
        const placeholder = (prop?.placeholder as string | undefined) || undefined;

        if (propType === 'boolean') {
          return (
            <div key={key} className="pg-field pg-field-checkbox" data-testid={`mcp-playground-param-${key}`}>
              <Checkbox
                id={`mcp-param-${key}`}
                checked={!!values[key]}
                onCheckedChange={(checked) => handleChange(key, !!checked)}
              />
              <Label htmlFor={`mcp-param-${key}`} className="pg-field-label">
                {key}
                {isRequired ? <span className="pg-req">required</span> : <span className="pg-opt">optional</span>}
              </Label>
              {desc && <span className="pg-field-hint">{desc}</span>}
            </div>
          );
        }

        if (propType === 'array' || propType === 'object') {
          const strValue =
            typeof values[key] === 'string'
              ? (values[key] as string)
              : JSON.stringify(values[key] ?? (propType === 'array' ? [] : {}), null, 2);
          return (
            <label className="pg-field" key={key} data-testid={`mcp-playground-param-${key}`}>
              <span className="pg-field-label">
                {key}
                {isRequired ? <span className="pg-req">required</span> : <span className="pg-opt">optional</span>}
              </span>
              <Textarea
                id={`mcp-param-${key}`}
                value={strValue}
                onChange={(e) => {
                  try {
                    handleChange(key, JSON.parse(e.target.value));
                  } catch {
                    handleChange(key, e.target.value);
                  }
                }}
                className={'pg-input pg-textarea' + (err ? ' err' : '')}
                rows={3}
              />
              {desc && <span className="pg-field-hint">{desc}</span>}
            </label>
          );
        }

        return (
          <label className="pg-field" key={key} data-testid={`mcp-playground-param-${key}`}>
            <span className="pg-field-label">
              {key}
              {isRequired ? <span className="pg-req">required</span> : <span className="pg-opt">optional</span>}
            </span>
            <Input
              id={`mcp-param-${key}`}
              type={propType === 'number' || propType === 'integer' ? 'number' : 'text'}
              value={String(values[key] ?? '')}
              placeholder={placeholder}
              className={'pg-input' + (err ? ' err' : '')}
              onChange={(e) => {
                if (propType === 'number' || propType === 'integer') {
                  handleChange(key, e.target.value === '' ? '' : Number(e.target.value));
                } else {
                  handleChange(key, e.target.value);
                }
              }}
            />
            {desc && <span className="pg-field-hint">{desc}</span>}
          </label>
        );
      })}
    </div>
  );
}

function FieldArgForm({
  fields,
  values,
  onChange,
  errors,
}: {
  fields: ArgField[] | undefined;
  values: Record<string, string>;
  onChange: (values: Record<string, string>) => void;
  errors?: Record<string, true>;
}) {
  if (!fields || fields.length === 0) {
    return (
      <div className="pg-noargs" data-testid="mcp-playground-noargs">
        No inputs needed — just run it.
      </div>
    );
  }

  const handleChange = (name: string, value: string) => {
    onChange({ ...values, [name]: value });
  };

  return (
    <div className="pg-form" data-testid="mcp-playground-form">
      {fields.map((f) => {
        const err = errors?.[f.name];
        return (
          <label className="pg-field" key={f.name} data-testid={`mcp-playground-param-${f.name}`}>
            <span className="pg-field-label">
              {f.name}
              {f.required ? <span className="pg-req">required</span> : <span className="pg-opt">optional</span>}
            </span>
            <Input
              id={`mcp-param-${f.name}`}
              type="text"
              value={values[f.name] ?? ''}
              placeholder={f.placeholder}
              className={'pg-input' + (err ? ' err' : '')}
              onChange={(e) => handleChange(f.name, e.target.value)}
            />
            {f.description && <span className="pg-field-hint">{f.description}</span>}
          </label>
        );
      })}
    </div>
  );
}
