import { CopyButton } from '@/components/CopyButton';

interface CopyableContentProps {
  text: string;
  className?: string;
}

export function CopyableContent({ text, className = '' }: CopyableContentProps) {
  return (
    <div className={`relative flex items-center group ${className}`}>
      <span className="truncate">{text}</span>
      <div className="opacity-0 group-hover:opacity-100 transition-opacity">
        <CopyButton text={text} />
      </div>
    </div>
  );
}
