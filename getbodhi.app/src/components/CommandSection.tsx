import { CopyableCodeBlock } from './CopyableCodeBlock';

interface CommandSectionProps {
  title: string;
  command: string;
  language?: 'bash' | 'yaml';
  description?: string;
}

export function CommandSection({ title, command, language = 'bash', description }: CommandSectionProps) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold text-muted-foreground">{title}</h3>
      </div>
      {description && <p className="text-sm text-muted-foreground">{description}</p>}
      <CopyableCodeBlock command={command} language={language} />
    </div>
  );
}
